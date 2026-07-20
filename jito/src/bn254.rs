//! BN254 key/signature wrappers in the Jito NCN program's signature domain.
//!
//! These types mirror the shape of `commonware_avs_core::bn254` (so they satisfy
//! the same `commonware_cryptography` traits the chassis expects) but their byte
//! formats and cryptography are the NCN program's, NOT the EigenLayer chassis
//! scheme's:
//!
//! - Points are carried in Solana `alt_bn128` compressed form (big-endian, as
//!   stored in `NCNOperatorAccount` / `Snapshot`), not ark-serialize form.
//! - Hash-to-curve is `ncn-program-core`'s [`Sha256Normalized`]
//!   (`sha256(domain ‖ digest ‖ counter)`, reject-and-retry, x-candidate
//!   decompress) — the domain the deployed NCN program recomputes on-chain.
//! - All signing/verification calls into `ncn-program-core`; nothing is
//!   reimplemented here (INTERFACES.md §1/§5).
//!
//! The three signature domains in this workspace (chassis eigenlayer bn254,
//! bls12381 reference, this one) are mutually incompatible by design; a
//! certificate assembled with these types verifies inside the NCN program and
//! nowhere else.

use commonware_codec::{Error, FixedSize, Read, Write};
use commonware_cryptography::{
    Hasher as _, PublicKey as CPublicKey, Sha256, Signature as CSignature, Signer as CSigner,
    Verifier,
};
use commonware_math::algebra::Random;
use commonware_utils::{Array, Span, union_unique};
use ncn_program_core::g1_point::{G1CompressedPoint, G1Point};
use ncn_program_core::g2_point::{G2CompressedPoint, G2Point};
use ncn_program_core::privkey::PrivKey;
use ncn_program_core::schemes::{MessageDigest, Sha256Normalized};
use num::CheckedAdd as _;
use rand_core::CryptoRngCore;
use std::borrow::Cow;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// 32-byte digests are the only signable input (chassis digest-only rule).
pub const DIGEST_LENGTH: usize = 32;
/// Big-endian Fr scalar, as `ncn_program_core::privkey::PrivKey` stores it.
pub const PRIVATE_KEY_LENGTH: usize = 32;
/// Solana `alt_bn128` G1 compressed point (big-endian x with flag bits).
pub const G1_COMPRESSED_LENGTH: usize = 32;
/// Signatures are compressed G1 points.
pub const SIGNATURE_LENGTH: usize = G1_COMPRESSED_LENGTH;
/// Solana `alt_bn128` G2 compressed point.
pub const G2_COMPRESSED_LENGTH: usize = 64;
/// Operator identity keys are compressed G2 points.
pub const PUBLIC_KEY_LENGTH: usize = G2_COMPRESSED_LENGTH;

/// Derives the 32-byte digest signed for a `(namespace, message)` pair.
///
/// Mirrors the chassis `bn254` convention: an empty namespace with an exactly
/// 32-byte message IS the digest (the certificate path — nothing else may enter
/// the preimage or the on-chain verification breaks); anything else is hashed
/// with [`union_unique`] domain separation (the p2p handshake path).
fn digest_bytes(namespace: &[u8], message: &[u8]) -> [u8; DIGEST_LENGTH] {
    if namespace.is_empty() && message.len() == DIGEST_LENGTH {
        message.try_into().expect("length checked above")
    } else {
        let payload = if namespace.is_empty() {
            Cow::Borrowed(message)
        } else {
            Cow::Owned(union_unique(namespace, message))
        };
        let mut hasher = Sha256::new();
        hasher.update(payload.as_ref());
        hasher
            .finalize()
            .as_ref()
            .try_into()
            .expect("sha256 digest is 32 bytes")
    }
}

/// BN254 private key: a big-endian Fr scalar in the NCN program's `PrivKey`
/// byte convention.
#[derive(Clone, Eq, PartialEq)]
pub struct PrivateKey {
    raw: [u8; PRIVATE_KEY_LENGTH],
}

