//! secp256k1 ECDSA multisig [`certificate::Scheme`](commonware_cryptography::certificate::Scheme)
//! for the `commonware-consensus` aggregation engine.
//!
//! [`EcdsaScheme`] mirrors the secp256r1 multisig reference implementation
//! (`commonware_cryptography::certificate::secp256r1`) with two deliberate
//! differences:
//!
//! 1. **Signing covers ONLY the raw 32-byte digest** — the digest is signed as an
//!    Ethereum prehash with no namespace and no height (see [`Ecdsa::sign_digest`]).
//!    This is what keeps assembled certificates verifiable by an on-chain verifier
//!    that recovers each signer with exactly `ecrecover(task_digest, v, r, s)`.
//! 2. The participant identity is the operator's 20-byte Ethereum address (the key
//!    EigenLayer registration is bound to), and verification is recovery-based —
//!    no separate signing public key needs distribution.
//!
//! # Replay domain (READ BEFORE "FIXING")
//!
//! Because the signature binds only the digest — not the `Item`'s height, the epoch,
//! or any namespace — signatures for the same digest are interchangeable across
//! heights and deployments. This matches the security model of the BN254 scheme in
//! [`crate::bn254`]: the digest itself must bind the full task content, and the
//! consuming contract must enforce its own replay protection (e.g. strict ordering
//! of state-transition indices), so replaying an identical digest at another height
//! submits the same state transition, which the contract accepts at most once. Do
//! NOT add the height or a namespace to the preimage — that would break on-chain
//! verification.
//!
//! Unlike BLS aggregation there is no rogue-key concern: every signature is verified
//! individually against its participant's address.

use super::{Ecdsa, PrivateKey, PublicKey, Signature};
use alloy_primitives::Address;
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
/// the on-chain verifier recovers signatures over 32-byte task digests. Returns
/// `None` for any other digest width instead of panicking so a mis-instantiated
/// scheme fails verification rather than the process.
fn item_digest<D: Digest>(item: &Item<D>) -> Option<[u8; 32]> {
    item.digest.as_ref().try_into().ok()
}

/// secp256k1 ECDSA multisig certificate scheme over a fixed participant set.
///
/// Participant indices are positions in the `ordered::Set` of operator addresses
/// (sorted by address bytes) — every node and the router MUST build the set from
/// the same operator list or attestations will be attributed to the wrong signer
/// and blocked by the engine (`PeerMismatch`).
#[derive(Clone, Debug)]
pub struct EcdsaScheme {
    /// Ordered operator addresses; participant indices are positions here.
    participants: Set<PublicKey>,
    /// Our participant index and signer; `None` for verifier-only instances.
    signer: Option<(Participant, Ecdsa)>,
}

impl EcdsaScheme {
    /// Creates a signing scheme instance.
    ///
    /// Returns `None` if `private_key`'s address is not in the participant set
    /// (mirroring the multisig reference implementations).
    ///
    /// # Panics
    ///
    /// Panics if the participant set is empty — a construction-time configuration
    /// error, never network input (quorum math panics on `n == 0`).
    pub fn signer(participants: Set<PublicKey>, private_key: PrivateKey) -> Option<Self> {
        Self::validate_participants(&participants);
        let signer = Ecdsa::new(private_key);
        let index = participants.index(&signer.public_key())?;
        Some(Self {
            participants,
            signer: Some((index, signer)),
        })
    }

    /// Creates a verifier-only scheme instance (`me() == None`): validates acks,
    /// assembles and verifies certificates, but never signs. This is what the router
    /// runs.
    ///
    /// # Panics
    ///
    /// Panics if the participant set is empty — a construction-time configuration
    /// error, never network input.
    pub fn verifier(participants: Set<PublicKey>) -> Self {
        Self::validate_participants(&participants);
        Self {
            participants,
            signer: None,
        }
    }

    fn validate_participants(participants: &Set<PublicKey>) {
        assert!(
            !participants.is_empty(),
            "participant set must not be empty (quorum math panics on n == 0)"
        );
    }

    /// Maps a certificate's signer bitmap to the signers' Ethereum addresses
    /// (participant order). The router pairs these with the certificate's
    /// signatures for the on-chain submission.
    ///
    /// Out-of-range bits are skipped; only call with a bitmap from a certificate
    /// that passed [`Scheme::verify_certificate`] (which enforces
    /// `signers.len() == participants.len()`).
    pub fn signer_addresses(&self, signers: &Signers) -> Vec<Address> {
        signers
            .iter()
            .filter_map(|participant| self.participants.key(participant).map(|k| k.address()))
            .collect()
    }
}

