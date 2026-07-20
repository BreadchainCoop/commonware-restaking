//! Jito NCN BN254 multisig [`certificate::Scheme`](commonware_cryptography::certificate::Scheme)
//! for the `commonware-consensus` aggregation engine.
//!
//! [`JitoBn254Scheme`] is modeled on the chassis EigenLayer scheme
//! (`commonware_avs_core::bn254::Bn254Scheme`) with the signature domain swapped
//! for the NCN program's (INTERFACES.md §1):
//!
//! 1. **Signing covers ONLY the raw 32-byte digest**, hashed to G1 with
//!    `ncn-program-core`'s `Sha256Normalized` (domain-tagged sha256,
//!    reject-and-retry). This is what keeps assembled certificates verifiable by
//!    the deployed NCN program, which recomputes exactly that hash-to-curve
//!    on-chain in `VerifyCertificate`.
//! 2. Certificates carry the aggregated G1 signature (compressed, 32 bytes),
//!    the aggregated G2 key of the signers (compressed, 64 bytes) and the
//!    signer bitmap — the wire triple `VerifyCertificate` consumes
//!    (`aggregated_signature`, `aggregated_g2`,
//!    `operators_signature_bitmap`).
//! 3. [`Scheme::verify_certificate`] runs the NCN program's own verification:
//!    the single challenge-combined pairing over EigenLayer's exact gamma
//!    (`verify_aggregated_signature::<Sha256Normalized>`), with `apk1`
//!    recomputed from the signer bitmap (on-chain it is the snapshot APK minus
//!    non-signers — same point, different arithmetic path).
//!
//! # Replay domain (READ BEFORE "FIXING")
//!
//! As in the chassis scheme, the signature binds only the digest — not the
//! `Item`'s height, the epoch, or any namespace. Replay protection is
//! consumer-local on Solana (the settlement program's `transition_count`,
//! bound INSIDE the digest preimage; see INTERFACES.md §4 and the D-MSG
//! decision). Do NOT add the height or a namespace to the preimage — that
//! would break on-chain verification.
//!
//! Rogue-key note: aggregation is plain point addition; safe only because the
//! NCN program verifies a proof of possession at operator registration
//! (`verify_operator_registeration`, pop gamma).

use crate::bn254::{
    Bn254Signer, G1PublicKey, PrivateKey, PublicKey, Signature, aggregate_g1_keys,
    aggregate_g2_keys, aggregate_signatures, aggregate_verify,
};
use bytes::{Buf, BufMut};
use commonware_codec::{EncodeSize, Error, Read, ReadExt, Write};
use commonware_consensus::aggregation::types::Item;
use commonware_cryptography::certificate::{Attestation, Scheme, Signers};
use commonware_cryptography::{Digest, Signer as _};
use commonware_parallel::Strategy;
use commonware_utils::ordered::{Quorum, Set};
use commonware_utils::{Faults, Participant};
use ncn_program_core::g2_point::{G2CompressedPoint, G2Point};
use rand_core::CryptoRngCore;

/// Extracts the raw 32 digest bytes from an [`Item`]'s digest.
///
/// The scheme only supports 32-byte digests (`sha256::Digest` in this system):
/// `Sha256Normalized` consumes exactly 32 bytes and the deployed NCN program
/// verifies 32-byte task digests. Returns `None` for any other digest width so
/// a mis-instantiated scheme fails verification rather than the process.
fn item_digest<D: Digest>(item: &Item<D>) -> Option<[u8; 32]> {
    item.digest.as_ref().try_into().ok()
}