impl PrivateKey {
    /// Validates that `raw` is a canonical, non-zero Fr scalar (the exact rule
    /// `ncn-program-core` applies when deriving G2 from a `PrivKey`).
    fn validate(raw: &[u8; PRIVATE_KEY_LENGTH]) -> Result<(), Error> {
        if raw.iter().all(|b| *b == 0) {
            return Err(Error::Invalid("jito::bn254::PrivateKey", "zero scalar"));
        }
        // G2 derivation deserializes the (reversed) bytes as an ark Fr element;
        // out-of-range scalars fail there, so reuse it as the canonical check.
        G2CompressedPoint::try_from(&PrivKey(*raw))
            .map(|_| ())
            .map_err(|_| Error::Invalid("jito::bn254::PrivateKey", "invalid Fr scalar"))
    }

    /// The `ncn-program-core` key this wraps.
    pub fn inner(&self) -> PrivKey {
        PrivKey(self.raw)
    }
}

impl Span for PrivateKey {}
impl Array for PrivateKey {}

impl FixedSize for PrivateKey {
    const SIZE: usize = PRIVATE_KEY_LENGTH;
}

impl Write for PrivateKey {
    fn write(&self, buf: &mut impl bytes::BufMut) {
        self.raw.write(buf);
    }
}

impl Read for PrivateKey {
    type Cfg = ();

    fn read_cfg(buf: &mut impl bytes::Buf, _cfg: &()) -> Result<Self, Error> {
        let raw = <[u8; PRIVATE_KEY_LENGTH]>::read_cfg(buf, &())?;
        Self::validate(&raw)?;
        Ok(Self { raw })
    }
}

impl TryFrom<&[u8]> for PrivateKey {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let raw: [u8; PRIVATE_KEY_LENGTH] = value
            .try_into()
            .map_err(|_| Error::InvalidLength(value.len()))?;
        Self::validate(&raw)?;
        Ok(Self { raw })
    }
}

impl TryFrom<&Vec<u8>> for PrivateKey {
    type Error = Error;
    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl TryFrom<Vec<u8>> for PrivateKey {
    type Error = Error;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl Hash for PrivateKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl Ord for PrivateKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl PartialOrd for PrivateKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl AsRef<[u8]> for PrivateKey {
    fn as_ref(&self) -> &[u8] {
        &self.raw
    }
}

impl Deref for PrivateKey {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.raw
    }
}

impl Debug for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print the scalar.
        write!(f, "jito::bn254::PrivateKey(REDACTED)")
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "jito::bn254::PrivateKey(REDACTED)")
    }
}

/// BN254 signer in the NCN program's signature domain.
///
/// Holds the `ncn-program-core` private key plus the derived G2 identity key.
/// Doubles as the p2p identity (the commonware [`CSigner`] impl signs
/// namespace-tagged handshake transcripts through the same
/// digest-then-[`Sha256Normalized`] path, so only one signature domain exists).
#[derive(Clone)]
pub struct Bn254Signer {
    key: PrivateKey,
    public: PublicKey,
}

impl Bn254Signer {
    /// Creates a signer from a validated private key. Returns `None` when the
    /// G2/G1 derivation fails (cannot happen for a key that passed
    /// [`PrivateKey`] validation, but never panic on key material).
    pub fn new(key: PrivateKey) -> Option<Self> {
        let g2 = G2CompressedPoint::try_from(&key.inner()).ok()?;
        let public = PublicKey::try_from(g2.0.as_slice()).ok()?;
        Some(Self { key, public })
    }