impl Scheme for EcdsaScheme {
    type Subject<'a, D: Digest> = &'a Item<D>;
    type PublicKey = PublicKey;
    type Signature = Signature;
    type Certificate = EcdsaCertificate;

    fn me(&self) -> Option<Participant> {
        self.signer.as_ref().map(|(index, _)| *index)
    }

    fn participants(&self) -> &Set<PublicKey> {
        &self.participants
    }

    /// Signs the raw digest ONLY (Ethereum prehash semantics).
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

        // Certificate signatures are stored in ascending participant-index order —
        // the same order the bitmap iterates — so verification (and the on-chain
        // submission) can zip bitmap indices with signatures positionally.
        entries.sort_by_key(|(signer, _)| *signer);
        let (signers, signatures): (Vec<_>, Vec<_>) = entries.into_iter().unzip();
        let signers = Signers::from(self.participants.len(), signers);

        Some(EcdsaCertificate {
            signers,
            signatures,
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

        // Every bitmap signer must have exactly one signature.
        if certificate.signers.count() != certificate.signatures.len() {
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

        // Verify each signature against its bitmap participant's address.
        for (signer, signature) in certificate.signers.iter().zip(&certificate.signatures) {
            let Some(public_key) = self.participants.key(signer) else {
                return false;
            };
            if !public_key.verify_digest(&digest, signature) {
                return false;
            }
        }
        true
    }

    fn is_attributable() -> bool {
        // Certificates carry the signer bitmap alongside per-signer signatures,
        // each independently verifiable.
        true
    }

    fn is_batchable() -> bool {
        // Recovery-based verification has no batch form; eager per-certificate
        // verification is preferred.
        false
    }

    fn certificate_codec_config(&self) -> <Self::Certificate as Read>::Cfg {
        self.participants.len()
    }

    fn certificate_codec_config_unbounded() -> <Self::Certificate as Read>::Cfg {
        u32::MAX as usize
    }
}

/// Certificate formed by the bitmap of contributing signers plus their individual
/// 65-byte ECDSA signatures in ascending participant-index order (the bitmap's
/// iteration order).
///
/// Codec: `signers` (varint-length bitmap) followed by exactly `signers.count()`
/// fixed 65-byte signatures; `Read::Cfg` is the maximum participant count (upper
/// bound only — exact sizing is enforced by [`Scheme::verify_certificate`]).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EcdsaCertificate {
    /// Bitmap of participant indices that contributed signatures.
    pub signers: Signers,
    /// Per-signer signatures, index-aligned with the bitmap's ascending iteration.
    pub signatures: Vec<Signature>,
}

impl Write for EcdsaCertificate {
    fn write(&self, writer: &mut impl BufMut) {
        self.signers.write(writer);
        for signature in &self.signatures {
            signature.write(writer);
        }
    }
}

impl EncodeSize for EcdsaCertificate {
    fn encode_size(&self) -> usize {
        self.signers.encode_size()
            + self
                .signatures
                .iter()
                .map(|signature| signature.encode_size())
                .sum::<usize>()
    }
}

impl Read for EcdsaCertificate {
    type Cfg = usize;