/// Jito NCN BN254 multisig certificate scheme over a fixed participant set.
///
/// Participant indices are positions in the `ordered::Set` of compressed G2
/// keys (sorted by on-chain compressed bytes) — every participant and verifier
/// MUST build the set from the same `NCNOperatorAccount` reads or attestations
/// will be attributed to the wrong signer. NOTE: participant indices are NOT
/// the on-chain `ncn_operator_index`; the submitter maps between the two when
/// building the on-chain bitmap (see `crate::network::JitoQuorum`).
#[derive(Clone, Debug)]
pub struct JitoBn254Scheme {
    /// Ordered G2 identity/signing keys; participant indices are positions here.
    participants: Set<PublicKey>,
    /// G1 public keys in the SAME order as `participants` (index-aligned).
    g1_keys: Vec<G1PublicKey>,
    /// Our participant index and signer; `None` for verifier-only instances.
    signer: Option<(Participant, Bn254Signer)>,
}

impl JitoBn254Scheme {
    /// Creates a signing scheme instance.
    ///
    /// `g1_keys` must be index-aligned with `participants` (i.e. `g1_keys[i]`
    /// is the G1 key registered for `participants[i]`); build both from the
    /// same sorted operator list.
    ///
    /// Returns `None` if `private_key`'s G2 public key is not in the
    /// participant set (mirroring the chassis scheme).
    ///
    /// # Panics
    ///
    /// Panics if `g1_keys.len() != participants.len()` or the participant set
    /// is empty — construction-time configuration errors, never network input.
    pub fn signer(
        participants: Set<PublicKey>,
        g1_keys: Vec<G1PublicKey>,
        private_key: PrivateKey,
    ) -> Option<Self> {
        Self::validate_participants(&participants, &g1_keys);
        let signer = Bn254Signer::new(private_key)?;
        let index = participants.index(&signer.public_key())?;
        Some(Self {
            participants,
            g1_keys,
            signer: Some((index, signer)),
        })
    }

    /// Creates a verifier-only scheme instance (`me() == None`): validates
    /// acks, assembles and verifies certificates, but never signs. This is
    /// what an aggregator/router runs.
    ///
    /// # Panics
    ///
    /// Panics if `g1_keys.len() != participants.len()` or the participant set
    /// is empty — construction-time configuration errors, never network input.
    pub fn verifier(participants: Set<PublicKey>, g1_keys: Vec<G1PublicKey>) -> Self {
        Self::validate_participants(&participants, &g1_keys);
        Self {
            participants,
            g1_keys,
            signer: None,
        }
    }

    fn validate_participants(participants: &Set<PublicKey>, g1_keys: &[G1PublicKey]) {
        assert!(
            !participants.is_empty(),
            "participant set must not be empty (quorum math panics on n == 0)"
        );
        assert_eq!(
            participants.len(),
            g1_keys.len(),
            "g1_keys must be index-aligned with participants"
        );
    }

    /// Maps a certificate's signer bitmap to the signers' G1 public keys
    /// (participant order). The submitter aggregates these into the `apk1`
    /// implied by the on-chain non-signer subtraction.
    ///
    /// Out-of-range bits are skipped; only call with a bitmap from a
    /// certificate that passed [`Scheme::verify_certificate`] (which enforces
    /// `signers.len() == participants.len()`).
    pub fn signer_g1_points(&self, signers: &Signers) -> Vec<G1PublicKey> {
        signers
            .iter()
            .filter_map(|participant| self.g1_keys.get(usize::from(participant)).cloned())
            .collect()
    }

    /// Maps a certificate's signer bitmap to the signers' G2 public keys
    /// (participant order). Same caveats as
    /// [`JitoBn254Scheme::signer_g1_points`].
    pub fn signer_g2_keys(&self, signers: &Signers) -> Vec<PublicKey> {
        signers
            .iter()
            .filter_map(|participant| self.participants.key(participant).cloned())
            .collect()
    }
}

impl Scheme for JitoBn254Scheme {
    type Subject<'a, D: Digest> = &'a Item<D>;
    type PublicKey = PublicKey;
    type Signature = Signature;
    type Certificate = JitoCertificate;

    fn me(&self) -> Option<Participant> {
        self.signer.as_ref().map(|(index, _)| *index)
    }

    fn participants(&self) -> &Set<PublicKey> {
        &self.participants
    }