    /// Signs a raw 32-byte digest: `Sha256Normalized::hash_to_curve(digest) * sk`
    /// via `ncn-program-core` — the exact signature the NCN program verifies.
    pub fn sign_digest(&self, digest: &[u8; DIGEST_LENGTH]) -> Signature {
        let point = self
            .key
            .inner()
            .sign::<Sha256Normalized>(&MessageDigest(*digest))
            // Sha256Normalized retries 255 counters, each succeeding with
            // probability ~1/2; failure is cryptographically unreachable for a
            // fixed digest and a valid scalar.
            .expect("hash-to-curve exhausted 255 counters (probability ~2^-255)");
        Signature::from_uncompressed(point)
            .expect("signing a valid scalar cannot produce the identity point")
    }

    /// The signer's G1 public key (compressed), as registered on-chain.
    pub fn public_g1(&self) -> G1PublicKey {
        let g1 = G1CompressedPoint::try_from(self.key.inner())
            .expect("valid scalar always derives a G1 key");
        G1PublicKey::try_from(g1.0.as_slice()).expect("derived G1 key is valid and non-identity")
    }

    /// The wrapped private key.
    pub fn private_key(&self) -> PrivateKey {
        self.key.clone()
    }
}

impl Debug for Bn254Signer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bn254Signer")
            .field("public", &self.public)
            .finish_non_exhaustive()
    }
}

impl Random for Bn254Signer {
    fn random(mut rng: impl CryptoRngCore) -> Self {
        use ark_ff::UniformRand;
        use ark_serialize::CanonicalSerialize;
        loop {
            // Sample uniformly below the group order Fr (never Fq — see
            // ncn-program-core's PrivKey::from_random rationale), driven by the
            // caller's rng rather than thread_rng.
            let scalar = ark_bn254::Fr::rand(&mut rng);
            let mut le = [0u8; PRIVATE_KEY_LENGTH];
            scalar
                .serialize_compressed(&mut le[..])
                .expect("Fr compressed encoding is exactly 32 bytes");
            le.reverse(); // ark is little-endian; PrivKey is big-endian.
            if let Ok(key) = PrivateKey::try_from(le.as_slice())
                && let Some(signer) = Bn254Signer::new(key)
            {
                return signer;
            }
        }
    }
}

impl CSigner for Bn254Signer {
    type Signature = Signature;
    type PublicKey = PublicKey;

    fn public_key(&self) -> PublicKey {
        self.public.clone()
    }

    fn sign(&self, namespace: &[u8], message: &[u8]) -> Signature {
        self.sign_digest(&digest_bytes(namespace, message))
    }
}

/// Operator identity key: a compressed G2 point in the NCN program's on-chain
/// byte format (the `g2_pubkey` field of `NCNOperatorAccount`).
#[derive(Clone)]
pub struct PublicKey {
    raw: [u8; PUBLIC_KEY_LENGTH],
    /// Cached uncompressed form for pairing checks.
    point: G2Point,
}

impl PublicKey {
    fn try_from_raw(raw: [u8; PUBLIC_KEY_LENGTH]) -> Result<Self, Error> {
        if raw.iter().all(|b| *b == 0) {
            // The identity acts as a "free pass" under aggregate verification.
            return Err(Error::Invalid(
                "jito::bn254::PublicKey",
                "identity G2 point",
            ));
        }
        let point = G2Point::try_from(G2CompressedPoint(raw))
            .map_err(|_| Error::Invalid("jito::bn254::PublicKey", "invalid G2 point"))?;
        Ok(Self { raw, point })
    }

    /// The uncompressed G2 point (128 bytes), for `ncn-program-core` pairings.
    pub fn g2_point(&self) -> G2Point {
        self.point
    }

    /// The compressed on-chain wire form (64 bytes).
    pub fn compressed(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.raw
    }

    /// Verifies a signature over a raw 32-byte digest — the single-pairing check
    /// the NCN program applies per operator signature.
    pub fn verify_digest(&self, digest: &[u8; DIGEST_LENGTH], signature: &Signature) -> bool {
        self.point
            .verify_signature::<Sha256Normalized, G1Point>(
                signature.g1_point(),
                &MessageDigest(*digest),
            )
            .is_ok()
    }
}

