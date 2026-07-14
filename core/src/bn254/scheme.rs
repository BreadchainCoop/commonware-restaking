//! BN254 multisig [`certificate::Scheme`](commonware_cryptography::certificate::Scheme)
//! for the `commonware-consensus` aggregation engine.
//!
//! [`Bn254Scheme`] mirrors the bls12381 multisig reference implementation
//! (`commonware_cryptography::certificate::bls12381_multisig`) with two deliberate
//! differences:
//!
//! 1. **Signing covers ONLY the raw 32-byte digest** — `map_to_curve(digest) * sk`
//!    with no namespace and no height (see [`Bn254::sign_digest`]). This is what
//!    keeps assembled certificates verifiable by deployed EigenLayer
//!    `BLSSignatureChecker` contracts, which recompute exactly
//!    `map_to_curve(task_digest)` on-chain.
//! 2. The G2 identity key doubles as the signing key (no `BiMap` indirection), and a
//!    parallel `Vec<G1PublicKey>` in participant order lets an aggregator translate a
//!    certificate's signer bitmap into the G1 points needed for the on-chain
//!    `NonSignerStakesAndSignature` submission.
//!
//! # Replay domain (READ BEFORE "FIXING")
//!
//! Because the signature binds only the digest — not the `Item`'s height, the epoch,
//! or any namespace — signatures for the same digest are interchangeable across
//! heights and deployments. The digest itself must bind the full task content, and
//! the consuming contract must enforce its own replay protection (e.g. strict
//! ordering of state-transition indices), so replaying an identical digest at
//! another height submits the same state transition, which the contract accepts at
//! most once. Do NOT add the height or a namespace to the preimage — that would
//! break on-chain verification.
//!
//! Rogue-key note: certificates aggregate signatures/keys by plain point addition;
//! this is safe only because operators prove possession of their BN254 keys at
//! EigenLayer registration time.

use super::{Bn254, G1PublicKey, PrivateKey, PublicKey, Signature, aggregate_signatures};
use bytes::{Buf, BufMut};
use commonware_codec::{EncodeSize, Error, Read, ReadExt, Write};
use commonware_consensus::aggregation::types::Item;
use commonware_cryptography::certificate::{Attestation, Scheme, Signers};
use commonware_cryptography::{Digest, Signer as _};
use commonware_parallel::Strategy;
use commonware_utils::ordered::{Quorum, Set};
use commonware_utils::{Faults, Participant};
use rand_core::CryptoRngCore;

/// Extracts the raw 32 digest bytes from an [`Item`]'s digest.
///
/// The scheme only supports 32-byte digests (`sha256::Digest` in this system):
/// `map_to_curve` consumes exactly 32 bytes and the deployed contracts hash 32-byte
/// task digests. Returns `None` for any other digest width instead of panicking so a
/// mis-instantiated scheme fails verification rather than the process.
fn item_digest<D: Digest>(item: &Item<D>) -> Option<[u8; 32]> {
    item.digest.as_ref().try_into().ok()
}

/// BN254 multisig certificate scheme over a fixed participant set.
///
/// Participant indices are positions in the `ordered::Set` of G2 public keys (sorted
/// by compressed bytes) — every participant and verifier MUST build the set from the
/// same operator list or attestations will be attributed to the wrong signer and
/// blocked by the engine (`PeerMismatch`).
#[derive(Clone, Debug)]
pub struct Bn254Scheme {
    /// Ordered G2 identity/signing keys; participant indices are positions here.
    participants: Set<PublicKey>,
    /// G1 public keys in the SAME order as `participants` (index-aligned).
    g1_keys: Vec<G1PublicKey>,
    /// Our participant index and signer; `None` for verifier-only instances.
    signer: Option<(Participant, Bn254)>,
}