    /// Signs `Sha256Normalized(subject.digest) * sk` — the raw digest ONLY.
    ///
    /// The subject's height and the derived `_AGG_ACK` namespace are
    /// deliberately ignored; see the module docs for why this is required for
    /// on-chain verification and why the digest-only replay domain is
    /// acceptable.
    fn sign<D: Digest>(&self, subject: Self::Subject<'_, D>) -> Option<Attestation<Self>> {
        let (index, signer) = self.signer.as_ref()?;
        let digest = item_digest(subject)?;
        let signature = signer.sign_digest(&digest);
        Some(Attestation {
            signer: *index,
            signature: signature.into(),
        })
    }

    fn verify_attestation<R, D>(
        &self,
        _rng: &mut R,
        subject: Self::Subject<'_, D>,
        attestation: &Attestation<Self>,
        _strategy: &impl Strategy,
    ) -> bool
    where
        R: CryptoRngCore,
        D: Digest,
    {
        let Some(public_key) = self.participants.key(attestation.signer) else {
            return false;
        };
        // Lazy decode of untrusted bytes: `None` means malformed — reject,
        // never panic.
        let Some(signature) = attestation.signature.get() else {
            return false;
        };
        let Some(digest) = item_digest(subject) else {
            return false;
        };
        public_key.verify_digest(&digest, signature)
    }

    fn assemble<I, M>(
        &self,
        attestations: I,
        _strategy: &impl Strategy,
    ) -> Option<Self::Certificate>
    where
        I: IntoIterator<Item = Attestation<Self>>,
        I::IntoIter: Send,
        M: Faults,
    {
        // Collect the signers and signatures. Out-of-range indices MUST abort
        // before `Signers::from` (which panics on them); duplicate signers are
        // documented UB for `assemble` — the engine dedupes by participant
        // index before calling.
        let mut entries = Vec::new();
        for Attestation { signer, signature } in attestations {
            if usize::from(signer) >= self.participants.len() {
                return None;
            }
            let signature = signature.get().cloned()?;
            entries.push((signer, signature));
        }
        if entries.len() < self.participants.quorum::<M>() as usize {
            return None;
        }

        // Aggregate signature (point addition) + aggregate G2 of the signers —
        // both halves of the on-chain `VerifyCertificate` wire form.
        let (signers, signatures): (Vec<_>, Vec<_>) = entries.into_iter().unzip();
        let g2_keys: Vec<PublicKey> = signers
            .iter()
            .filter_map(|signer| self.participants.key(*signer).cloned())
            .collect();
        let signers = Signers::from(self.participants.len(), signers);
        let signature = aggregate_signatures(&signatures)?;
        let agg_g2_point = aggregate_g2_keys(&g2_keys)?;
        let agg_g2 = G2CompressedPoint::try_from(&agg_g2_point).ok()?;
        let agg_g2 = PublicKey::try_from(agg_g2.0.as_slice()).ok()?;

        Some(JitoCertificate {
            signers,
            signature,
            agg_g2,
        })
    }

    fn verify_certificate<R, D, M>(
        &self,
        _rng: &mut R,
        subject: Self::Subject<'_, D>,
        certificate: &Self::Certificate,
        _strategy: &impl Strategy,
    ) -> bool
    where
        R: CryptoRngCore,
        D: Digest,
        M: Faults,
    {
        // The decoded bitmap length is only upper-bounded by the codec config;
        // enforce the exact participant-set size here (a truncated or extended
        // bitmap would otherwise shift signer indices).
        if certificate.signers.len() != self.participants.len() {
            return false;
        }

        // Enforce the quorum under the caller's fault model (the aggregation
        // engine always passes N3f1: quorum = n - (n-1)/3).
        if certificate.signers.count() < self.participants.quorum::<M>() as usize {
            return false;
        }

        let Some(digest) = item_digest(subject) else {
            return false;
        };

        // Recompute apk1 from the bitmap (the point the on-chain non-signer
        // subtraction produces) and run the NCN program's exact verification:
        // one challenge-combined pairing over the certificate gamma.
        let signer_g1 = self.signer_g1_points(&certificate.signers);
        if signer_g1.len() != certificate.signers.count() {
            return false;
        }
        let Some(apk1) = aggregate_g1_keys(&signer_g1) else {
            return false;
        };
        aggregate_verify(
            &certificate.agg_g2.g2_point(),
            &apk1,
            &digest,
            &certificate.signature,
        )
    }