impl Eq for PublicKey {}

impl PartialEq for PublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl Span for PublicKey {}
impl Array for PublicKey {}

impl FixedSize for PublicKey {
    const SIZE: usize = PUBLIC_KEY_LENGTH;
}

impl Write for PublicKey {
    fn write(&self, buf: &mut impl bytes::BufMut) {
        self.raw.write(buf);
    }
}

impl Read for PublicKey {
    type Cfg = ();

    fn read_cfg(buf: &mut impl bytes::Buf, _cfg: &()) -> Result<Self, Error> {
        let raw = <[u8; PUBLIC_KEY_LENGTH]>::read_cfg(buf, &())?;
        Self::try_from_raw(raw)
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let raw: [u8; PUBLIC_KEY_LENGTH] = value
            .try_into()
            .map_err(|_| Error::InvalidLength(value.len()))?;
        Self::try_from_raw(raw)
    }
}

impl TryFrom<&Vec<u8>> for PublicKey {
    type Error = Error;
    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl TryFrom<Vec<u8>> for PublicKey {
    type Error = Error;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl Hash for PublicKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl Ord for PublicKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.raw
    }
}

impl Deref for PublicKey {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.raw
    }
}

impl Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.raw))
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.raw))
    }
}

impl Verifier for PublicKey {
    type Signature = Signature;

    fn verify(&self, namespace: &[u8], message: &[u8], signature: &Signature) -> bool {
        self.verify_digest(&digest_bytes(namespace, message), signature)
    }
}

impl CPublicKey for PublicKey {}

/// An operator's G1 public key in the NCN program's compressed on-chain format
/// (the `g1_pubkey` field of `NCNOperatorAccount`; what APK arithmetic uses).
#[derive(Clone)]
pub struct G1PublicKey {
    raw: [u8; G1_COMPRESSED_LENGTH],
    point: G1Point,
}

impl G1PublicKey {
    fn try_from_raw(raw: [u8; G1_COMPRESSED_LENGTH]) -> Result<Self, Error> {
        if raw.iter().all(|b| *b == 0) {
            return Err(Error::Invalid(
                "jito::bn254::G1PublicKey",
                "identity G1 point",
            ));
        }
        let point = G1Point::try_from(&G1CompressedPoint(raw))
            .map_err(|_| Error::Invalid("jito::bn254::G1PublicKey", "invalid G1 point"))?;
        Ok(Self { raw, point })
    }

    /// The uncompressed G1 point (64 bytes) for `ncn-program-core` arithmetic.
    pub fn g1_point(&self) -> G1Point {
        self.point
    }

    /// The compressed on-chain wire form (32 bytes).
    pub fn compressed(&self) -> [u8; G1_COMPRESSED_LENGTH] {
        self.raw
    }
}

impl Eq for G1PublicKey {}

impl PartialEq for G1PublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl Span for G1PublicKey {}
impl Array for G1PublicKey {}

impl FixedSize for G1PublicKey {
    const SIZE: usize = G1_COMPRESSED_LENGTH;
}

impl Write for G1PublicKey {
    fn write(&self, buf: &mut impl bytes::BufMut) {
        self.raw.write(buf);
    }
}

impl Read for G1PublicKey {
    type Cfg = ();

    fn read_cfg(buf: &mut impl bytes::Buf, _cfg: &()) -> Result<Self, Error> {
        let raw = <[u8; G1_COMPRESSED_LENGTH]>::read_cfg(buf, &())?;
        Self::try_from_raw(raw)
    }
}

impl TryFrom<&[u8]> for G1PublicKey {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let raw: [u8; G1_COMPRESSED_LENGTH] = value
            .try_into()
            .map_err(|_| Error::InvalidLength(value.len()))?;
        Self::try_from_raw(raw)
    }
}