impl Bn254Scheme {
    /// Creates a signing scheme instance.
    ///
    /// `g1_keys` must be index-aligned with `participants` (i.e. `g1_keys[i]` is the
    /// G1 key registered for `participants[i]`); build both from the same sorted
    /// operator list.
    ///
    /// Returns `None` if `private_key`'s G2 public key is not in the participant set
    /// (mirroring the bls12381 multisig reference).
    ///
    /// # Panics
    ///
    /// Panics if `g1_keys.len() != participants.len()` or the participant set is
    /// empty — construction-time configuration errors, never network input.
    pub fn signer(
        participants: Set<PublicKey>,
        g1_keys: Vec<G1PublicKey>,
        private_key: PrivateKey,
    ) -> Option<Self> {
        Self::validate_participants(&participants, &g1_keys);
        let signer = Bn254::new(private_key).ok()?;
        let index = participants.index(&signer.public_key())?;
        Some(Self {
            participants,
            g1_keys,
            signer: Some((index, signer)),
        })
    }

    /// Creates a verifier-only scheme instance (`me() == None`): validates acks,
    /// assembles and verifies certificates, but never signs. This is what an
    /// aggregator/router runs.
    ///
    /// # Panics
    ///
    /// Panics if `g1_keys.len() != participants.len()` or the participant set is
    /// empty — construction-time configuration errors, never network input.
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

    /// Maps a certificate's signer bitmap to the signers' G1 public keys (participant
    /// order). An aggregator feeds these to `BLSApkRegistry.pubkeyHashToOperator` to
    /// resolve operator addresses for the on-chain submission.
    ///
    /// Out-of-range bits are skipped; only call with a bitmap from a certificate that
    /// passed [`Scheme::verify_certificate`] (which enforces
    /// `signers.len() == participants.len()`).
    pub fn signer_g1_points(&self, signers: &Signers) -> Vec<G1PublicKey> {
        signers
            .iter()
            .filter_map(|participant| self.g1_keys.get(usize::from(participant)).cloned())
            .collect()
    }

    /// Maps a certificate's signer bitmap to the signers' G2 public keys (participant
    /// order). Same caveats as [`Bn254Scheme::signer_g1_points`].
    pub fn signer_g2_keys(&self, signers: &Signers) -> Vec<PublicKey> {
        signers
            .iter()
            .filter_map(|participant| self.participants.key(participant).cloned())
            .collect()
    }
}

impl Scheme for Bn254Scheme {
    type Subject<'a, D: Digest> = &'a Item<D>;
    type PublicKey = PublicKey;
    type Signature = Signature;
    type Certificate = Bn254Certificate;

    fn me(&self) -> Option<Participant> {
        self.signer.as_ref().map(|(index, _)| *index)
    }

    fn participants(&self) -> &Set<PublicKey> {
        &self.participants
    }

    /// Signs `map_to_curve(subject.digest) * sk` — the raw digest ONLY.
    ///
    /// The subject's height and the derived `_AGG_ACK` namespace are deliberately
    /// ignored; see the module docs for why this is required for on-chain
    /// verification and why the digest-only replay domain is acceptable.
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
        // Lazy decode of untrusted bytes: `None` means malformed — reject, never
        // panic.
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
        // Collect the signers and signatures. Out-of-range indices MUST abort before
        // `Signers::from` (which panics on them); duplicate signers are documented
        // UB for `assemble` — the engine dedupes by participant index before calling.
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

        // Produce the signer bitmap and the aggregated (point-added) signature.
        let (signers, signatures): (Vec<_>, Vec<_>) = entries.into_iter().unzip();
        let signers = Signers::from(self.participants.len(), signers);
        let signature = aggregate_signatures(&signatures)?;

        Some(Bn254Certificate { signers, signature })
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

        // Enforce the quorum under the caller's fault model (the aggregation engine
        // always passes N3f1: quorum = n - (n-1)/3).
        if certificate.signers.count() < self.participants.quorum::<M>() as usize {
            return false;
        }