    fn is_attributable() -> bool {
        // Certificates carry the signer bitmap alongside the aggregate
        // signature, and individual G1 signatures are independently
        // verifiable.
        true
    }

    fn is_batchable() -> bool {
        // Every certificate verification is an independent pairing over a
        // different digest; eager per-certificate verification is preferred.
        false
    }

    fn certificate_codec_config(&self) -> <Self::Certificate as Read>::Cfg {
        self.participants.len()
    }

    fn certificate_codec_config_unbounded() -> <Self::Certificate as Read>::Cfg {
        u32::MAX as usize
    }
}

/// Certificate formed by the aggregated BN254 G1 signature, the aggregated G2
/// key of the signers, and the bitmap of signers that contributed — the exact
/// material `VerifyCertificate` consumes (INTERFACES.md §1: `agg_sig` G1
/// compressed 32B, `agg_g2` compressed 64B, bitmap).
///
/// Codec: `signers` (varint-length bitmap) followed by the fixed 32-byte
/// signature and the fixed 64-byte aggregate G2; `Read::Cfg` is the maximum
/// participant count (upper bound only — exact sizing is enforced by
/// [`Scheme::verify_certificate`]). The ON-CHAIN bitmap (LSB-first per
/// on-chain operator index) is derived from `signers` at submission time; the
/// codec here is chassis-internal.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JitoCertificate {
    /// Bitmap of participant indices that contributed signatures.
    pub signers: Signers,
    /// Aggregated (point-added) G1 signature covering all contributed
    /// signatures.
    pub signature: Signature,
    /// Aggregated (point-added) G2 public key of the contributing signers.
    pub agg_g2: PublicKey,
}

impl JitoCertificate {
    /// The certificate's aggregate G2 as an uncompressed `ncn-program-core`
    /// point (for pairings and instruction data).
    pub fn agg_g2_point(&self) -> G2Point {
        self.agg_g2.g2_point()
    }
}

impl Write for JitoCertificate {
    fn write(&self, writer: &mut impl BufMut) {
        self.signers.write(writer);
        self.signature.write(writer);
        self.agg_g2.write(writer);
    }
}

impl EncodeSize for JitoCertificate {
    fn encode_size(&self) -> usize {
        self.signers.encode_size() + self.signature.encode_size() + self.agg_g2.encode_size()
    }
}

impl Read for JitoCertificate {
    type Cfg = usize;