impl TryFrom<&Vec<u8>> for G1PublicKey {
    type Error = Error;
    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl TryFrom<Vec<u8>> for G1PublicKey {
    type Error = Error;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl Hash for G1PublicKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl Ord for G1PublicKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl PartialOrd for G1PublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl AsRef<[u8]> for G1PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.raw
    }
}

impl Deref for G1PublicKey {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.raw
    }
}

impl Debug for G1PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.raw))
    }
}

impl Display for G1PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.raw))
    }
}

/// A BLS signature: a compressed G1 point (32 bytes) in the NCN wire form —
/// exactly the `aggregated_signature` argument of `VerifyCertificate`.
#[derive(Clone)]
pub struct Signature {
    raw: [u8; SIGNATURE_LENGTH],
    point: G1Point,
}

impl Signature {
    fn try_from_raw(raw: [u8; SIGNATURE_LENGTH]) -> Result<Self, Error> {
        if raw.iter().all(|b| *b == 0) {
            // A zero signature pairs to the GT identity: reject at the decode
            // boundary, mirroring the chassis bn254 types.
            return Err(Error::Invalid(
                "jito::bn254::Signature",
                "identity G1 point",
            ));
        }
        let point = G1Point::try_from(&G1CompressedPoint(raw))
            .map_err(|_| Error::Invalid("jito::bn254::Signature", "invalid G1 point"))?;
        Ok(Self { raw, point })
    }

    /// Wraps an uncompressed `ncn-program-core` point, compressing it to the
    /// wire form. Returns `None` for the identity point.
    pub fn from_uncompressed(point: G1Point) -> Option<Self> {
        let compressed = G1CompressedPoint::try_from(point).ok()?;
        if compressed.0.iter().all(|b| *b == 0) {
            return None;
        }
        Some(Self {
            raw: compressed.0,
            point,
        })
    }

    /// The uncompressed G1 point (64 bytes) for `ncn-program-core` arithmetic.
    pub fn g1_point(&self) -> G1Point {
        self.point
    }

    /// The compressed on-chain wire form (32 bytes).
    pub fn compressed(&self) -> [u8; SIGNATURE_LENGTH] {
        self.raw
    }
}

impl Eq for Signature {}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl Span for Signature {}
impl Array for Signature {}

impl FixedSize for Signature {
    const SIZE: usize = SIGNATURE_LENGTH;
}

impl Write for Signature {
    fn write(&self, buf: &mut impl bytes::BufMut) {
        self.raw.write(buf);
    }
}

impl Read for Signature {
    type Cfg = ();

    fn read_cfg(buf: &mut impl bytes::Buf, _cfg: &()) -> Result<Self, Error> {
        let raw = <[u8; SIGNATURE_LENGTH]>::read_cfg(buf, &())?;
        Self::try_from_raw(raw)
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let raw: [u8; SIGNATURE_LENGTH] = value
            .try_into()
            .map_err(|_| Error::InvalidLength(value.len()))?;
        Self::try_from_raw(raw)
    }
}

impl TryFrom<&Vec<u8>> for Signature {
    type Error = Error;
    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl TryFrom<Vec<u8>> for Signature {
    type Error = Error;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl Hash for Signature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl Ord for Signature {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl PartialOrd for Signature {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.raw
    }
}

impl Deref for Signature {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.raw
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.raw))
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.raw))
    }
}

impl CSignature for Signature {}

/// Point-adds signatures via `ncn-program-core` G1 arithmetic. Returns `None`
/// on an empty set or if the sum degenerates to the identity.
pub fn aggregate_signatures(signatures: &[Signature]) -> Option<Signature> {
    let mut iter = signatures.iter();
    let mut acc = iter.next()?.g1_point();
    for signature in iter {
        acc = acc.checked_add(&signature.g1_point())?;
    }
    Signature::from_uncompressed(acc)
}