        let Some(digest) = item_digest(subject) else {
            return false;
        };

        // Aggregate the bitmap signers' G2 keys and do ONE pairing check of the
        // aggregate signature against the raw digest.
        let mut publics = Vec::with_capacity(certificate.signers.count());
        for signer in certificate.signers.iter() {
            let Some(public_key) = self.participants.key(signer) else {
                return false;
            };
            publics.push(public_key.clone());
        }
        super::aggregate_verify(&publics, None, &digest, &certificate.signature)
    }

    fn is_attributable() -> bool {
        // Certificates carry the signer bitmap alongside the aggregate signature,
        // and individual G1 signatures are independently verifiable.
        true
    }

    fn is_batchable() -> bool {
        // Every certificate verification is an independent pairing over a different
        // digest; no batch verification is implemented, so eager per-certificate
        // verification is preferred (unlike the bls12381 reference).
        false
    }

    fn certificate_codec_config(&self) -> <Self::Certificate as Read>::Cfg {
        self.participants.len()
    }

    fn certificate_codec_config_unbounded() -> <Self::Certificate as Read>::Cfg {
        u32::MAX as usize
    }
}

/// Certificate formed by an aggregated BN254 G1 signature plus the bitmap of signers
/// that contributed to it.
///
/// Codec mirrors the bls12381 multisig reference: `signers` (varint-length bitmap)
/// followed by the fixed 32-byte signature; `Read::Cfg` is the maximum participant
/// count (upper bound only — exact sizing is enforced by
/// [`Scheme::verify_certificate`]).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bn254Certificate {
    /// Bitmap of participant indices that contributed signatures.
    pub signers: Signers,
    /// Aggregated (point-added) G1 signature covering all contributed signatures.
    pub signature: Signature,
}

impl Write for Bn254Certificate {
    fn write(&self, writer: &mut impl BufMut) {
        self.signers.write(writer);
        self.signature.write(writer);
    }
}

impl EncodeSize for Bn254Certificate {
    fn encode_size(&self) -> usize {
        self.signers.encode_size() + self.signature.encode_size()
    }
}

impl Read for Bn254Certificate {
    type Cfg = usize;