    fn read_cfg(reader: &mut impl Buf, max_participants: &usize) -> Result<Self, Error> {
        let signers = Signers::read_cfg(reader, max_participants)?;
        if signers.count() == 0 {
            return Err(Error::Invalid(
                "jito::JitoCertificate",
                "certificate contains no signers",
            ));
        }
        // Decode (and validate the points) eagerly: certificates are rare and
        // small, and this rejects malformed bytes at the decode boundary.
        let signature = Signature::read(reader)?;
        let agg_g2 = PublicKey::read(reader)?;
        Ok(Self {
            signers,
            signature,
            agg_g2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bn254::aggregate_signatures;
    use commonware_codec::{Decode, Encode, FixedSize};
    use commonware_consensus::aggregation::types::{Ack, Certificate as AggCertificate};
    use commonware_consensus::types::{Epoch, Height};
    use commonware_cryptography::sha256::Digest as Sha256Digest;
    use commonware_cryptography::{Hasher as _, Sha256};
    use commonware_math::algebra::Random as _;
    use commonware_parallel::Sequential;
    use commonware_utils::{N3f1, TryCollect, test_rng};
    use ncn_program_core::g2_point::G2Point;
    use ncn_program_core::schemes::{MessageDigest, Sha256Normalized};

    /// Deterministic signer set with participant-ordered schemes.
    ///
    /// Returns `(signers, verifier)` where `signers[i]` is the scheme whose
    /// participant index is `i` (i.e. sorted by compressed G2 bytes).
    fn setup(n: usize) -> (Vec<JitoBn254Scheme>, JitoBn254Scheme) {
        let mut rng = test_rng();
        let keys: Vec<Bn254Signer> = (0..n).map(|_| Bn254Signer::random(&mut rng)).collect();
        let participants: Set<PublicKey> = keys
            .iter()
            .map(|k| k.public_key())
            .try_collect()
            .expect("no duplicate keys");
        // g1_keys must be index-aligned with the sorted participant set.
        let g1_keys: Vec<G1PublicKey> = participants
            .iter()
            .map(|pk| {
                keys.iter()
                    .find(|k| &k.public_key() == pk)
                    .expect("participant derives from keys")
                    .public_g1()
            })
            .collect();

        let mut schemes: Vec<JitoBn254Scheme> = keys
            .iter()
            .map(|k| {
                JitoBn254Scheme::signer(participants.clone(), g1_keys.clone(), k.private_key())
                    .expect("key is in participant set")
            })
            .collect();
        schemes.sort_by_key(|s| s.me().expect("signer scheme has an index"));
        let verifier = JitoBn254Scheme::verifier(participants, g1_keys);
        (schemes, verifier)
    }

    fn item(height: u64, payload: &[u8]) -> Item<Sha256Digest> {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        Item {
            height: Height::new(height),
            digest: hasher.finalize(),
        }
    }

    fn sign_all(
        schemes: &[JitoBn254Scheme],
        item: &Item<Sha256Digest>,
        count: usize,
    ) -> Vec<Attestation<JitoBn254Scheme>> {
        schemes
            .iter()
            .take(count)
            .map(|s| s.sign::<Sha256Digest>(item).expect("signer can sign"))
            .collect()
    }

    // (a) Attestation sign/verify round-trip, including wrong-digest and
    // re-attribution rejection.
    #[test]
    fn attestation_roundtrip() {
        let mut rng = test_rng();
        let (schemes, verifier) = setup(4);
        let subject = item(1, b"task payload");

        for scheme in &schemes {
            let attestation = scheme.sign::<Sha256Digest>(&subject).unwrap();
            assert_eq!(Some(attestation.signer), scheme.me());
            assert!(verifier.verify_attestation::<_, Sha256Digest>(
                &mut rng,
                &subject,
                &attestation,
                &Sequential,
            ));

            // Wrong digest rejected.
            let wrong = item(1, b"different payload");
            assert!(!verifier.verify_attestation::<_, Sha256Digest>(
                &mut rng,
                &wrong,
                &attestation,
                &Sequential,
            ));

            // Attributing the signature to another participant rejected.
            let mut reattributed = attestation.clone();
            reattributed.signer =
                Participant::new((attestation.signer.get() + 1) % schemes.len() as u32);
            assert!(!verifier.verify_attestation::<_, Sha256Digest>(
                &mut rng,
                &subject,
                &reattributed,
                &Sequential,
            ));

            // Out-of-range signer index rejected (not panicking).
            let mut out_of_range = attestation.clone();
            out_of_range.signer = Participant::new(999);
            assert!(!verifier.verify_attestation::<_, Sha256Digest>(
                &mut rng,
                &subject,
                &out_of_range,
                &Sequential,
            ));
        }
    }

    #[test]
    fn verifier_cannot_sign() {
        let (_, verifier) = setup(4);
        assert!(verifier.me().is_none());
        assert!(verifier.sign::<Sha256Digest>(&item(1, b"x")).is_none());
    }

    // The signature binds only the digest — the same digest at another height
    // produces the same attestation signature (see module docs).
    #[test]
    fn signature_is_height_independent() {
        let (schemes, _) = setup(4);
        let a = item(1, b"same payload");
        let b = item(2, b"same payload");
        let sig_a = schemes[0].sign::<Sha256Digest>(&a).unwrap();
        let sig_b = schemes[0].sign::<Sha256Digest>(&b).unwrap();
        assert_eq!(
            sig_a.signature.get().unwrap().as_ref(),
            sig_b.signature.get().unwrap().as_ref()
        );
    }

    // (b) Assemble below quorum returns None; at quorum the certificate
    // verifies.
    #[test]
    fn assemble_quorum_boundary_n4() {
        let mut rng = test_rng();
        let (schemes, verifier) = setup(4);
        let subject = item(3, b"n4 task");
        let quorum = verifier.participants().quorum::<N3f1>() as usize;
        assert_eq!(quorum, 3); // n=4 → f=1 → quorum=3

        // Below quorum: None.
        let below = sign_all(&schemes, &subject, quorum - 1);
        assert!(verifier.assemble::<_, N3f1>(below, &Sequential).is_none());

        // At quorum: Some, and verify_certificate accepts.
        let at = sign_all(&schemes, &subject, quorum);
        let certificate = verifier.assemble::<_, N3f1>(at, &Sequential).unwrap();
        assert_eq!(certificate.signers.count(), quorum);
        assert_eq!(certificate.signers.len(), 4);
        assert!(verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &certificate,
            &Sequential,
        ));

        // Wrong digest rejected.
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &item(3, b"other task"),
            &certificate,
            &Sequential,
        ));
    }