    fn read_cfg(reader: &mut impl Buf, max_participants: &usize) -> Result<Self, Error> {
        let signers = Signers::read_cfg(reader, max_participants)?;
        if signers.count() == 0 {
            return Err(Error::Invalid(
                "ecdsa::EcdsaCertificate",
                "Certificate contains no signers",
            ));
        }

        // Exactly one signature per bitmap signer. Decoding (and canonical-form
        // validation) is eager: certificates are rare and small, and this rejects
        // malformed bytes at the decode boundary.
        let mut signatures = Vec::with_capacity(signers.count());
        for _ in 0..signers.count() {
            signatures.push(Signature::read(reader)?);
        }

        Ok(Self {
            signers,
            signatures,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecdsa::get_signer;
    use commonware_codec::{Decode, Encode};
    use commonware_consensus::aggregation::types::{Ack, Certificate as AggCertificate};
    use commonware_consensus::types::{Epoch, Height};
    use commonware_cryptography::sha256::Digest as Sha256Digest;
    use commonware_cryptography::{Hasher as _, Sha256};
    use commonware_parallel::Sequential;
    use commonware_utils::{N3f1, TryCollect, test_rng};

    /// Deterministic signer set with participant-ordered schemes.
    ///
    /// Returns `(signers, verifier)` where `signers[i]` is the scheme whose
    /// participant index is `i` (i.e. sorted by address bytes).
    fn setup(n: usize) -> (Vec<EcdsaScheme>, EcdsaScheme) {
        let keys: Vec<Ecdsa> = (0..n).map(|i| Ecdsa::from_seed(i as u64)).collect();
        let participants: Set<PublicKey> = keys
            .iter()
            .map(|k| k.public_key())
            .try_collect()
            .expect("no duplicate keys");

        let mut schemes: Vec<EcdsaScheme> = keys
            .iter()
            .map(|k| {
                EcdsaScheme::signer(participants.clone(), k.private_key())
                    .expect("key is in participant set")
            })
            .collect();
        schemes.sort_by_key(|s| s.me().expect("signer scheme has an index"));
        let verifier = EcdsaScheme::verifier(participants);
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
        schemes: &[EcdsaScheme],
        item: &Item<Sha256Digest>,
        count: usize,
    ) -> Vec<Attestation<EcdsaScheme>> {
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
        assert_eq!(certificate.signatures.len(), quorum);
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

    // Assemble must order signatures by participant index regardless of the
    // order attestations arrive in.
    #[test]
    fn assemble_orders_signatures_by_participant_index() {
        let mut rng = test_rng();
        let (schemes, verifier) = setup(4);
        let subject = item(12, b"unordered attestations");

        let mut attestations = sign_all(&schemes, &subject, 3);
        attestations.reverse();
        let certificate = verifier
            .assemble::<_, N3f1>(attestations, &Sequential)
            .unwrap();
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

    // (c) Tampered bitmap / signature list / wrong participant count rejected.
    #[test]
    fn tampered_certificates_rejected() {
        let mut rng = test_rng();
        let (schemes, verifier) = setup(4);
        let subject = item(5, b"tamper me");
        let attestations = sign_all(&schemes, &subject, 3);
        let certificate = verifier
            .assemble::<_, N3f1>(attestations, &Sequential)
            .unwrap();

        // Claiming an extra signer that did not sign fails (count mismatch).
        let mut extra_signer: Vec<Participant> = certificate.signers.iter().collect();
        extra_signer.push(Participant::new(3));
        let tampered = EcdsaCertificate {
            signers: Signers::from(4, extra_signer),
            signatures: certificate.signatures.clone(),
        };
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &tampered,
            &Sequential,
        ));

        // Dropping a signer from the bitmap (below quorum) is rejected.
        let fewer: Vec<Participant> = certificate.signers.iter().take(2).collect();
        let below_quorum = EcdsaCertificate {
            signers: Signers::from(4, fewer),
            signatures: certificate.signatures[..2].to_vec(),
        };
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &below_quorum,
            &Sequential,
        ));

        // Swapping which participants are claimed (same count) fails recovery.
        let signers: Vec<Participant> = certificate.signers.iter().collect();
        let swapped_participants: Vec<Participant> = signers
            .iter()
            .map(|p| Participant::new((p.get() + 1) % 4))
            .collect();
        let swapped = EcdsaCertificate {
            signers: Signers::from(4, swapped_participants),
            signatures: certificate.signatures.clone(),
        };
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &swapped,
            &Sequential,
        ));