    fn read_cfg(reader: &mut impl Buf, max_participants: &usize) -> Result<Self, Error> {
        let signers = Signers::read_cfg(reader, max_participants)?;
        if signers.count() == 0 {
            return Err(Error::Invalid(
                "bn254::Bn254Certificate",
                "Certificate contains no signers",
            ));
        }

        // Unlike the reference's `Lazy` signature, decode (and validate the G1
        // point) eagerly: certificates are rare and small, and this rejects
        // malformed bytes at the decode boundary.
        let signature = Signature::read(reader)?;

        Ok(Self { signers, signature })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bn254::{aggregate_verify, get_signer};
    use commonware_codec::{Decode, Encode, FixedSize};
    use commonware_consensus::aggregation::types::{Ack, Certificate as AggCertificate};
    use commonware_consensus::types::{Epoch, Height};
    use commonware_cryptography::sha256::Digest as Sha256Digest;
    use commonware_cryptography::{Hasher as _, Sha256};
    use commonware_parallel::Sequential;
    use commonware_utils::{N3f1, TryCollect, test_rng};

    /// Deterministic signer set with participant-ordered schemes.
    ///
    /// Returns `(signers, verifier)` where `signers[i]` is the scheme whose
    /// participant index is `i` (i.e. sorted by G2 compressed bytes).
    fn setup(n: usize) -> (Vec<Bn254Scheme>, Bn254Scheme) {
        let keys: Vec<Bn254> = (0..n).map(|i| Bn254::from_seed(i as u64)).collect();
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

        let mut schemes: Vec<Bn254Scheme> = keys
            .iter()
            .map(|k| {
                Bn254Scheme::signer(participants.clone(), g1_keys.clone(), k.private_key())
                    .expect("key is in participant set")
            })
            .collect();
        schemes.sort_by_key(|s| s.me().expect("signer scheme has an index"));
        let verifier = Bn254Scheme::verifier(participants, g1_keys);
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
        schemes: &[Bn254Scheme],
        item: &Item<Sha256Digest>,
        count: usize,
    ) -> Vec<Attestation<Bn254Scheme>> {
        schemes
            .iter()
            .take(count)
            .map(|s| s.sign::<Sha256Digest>(item).expect("signer can sign"))
            .collect()
    }

    // (a) Attestation sign/verify round-trip, including wrong-digest rejection.
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
    // produces the same attestation signature (see module docs for why this is
    // intentional and safe).
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

    // (b) Assemble below quorum returns None; at quorum the certificate verifies.
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

    // (c) Tampered bitmap / wrong participant count rejected.
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
        let tampered = Bn254Certificate {
            signers: Signers::from(4, extra_signer),
            signature: certificate.signature.clone(),
        };
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &tampered,
            &Sequential,
        ));

        // Dropping a signer from the bitmap (below quorum) is rejected.
        let fewer: Vec<Participant> = certificate.signers.iter().take(2).collect();
        let below_quorum = Bn254Certificate {
            signers: Signers::from(4, fewer),
            signature: certificate.signature.clone(),
        };
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &below_quorum,
            &Sequential,
        ));

        // Swapping which participants are claimed (same count) fails the pairing.
        let swapped = Bn254Certificate {
            signers: Signers::from(4, [1u32, 2, 3].map(Participant::new)),
            signature: certificate.signature.clone(),
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

        // Bitmap sized for a different participant count is rejected outright.
        let signers: Vec<Participant> = certificate.signers.iter().collect();
        let oversized = Bn254Certificate {
            signers: Signers::from(5, signers.clone()),
            signature: certificate.signature.clone(),
        };
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &oversized,
            &Sequential,
        ));
        let undersized = Bn254Certificate {
            signers: Signers::from(3, signers),
            signature: certificate.signature,
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
            Bn254Certificate::decode_cfg(encoded.clone(), &verifier.certificate_codec_config())
                .expect("decode certificate");
        assert_eq!(decoded, certificate);

        // The unbounded (journal) config also decodes it.
        let decoded_unbounded = Bn254Certificate::decode_cfg(
            encoded.clone(),
            &Bn254Scheme::certificate_codec_config_unbounded(),
        )
        .expect("decode certificate unbounded");
        assert_eq!(decoded_unbounded, certificate);

        // A tighter bound than the actual bitmap is rejected.
        assert!(Bn254Certificate::decode_cfg(encoded, &2).is_err());

        // Certificates with no signers are rejected at decode time.
        let empty = Bn254Certificate {
            signers: Signers::from(4, std::iter::empty::<Participant>()),
            signature: certificate.signature.clone(),
        };
        assert!(Bn254Certificate::decode_cfg(empty.encode(), &4).is_err());

        // A corrupted signature (not a decodable G1 point: 0xff-filled bytes encode
        // an out-of-range field element) fails decode, not verification.
        let mut corrupt = certificate.encode_mut();
        let len = corrupt.len();
        corrupt[len - Signature::SIZE..].fill(0xff);
        assert!(Bn254Certificate::decode_cfg(corrupt.freeze(), &4).is_err());
    }

    // (e) ON-CHAIN PARITY ANCHOR. This is the on-chain compatibility guarantee:
    // - the scheme's attestation signature bytes equal the low-level
    //   `sign_digest` bytes for a fixed key;
    // - the assembled certificate's aggregate equals `aggregate_signatures` of the
    //   individual signatures;
    // - `aggregate_verify(participating_g2, None, digest, aggregate)` holds.
    // If this test breaks, certificates will no longer verify inside deployed
    // `BLSSignatureChecker` contracts. Do not "fix" it by changing the preimage.
    #[test]
    fn onchain_parity_anchor() {
        let keys: Vec<Bn254> = ["101", "202", "303", "404"]
            .iter()
            .map(|k| get_signer(k))
            .collect();
        let participants: Set<PublicKey> = keys
            .iter()
            .map(|k| k.public_key())
            .try_collect()
            .expect("distinct keys");
        let g1_keys: Vec<G1PublicKey> = participants
            .iter()
            .map(|pk| {
                keys.iter()
                    .find(|k| &k.public_key() == pk)
                    .unwrap()
                    .public_g1()
            })
            .collect();

        let mut hasher = Sha256::new();
        hasher.update(b"fixed on-chain task digest");
        let digest = hasher.finalize();
        let digest_bytes: [u8; 32] = digest.as_ref().try_into().unwrap();
        let subject = Item {
            height: Height::new(77),
            digest,
        };

        // 1. Attestation signature bytes == low-level raw digest signature bytes.
        let mut schemes: Vec<Bn254Scheme> = keys
            .iter()
            .map(|k| {
                Bn254Scheme::signer(participants.clone(), g1_keys.clone(), k.private_key()).unwrap()
            })
            .collect();
        schemes.sort_by_key(|s| s.me().unwrap());
        for key in &keys {
            let scheme = schemes
                .iter()
                .find(|s| s.participants().key(s.me().unwrap()) == Some(&key.public_key()))
                .unwrap();
            let attestation = scheme.sign::<Sha256Digest>(&subject).unwrap();
            let low_level = key.sign_digest(&digest_bytes);
            assert_eq!(
                attestation.signature.get().unwrap().as_ref(),
                low_level.as_ref(),
                "scheme signature must be bit-identical to Bn254::sign_digest"
            );
        }

        // 2. Assembled aggregate == aggregate_signatures of the individual sigs.
        let quorum_schemes = &schemes[..3];
        let attestations: Vec<_> = quorum_schemes
            .iter()
            .map(|s| s.sign::<Sha256Digest>(&subject).unwrap())
            .collect();
        let certificate = schemes[0]
            .assemble::<_, N3f1>(attestations.clone(), &Sequential)
            .unwrap();

        let individual: Vec<Signature> = attestations
            .iter()
            .map(|a| a.signature.get().unwrap().clone())
            .collect();
        let expected_aggregate = aggregate_signatures(&individual).unwrap();
        assert_eq!(certificate.signature.as_ref(), expected_aggregate.as_ref());

        // 3. aggregate_verify over the participating G2 keys accepts it — exactly
        // the pairing the deployed contract evaluates.
        let verifier = Bn254Scheme::verifier(participants, g1_keys);
        let participating_g2 = verifier.signer_g2_keys(&certificate.signers);
        assert_eq!(participating_g2.len(), 3);
        assert!(aggregate_verify(
            &participating_g2,
            None,
            &digest_bytes,
            &certificate.signature
        ));

        // The G1 helper resolves the same participants (for the on-chain call).
        let participating_g1 = verifier.signer_g1_points(&certificate.signers);
        assert_eq!(participating_g1.len(), 3);
        for (g1, g2) in participating_g1.iter().zip(&participating_g2) {
            let key = keys.iter().find(|k| &k.public_key() == g2).unwrap();
            assert_eq!(g1, &key.public_g1());
        }
    }

    // (f) Engine-integration smoke test through the aggregation types. Proves the
    // blanket `aggregation::scheme::Scheme<D>` marker holds for Bn254Scheme (the
    // same entry points the engine uses: Ack::sign / Ack::verify /
    // Certificate::from_acks / Certificate::verify with M = N3f1).
    #[test]
    fn aggregation_engine_smoke() {
        let mut rng = test_rng();
        let (schemes, verifier) = setup(4);
        let subject = item(11, b"engine smoke");

        let acks: Vec<Ack<Bn254Scheme, Sha256Digest>> = schemes
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