    #[test]
    fn assemble_quorum_boundary_n3() {
        let mut rng = test_rng();
        let (schemes, verifier) = setup(3);
        let subject = item(9, b"n3 task");
        let quorum = verifier.participants().quorum::<N3f1>() as usize;
        assert_eq!(quorum, 3); // n=3 → f=0 → quorum=3 (all signers)

        let below = sign_all(&schemes, &subject, 2);
        assert!(verifier.assemble::<_, N3f1>(below, &Sequential).is_none());

        let at = sign_all(&schemes, &subject, 3);
        let certificate = verifier.assemble::<_, N3f1>(at, &Sequential).unwrap();
        assert!(verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &certificate,
            &Sequential,
        ));
    }

    #[test]
    fn assemble_rejects_out_of_range_signer() {
        let (schemes, verifier) = setup(4);
        let subject = item(4, b"task");
        let mut attestations = sign_all(&schemes, &subject, 3);
        attestations[0].signer = Participant::new(999);
        // Must return None (not panic in Signers::from).
        assert!(
            verifier
                .assemble::<_, N3f1>(attestations, &Sequential)
                .is_none()
        );
    }

    // (c) Tampered bitmap / wrong participant count / swapped aggregate G2
    // rejected.
    #[test]
    fn tampered_certificates_rejected() {
        let mut rng = test_rng();
        let (schemes, verifier) = setup(4);
        let subject = item(5, b"tamper me");
        let attestations = sign_all(&schemes, &subject, 3);
        let certificate = verifier
            .assemble::<_, N3f1>(attestations, &Sequential)
            .unwrap();

        // Claiming an extra signer that did not sign fails the pairing check.
        let mut extra_signer: Vec<Participant> = certificate.signers.iter().collect();
        extra_signer.push(Participant::new(3));
        let tampered = JitoCertificate {
            signers: Signers::from(4, extra_signer),
            signature: certificate.signature.clone(),
            agg_g2: certificate.agg_g2.clone(),
        };
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &tampered,
            &Sequential,
        ));

        // Dropping a signer from the bitmap (below quorum) is rejected.
        let fewer: Vec<Participant> = certificate.signers.iter().take(2).collect();
        let below_quorum = JitoCertificate {
            signers: Signers::from(4, fewer),
            signature: certificate.signature.clone(),
            agg_g2: certificate.agg_g2.clone(),
        };
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &below_quorum,
            &Sequential,
        ));

        // Swapping which participants are claimed (same count) fails the
        // pairing.
        let swapped = JitoCertificate {
            signers: Signers::from(4, [1u32, 2, 3].map(Participant::new)),
            signature: certificate.signature.clone(),
            agg_g2: certificate.agg_g2.clone(),
        };
        assert_ne!(
            swapped.signers, certificate.signers,
            "test requires a different signer set"
        );
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &swapped,
            &Sequential,
        ));

        // Substituting a different (valid) aggregate G2 breaks the gamma
        // binding between apk1 and apkG2.
        let other_g2 = verifier
            .participants()
            .key(Participant::new(3))
            .unwrap()
            .clone();
        let wrong_g2 = JitoCertificate {
            signers: certificate.signers.clone(),
            signature: certificate.signature.clone(),
            agg_g2: other_g2,
        };
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &wrong_g2,
            &Sequential,
        ));

        // Bitmap sized for a different participant count is rejected outright.
        let signers: Vec<Participant> = certificate.signers.iter().collect();
        let oversized = JitoCertificate {
            signers: Signers::from(5, signers.clone()),
            signature: certificate.signature.clone(),
            agg_g2: certificate.agg_g2.clone(),
        };
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &oversized,
            &Sequential,
        ));
        let undersized = JitoCertificate {
            signers: Signers::from(3, signers),
            signature: certificate.signature,
            agg_g2: certificate.agg_g2,
        };
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &undersized,
            &Sequential,
        ));
    }

    // (d) Certificate codec round-trip with the bounded config.
    #[test]
    fn certificate_codec_roundtrip() {
        let (schemes, verifier) = setup(4);
        let subject = item(6, b"codec");
        let attestations = sign_all(&schemes, &subject, 3);
        let certificate = verifier
            .assemble::<_, N3f1>(attestations, &Sequential)
            .unwrap();

        let encoded = certificate.encode();
        let decoded =
            JitoCertificate::decode_cfg(encoded.clone(), &verifier.certificate_codec_config())
                .expect("decode certificate");
        assert_eq!(decoded, certificate);

        // The unbounded (journal) config also decodes it.
        let decoded_unbounded = JitoCertificate::decode_cfg(
            encoded.clone(),
            &JitoBn254Scheme::certificate_codec_config_unbounded(),
        )
        .expect("decode certificate unbounded");
        assert_eq!(decoded_unbounded, certificate);

        // A tighter bound than the actual bitmap is rejected.
        assert!(JitoCertificate::decode_cfg(encoded, &2).is_err());

        // Certificates with no signers are rejected at decode time.
        let empty = JitoCertificate {
            signers: Signers::from(4, std::iter::empty::<Participant>()),
            signature: certificate.signature.clone(),
            agg_g2: certificate.agg_g2.clone(),
        };
        assert!(JitoCertificate::decode_cfg(empty.encode(), &4).is_err());

        // A corrupted signature (0xff-filled bytes are not a decompressible
        // point) fails decode, not verification.
        let mut corrupt = certificate.encode_mut();
        let len = corrupt.len();
        corrupt[len - PublicKey::SIZE - Signature::SIZE..len - PublicKey::SIZE].fill(0xff);
        assert!(JitoCertificate::decode_cfg(corrupt.freeze(), &4).is_err());
    }

    // (e) ON-CHAIN PARITY ANCHOR. This is the on-chain compatibility
    // guarantee:
    // - the scheme's attestation signature bytes equal `PrivKey::sign` (via
    //   `Bn254Signer::sign_digest`) for a fixed key;
    // - the assembled certificate's aggregate equals `aggregate_signatures` of
    //   the individual signatures, and its aggregate G2 equals the point-sum
    //   of the signers' G2 keys;
    // - `ncn_program_core::g2_point::G2Point::verify_aggregated_signature::
    //   <Sha256Normalized>` — the EXACT function the NCN program runs inside
    //   `VerifyCertificate` — accepts (sigma, digest, apk1).
    // If this test breaks, certificates will no longer verify inside the
    // deployed NCN program. Do not "fix" it by changing the preimage.
    #[test]
    fn onchain_parity_anchor() {
        let (schemes, verifier) = setup(4);
        let mut hasher = Sha256::new();
        hasher.update(b"fixed on-chain task digest");
        let digest = hasher.finalize();
        let digest_bytes: [u8; 32] = digest.as_ref().try_into().unwrap();
        let subject = Item {
            height: Height::new(77),
            digest,
        };

        // 1. Attestation signature bytes == low-level digest signature bytes.
        for scheme in &schemes {
            let (_, signer) = scheme.signer.as_ref().unwrap();
            let attestation = scheme.sign::<Sha256Digest>(&subject).unwrap();
            let low_level = signer.sign_digest(&digest_bytes);
            assert_eq!(
                attestation.signature.get().unwrap().as_ref(),
                low_level.as_ref(),
                "scheme signature must be bit-identical to Bn254Signer::sign_digest"
            );
        }

        // 2. Assembled aggregates == point sums of the individual parts.
        let quorum_schemes = &schemes[..3];
        let attestations: Vec<_> = quorum_schemes
            .iter()
            .map(|s| s.sign::<Sha256Digest>(&subject).unwrap())
            .collect();
        let certificate = verifier
            .assemble::<_, N3f1>(attestations.clone(), &Sequential)
            .unwrap();

        let individual: Vec<Signature> = attestations
            .iter()
            .map(|a| a.signature.get().unwrap().clone())
            .collect();
        let expected_aggregate = aggregate_signatures(&individual).unwrap();
        assert_eq!(certificate.signature.as_ref(), expected_aggregate.as_ref());

        let participating_g2 = verifier.signer_g2_keys(&certificate.signers);
        let expected_g2 = aggregate_g2_keys(&participating_g2).unwrap();
        assert_eq!(certificate.agg_g2_point().0, expected_g2.0);

        // 3. The NCN program's own verification function accepts the triple —
        // exactly the pairing `VerifyCertificate` evaluates on-chain.
        let participating_g1 = verifier.signer_g1_points(&certificate.signers);
        let apk1 = aggregate_g1_keys(&participating_g1).unwrap();
        let program_side: G2Point = certificate.agg_g2_point();
        assert!(
            program_side
                .verify_aggregated_signature::<Sha256Normalized>(
                    certificate.signature.g1_point(),
                    &MessageDigest(digest_bytes),
                    apk1,
                )
                .is_ok(),
            "ncn-program-core must accept the assembled certificate"
        );
    }

    // (f) Engine-integration smoke test through the aggregation types. Proves
    // the blanket `aggregation::scheme::Scheme<D>` marker holds for
    // JitoBn254Scheme (the same entry points the engine uses).
    #[test]
    fn aggregation_engine_smoke() {
        let mut rng = test_rng();
        let (schemes, verifier) = setup(4);
        let subject = item(11, b"engine smoke");

        let acks: Vec<Ack<JitoBn254Scheme, Sha256Digest>> = schemes
            .iter()
            .map(|s| Ack::sign(s, Epoch::zero(), subject.clone()).expect("participant signs"))
            .collect();
        for ack in &acks {
            assert!(ack.verify(&mut rng, &verifier, &Sequential));
        }

        // Verifier-only schemes cannot produce acks.
        assert!(Ack::sign(&verifier, Epoch::zero(), subject.clone()).is_none());

        let certificate =
            AggCertificate::from_acks(&verifier, &acks, &Sequential).expect("quorum of acks");
        assert_eq!(certificate.item, subject);
        assert!(certificate.verify(&mut rng, &verifier, &Sequential));

        // Below-quorum ack sets do not form a certificate.
        assert!(AggCertificate::from_acks(&verifier, &acks[..2], &Sequential).is_none());
    }
}