        // Reordering the signature list breaks positional attribution.
        let mut reordered = certificate.clone();
        reordered.signatures.swap(0, 1);
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &reordered,
            &Sequential,
        ));

        // Bitmap sized for a different participant count is rejected outright.
        let oversized = EcdsaCertificate {
            signers: Signers::from(5, signers.clone()),
            signatures: certificate.signatures.clone(),
        };
        assert!(!verifier.verify_certificate::<_, Sha256Digest, N3f1>(
            &mut rng,
            &subject,
            &oversized,
            &Sequential,
        ));
        let undersized = EcdsaCertificate {
            signers: Signers::from(3, signers),
            signatures: certificate.signatures,
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
            EcdsaCertificate::decode_cfg(encoded.clone(), &verifier.certificate_codec_config())
                .expect("decode certificate");
        assert_eq!(decoded, certificate);

        // The unbounded (journal) config also decodes it.
        let decoded_unbounded = EcdsaCertificate::decode_cfg(
            encoded.clone(),
            &EcdsaScheme::certificate_codec_config_unbounded(),
        )
        .expect("decode certificate unbounded");
        assert_eq!(decoded_unbounded, certificate);

        // A tighter bound than the actual bitmap is rejected.
        assert!(EcdsaCertificate::decode_cfg(encoded, &2).is_err());

        // Certificates with no signers are rejected at decode time.
        let empty = EcdsaCertificate {
            signers: Signers::from(4, std::iter::empty::<Participant>()),
            signatures: Vec::new(),
        };
        assert!(EcdsaCertificate::decode_cfg(empty.encode(), &4).is_err());

        // A corrupted signature (0xff-filled: invalid parity byte) fails decode,
        // not verification.
        let mut corrupt = certificate.encode_mut();
        let len = corrupt.len();
        corrupt[len - 65..].fill(0xff);
        assert!(EcdsaCertificate::decode_cfg(corrupt.freeze(), &4).is_err());

        // A truncated signature list fails decode.
        let mut truncated = certificate.encode_mut();
        let len = truncated.len();
        truncated.truncate(len - 65);
        assert!(EcdsaCertificate::decode_cfg(truncated.freeze(), &4).is_err());
    }

    // (e) ON-CHAIN PARITY ANCHOR. This is the on-chain compatibility guarantee:
    // - the scheme's attestation signature bytes equal the low-level
    //   `sign_digest` bytes for a fixed key (65-byte r||s||v, v in {27,28}, low-s);
    // - every certificate signature recovers to its bitmap participant's address
    //   via Ethereum's prehash recovery — exactly the `ecrecover(msgHash, v, r, s)`
    //   an on-chain verifier evaluates.
    // If this test breaks, certificates will no longer verify on-chain. Do not
    // "fix" it by changing the preimage.
    #[test]
    fn onchain_parity_anchor() {
        let keys: Vec<Ecdsa> = ["101", "202", "303", "404"]
            .iter()
            .map(|k| get_signer(k))
            .collect();
        let participants: Set<PublicKey> = keys
            .iter()
            .map(|k| k.public_key())
            .try_collect()
            .expect("distinct keys");

        let mut hasher = Sha256::new();
        hasher.update(b"fixed on-chain task digest");
        let digest = hasher.finalize();
        let digest_bytes: [u8; 32] = digest.as_ref().try_into().unwrap();
        let subject = Item {
            height: Height::new(77),
            digest,
        };

        // 1. Attestation signature bytes == low-level raw digest signature bytes.
        let mut schemes: Vec<EcdsaScheme> = keys
            .iter()
            .map(|k| EcdsaScheme::signer(participants.clone(), k.private_key()).unwrap())
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
                "scheme signature must be bit-identical to Ecdsa::sign_digest"
            );
        }

        // 2. Every certificate signature recovers to its participant's address —
        // exactly the ecrecover check an on-chain verifier evaluates.
        let quorum_schemes = &schemes[..3];
        let attestations: Vec<_> = quorum_schemes
            .iter()
            .map(|s| s.sign::<Sha256Digest>(&subject).unwrap())
            .collect();
        let certificate = schemes[0]
            .assemble::<_, N3f1>(attestations, &Sequential)
            .unwrap();

        let verifier = EcdsaScheme::verifier(participants.clone());
        let addresses = verifier.signer_addresses(&certificate.signers);
        assert_eq!(addresses.len(), 3);
        for (address, signature) in addresses.iter().zip(&certificate.signatures) {
            assert_eq!(
                signature.recover(&digest_bytes),
                Some(*address),
                "certificate signature must ecrecover to the bitmap participant"
            );
            let raw = signature.as_ref();
            assert!(raw[64] == 27 || raw[64] == 28, "v must be 27/28 on-chain");
        }

        // 3. The participant order (and thus the signature order) is ascending by
        // address — the strictly-ascending-signers order an on-chain verifier
        // typically enforces to reject duplicate signers.
        let mut sorted = addresses.clone();
        sorted.sort();
        assert_eq!(addresses, sorted);
    }

    // (f) Engine-integration smoke test through the aggregation types. Proves the
    // blanket `aggregation::scheme::Scheme<D>` marker holds for EcdsaScheme (the
    // same entry points the engine uses: Ack::sign / Ack::verify /
    // Certificate::from_acks / Certificate::verify with M = N3f1).
    #[test]
    fn aggregation_engine_smoke() {
        let mut rng = test_rng();
        let (schemes, verifier) = setup(4);
        let subject = item(11, b"engine smoke");

        let acks: Vec<Ack<EcdsaScheme, Sha256Digest>> = schemes
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