/// Point-adds G1 public keys — the aggregate `apk1` the verification pairing
/// consumes (on-chain this is the snapshot APK minus non-signers; host-side it
/// is the sum over the signer bitmap). Returns `None` on empty input or an
/// identity sum.
pub fn aggregate_g1_keys(keys: &[G1PublicKey]) -> Option<G1Point> {
    let mut iter = keys.iter();
    let mut acc = iter.next()?.g1_point();
    for key in iter {
        acc = acc.checked_add(&key.g1_point())?;
    }
    Some(acc)
}

/// Point-adds G2 identity keys — the certificate's `aggregated_g2` argument.
/// Returns `None` on empty input or if the sum cannot be represented.
pub fn aggregate_g2_keys(keys: &[PublicKey]) -> Option<G2Point> {
    let mut iter = keys.iter();
    let mut acc = iter.next()?.g2_point();
    for key in iter {
        acc = acc.checked_add(&key.g2_point())?;
    }
    Some(acc)
}

/// Verifies an aggregated certificate exactly the way the NCN program does:
/// one challenge-combined pairing over EigenLayer's gamma
/// (`ncn_program_core::g2_point::G2Point::verify_aggregated_signature`).
///
/// `apk_g2` is the certificate's aggregate G2, `apk1` the aggregate of the
/// SIGNERS' G1 keys, `signature` the aggregate signature.
pub fn aggregate_verify(
    apk_g2: &G2Point,
    apk1: &G1Point,
    digest: &[u8; DIGEST_LENGTH],
    signature: &Signature,
) -> bool {
    apk_g2
        .verify_aggregated_signature::<Sha256Normalized>(
            signature.g1_point(),
            &MessageDigest(*digest),
            *apk1,
        )
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_codec::{DecodeExt, Encode};
    use commonware_utils::test_rng;

    fn digest(payload: &[u8]) -> [u8; DIGEST_LENGTH] {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        hasher.finalize().as_ref().try_into().unwrap()
    }

    #[test]
    fn sign_verify_roundtrip_real_crypto() {
        let mut rng = test_rng();
        let signer = Bn254Signer::random(&mut rng);
        let d = digest(b"task digest");
        let signature = signer.sign_digest(&d);
        assert!(signer.public_key().verify_digest(&d, &signature));
        assert!(
            !signer
                .public_key()
                .verify_digest(&digest(b"other"), &signature)
        );

        // A different key must not verify.
        let other = Bn254Signer::random(&mut rng);
        assert!(!other.public_key().verify_digest(&d, &signature));
    }

    #[test]
    fn namespaced_signing_is_domain_separated() {
        let mut rng = test_rng();
        let signer = Bn254Signer::random(&mut rng);
        let namespaced = signer.sign(b"_HANDSHAKE", b"payload");
        let raw = signer.sign(b"", b"payload");
        assert_ne!(namespaced, raw);
        assert!(
            signer
                .public_key()
                .verify(b"_HANDSHAKE", b"payload", &namespaced)
        );
        assert!(!signer.public_key().verify(b"", b"payload", &namespaced));
        assert!(
            !signer
                .public_key()
                .verify(b"_OTHER", b"payload", &namespaced)
        );
    }

    #[test]
    fn empty_namespace_32_byte_message_is_the_digest() {
        // The certificate path: signing the digest through the namespace API
        // must be bit-identical to sign_digest (nothing else may enter the
        // preimage, or on-chain verification breaks).
        let mut rng = test_rng();
        let signer = Bn254Signer::random(&mut rng);
        let d = digest(b"certificate digest");
        assert_eq!(signer.sign(b"", &d), signer.sign_digest(&d));
    }

    #[test]
    fn aggregate_verify_matches_program_core() {
        let mut rng = test_rng();
        let signers: Vec<Bn254Signer> = (0..4).map(|_| Bn254Signer::random(&mut rng)).collect();
        let d = digest(b"aggregate me");

        let signatures: Vec<Signature> = signers.iter().map(|s| s.sign_digest(&d)).collect();
        let aggregate = aggregate_signatures(&signatures).expect("non-empty aggregate");

        let g1_keys: Vec<G1PublicKey> = signers.iter().map(|s| s.public_g1()).collect();
        let g2_keys: Vec<PublicKey> = signers.iter().map(|s| s.public_key()).collect();
        let apk1 = aggregate_g1_keys(&g1_keys).expect("non-empty apk");
        let apk_g2 = aggregate_g2_keys(&g2_keys).expect("non-empty apk g2");

        assert!(aggregate_verify(&apk_g2, &apk1, &d, &aggregate));
        // Wrong digest fails.
        assert!(!aggregate_verify(
            &apk_g2,
            &apk1,
            &digest(b"not it"),
            &aggregate
        ));
        // Dropping a signer from apk1 fails the challenge pairing.
        let short_apk1 = aggregate_g1_keys(&g1_keys[..3]).unwrap();
        assert!(!aggregate_verify(&apk_g2, &short_apk1, &d, &aggregate));
        // Mismatched G2 aggregate fails.
        let short_g2 = aggregate_g2_keys(&g2_keys[..3]).unwrap();
        assert!(!aggregate_verify(&short_g2, &apk1, &d, &aggregate));
    }

    #[test]
    fn codec_roundtrips_and_rejects_garbage() {
        let mut rng = test_rng();
        let signer = Bn254Signer::random(&mut rng);
        let d = digest(b"codec");

        let public = signer.public_key();
        let decoded = PublicKey::decode(public.encode()).expect("decode public key");
        assert_eq!(decoded, public);

        let signature = signer.sign_digest(&d);
        let decoded = Signature::decode(signature.encode()).expect("decode signature");
        assert_eq!(decoded, signature);

        let g1 = signer.public_g1();
        let decoded = G1PublicKey::decode(g1.encode()).expect("decode g1 key");
        assert_eq!(decoded, g1);

        // Identity points rejected at the decode boundary.
        assert!(PublicKey::decode(bytes::Bytes::from_static(&[0u8; 64])).is_err());
        assert!(Signature::decode(bytes::Bytes::from_static(&[0u8; 32])).is_err());
        assert!(G1PublicKey::decode(bytes::Bytes::from_static(&[0u8; 32])).is_err());
        // 0xff-filled bytes are not decompressible points.
        assert!(PublicKey::decode(bytes::Bytes::from_static(&[0xffu8; 64])).is_err());
        assert!(Signature::decode(bytes::Bytes::from_static(&[0xffu8; 32])).is_err());
    }

    #[test]
    fn private_key_rejects_zero_and_oversized_scalars() {
        assert!(PrivateKey::try_from(vec![0u8; 32]).is_err());
        // Fr modulus itself (big-endian) is out of range.
        let fr_modulus: [u8; 32] = [
            0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81,
            0x58, 0x5d, 0x28, 0x33, 0xe8, 0x48, 0x79, 0xb9, 0x70, 0x91, 0x43, 0xe1, 0xf5, 0x93,
            0xf0, 0x00, 0x00, 0x01,
        ];
        assert!(PrivateKey::try_from(fr_modulus.as_slice()).is_err());
        // One below the modulus is valid.
        let mut below = fr_modulus;
        below[31] = 0x00;
        assert!(PrivateKey::try_from(below.as_slice()).is_ok());
    }

    #[test]
    fn signer_key_material_roundtrips_through_codec() {
        let mut rng = test_rng();
        let signer = Bn254Signer::random(&mut rng);
        let key = signer.private_key();
        let decoded = PrivateKey::decode(key.encode()).expect("decode private key");
        let restored = Bn254Signer::new(decoded).expect("restore signer");
        assert_eq!(restored.public_key(), signer.public_key());
        assert_eq!(restored.public_g1(), signer.public_g1());
    }
}
