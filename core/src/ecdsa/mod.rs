//! secp256k1 ECDSA key types and Ethereum-compatible signing.
//!
//! A second engine-compatible certificate scheme alongside [`crate::bn254`], for
//! deployments whose on-chain verifier checks signatures via Ethereum's
//! `ecrecover` rather than a BLS pairing. The cryptographic contract with the
//! chain is:
//!
//! - Signing produces a 65-byte `r || s || v` signature (low-s per EIP-2, `v` in
//!   Electrum notation 27/28) over a 32-byte digest used AS-IS (no EIP-191 prefix,
//!   no extra hashing). This is exactly the byte layout an on-chain verifier
//!   recovers with `ecrecover(msgHash, v, r, s)`.
//! - Verification is recovery-based: recover the signer address from the signature
//!   and compare it with the expected address. This is what lets a 20-byte
//!   Ethereum address serve as the [`PublicKey`] type — the same address the
//!   operator registered with EigenLayer.
//! - Namespace semantics (see [`signing_payload`]): a `None` namespace with an
//!   exactly 32-byte message means "the message IS the digest" and is signed raw —
//!   this is the on-chain path. Any other input is hashed (with `union_unique`
//!   domain separation when a namespace is present) — this is the path the
//!   2026.5.0 `Signer`/`Verifier` traits use (the p2p handshake always provides a
//!   namespace).
//!
//! Type surface:
//! - [`PrivateKey`]: the secp256k1 scalar (32 bytes, big-endian).
//! - [`Ecdsa`]: the signer (implements the 2026.5.0 [`commonware_cryptography::Signer`]
//!   trait, so it can be the p2p identity signer for `authenticated::lookup`).
//! - [`PublicKey`]: the signer's Ethereum address (20 bytes). The operator's
//!   registered EigenLayer address, the p2p identity, AND the participant identity
//!   key ordering the certificate scheme's participant set.
//! - [`Signature`]: 65 bytes `r || s || v` with `v` in {27, 28}. Decoding rejects
//!   non-canonical encodings (zero `r`/`s`, high-s, bad parity byte) so signature
//!   bytes are unique per (digest, key) and byte-comparison equality is sound.

pub mod scheme;

pub use scheme::{EcdsaCertificate, EcdsaScheme};

use alloy_primitives::{Address, B256, U256};
use alloy_signer::k256::ecdsa::SigningKey;
use alloy_signer::utils::secret_key_to_address;
use bytes::{Buf, BufMut};
use commonware_codec::{Error, FixedSize, Read, Write};
use commonware_cryptography::{
    Hasher as _, PublicKey as CPublicKey, Sha256, Signature as CSignature, Signer, Verifier,
};
use commonware_math::algebra::Random;
use commonware_utils::{Array, Span, union_unique};
use rand_core::CryptoRngCore;
use std::borrow::Cow;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// Formats raw key/signature bytes as lowercase hex — the shared `Debug`/`Display`
/// body for the (non-secret) types.
fn fmt_hex(raw: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "0x{}", hex::encode(raw))
}

/// Length of a signing digest in bytes.
const DIGEST_LENGTH: usize = 32;
/// Length of a [`PrivateKey`] (secp256k1 scalar, big-endian) in bytes.
const PRIVATE_KEY_LENGTH: usize = 32;
/// Length of a [`PublicKey`] (Ethereum address) in bytes.
const PUBLIC_KEY_LENGTH: usize = 20;
/// Length of a [`Signature`] (`r || s || v`) in bytes.
const SIGNATURE_LENGTH: usize = 65;

/// secp256k1 group order / 2
/// (`0x7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0`).
/// Signatures with `s` above this are malleable (EIP-2) and rejected at the
/// decode boundary.
const HALF_CURVE_ORDER: U256 = U256::from_limbs([
    0xdfe92f46681b20a0,
    0x5d576e7357a4501d,
    0xffffffffffffffff,
    0x7fffffffffffffff,
]);

/// Computes the 32-byte digest that gets signed/recovered.
///
/// INVARIANT (on-chain compatibility): `namespace == None` and a 32-byte message
/// means the message is a pre-hashed digest and is used as-is — this is the path
/// whose signatures `ecrecover` validates on-chain. Every other combination is
/// hashed: `sha256(union_unique(namespace, message))` when a namespace is present
/// (the p2p handshake path), `sha256(message)` otherwise.
fn signing_payload(namespace: Option<&[u8]>, message: &[u8]) -> [u8; DIGEST_LENGTH] {
    if namespace.is_none() && message.len() == DIGEST_LENGTH {
        return message.try_into().expect("length checked above");
    }
    let payload = match namespace {
        Some(namespace) => Cow::Owned(union_unique(namespace, message)),
        None => Cow::Borrowed(message),
    };
    let mut hasher = Sha256::new();
    hasher.update(payload.as_ref());
    let digest = hasher.finalize();
    digest
        .as_ref()
        .try_into()
        .expect("sha256 digest is 32 bytes")
}

/// A secp256k1 ECDSA signer: private scalar plus the derived Ethereum address.
///
/// Implements the 2026.5.0 [`Signer`] trait (namespaced path) so it can serve as the
/// p2p identity signer, and exposes [`Ecdsa::sign_digest`] for the raw on-chain path.
#[derive(Clone)]
pub struct Ecdsa {
    private: SigningKey,
    address: Address,
}

impl Ecdsa {
    /// Builds a signer from a [`PrivateKey`], deriving the Ethereum address.
    pub fn new(private_key: PrivateKey) -> Self {
        Self::from_signing_key(private_key.key)
    }

    /// Builds a signer directly from a `SigningKey`.
    fn from_signing_key(private: SigningKey) -> Self {
        let address = secret_key_to_address(&private);
        Ecdsa { private, address }
    }

    /// Returns the private key.
    pub fn private_key(&self) -> PrivateKey {
        PrivateKey::from(self.private.clone())
    }

    /// Returns the signer's Ethereum address.
    pub fn address(&self) -> Address {
        self.address
    }

    /// Signs a raw 32-byte digest, used AS-IS (no prefix, no extra hashing).
    ///
    /// Produces the canonical low-s signature with the recovery id Ethereum's
    /// `ecrecover` expects — byte-compatible with on-chain recovery-based
    /// verification.
    pub fn sign_digest(&self, digest: &[u8; DIGEST_LENGTH]) -> Signature {
        let (sig, recovery_id) = self
            .private
            .sign_prehash_recoverable(digest)
            .expect("signing over a 32-byte prehash cannot fail");
        Signature::from_alloy(alloy_primitives::Signature::from((sig, recovery_id)))
    }

    /// Signs with the old `Option<&[u8]>` namespace semantics (see
    /// [`signing_payload`]). The 2026.5.0 [`Signer`] trait delegates here with
    /// `Some(namespace)`; internal digest signing uses `None`.
    pub fn sign_message(&self, namespace: Option<&[u8]>, message: &[u8]) -> Signature {
        self.sign_digest(&signing_payload(namespace, message))
    }
}

impl Signer for Ecdsa {
    type Signature = Signature;
    type PublicKey = PublicKey;

    fn sign(&self, namespace: &[u8], message: &[u8]) -> Signature {
        self.sign_message(Some(namespace), message)
    }

    fn public_key(&self) -> PublicKey {
        PublicKey::from(self.address)
    }
}

impl Random for Ecdsa {
    fn random(mut rng: impl CryptoRngCore) -> Self {
        Self::from_signing_key(SigningKey::random(&mut rng))
    }
}

impl Debug for Ecdsa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print the private scalar.
        f.debug_struct("Ecdsa")
            .field("address", &self.address)
            .finish_non_exhaustive()
    }
}

/// A secp256k1 private key: the scalar plus its 32-byte big-endian encoding.
#[derive(Clone)]
pub struct PrivateKey {
    raw: [u8; PRIVATE_KEY_LENGTH],
    key: SigningKey,
}

impl PartialEq for PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl Eq for PrivateKey {}

impl FixedSize for PrivateKey {
    const SIZE: usize = PRIVATE_KEY_LENGTH;
}

impl Write for PrivateKey {
    fn write(&self, buf: &mut impl BufMut) {
        self.raw.write(buf);
    }
}

impl Read for PrivateKey {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _cfg: &()) -> Result<Self, Error> {
        let raw = <[u8; PRIVATE_KEY_LENGTH]>::read_cfg(buf, &())?;
        // `from_slice` rejects zero scalars and values >= the curve order.
        let key = SigningKey::from_slice(&raw)
            .map_err(|_| Error::Invalid("ecdsa::PrivateKey", "invalid scalar"))?;
        Ok(PrivateKey { raw, key })
    }
}

impl Hash for PrivateKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl AsRef<[u8]> for PrivateKey {
    fn as_ref(&self) -> &[u8] {
        &self.raw
    }
}

impl From<SigningKey> for PrivateKey {
    fn from(key: SigningKey) -> Self {
        let raw: [u8; PRIVATE_KEY_LENGTH] = key.to_bytes().into();
        Self { raw, key }
    }
}

impl TryFrom<&[u8]> for PrivateKey {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let raw: [u8; PRIVATE_KEY_LENGTH] = value
            .try_into()
            .map_err(|_| Error::InvalidLength(value.len()))?;
        let key = SigningKey::from_slice(&raw)
            .map_err(|_| Error::Invalid("ecdsa::PrivateKey", "invalid scalar"))?;
        Ok(Self { raw, key })
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

impl Debug for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print key material.
        write!(f, "ecdsa::PrivateKey(REDACTED)")
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// A secp256k1 public identity: the signer's 20-byte Ethereum address.
///
/// This is the operator's registered EigenLayer address and doubles as the p2p
/// identity AND the participant identity key of [`EcdsaScheme`] — participant
/// indices derive from the byte ordering of these addresses, so the set must be
/// identical on every process. Verification is recovery-based (the full public
/// key is recovered from each 65-byte signature and reduced to an address).
#[derive(Clone, Eq, PartialEq)]
pub struct PublicKey {
    raw: [u8; PUBLIC_KEY_LENGTH],
}

impl Span for PublicKey {}

impl Array for PublicKey {}

impl FixedSize for PublicKey {
    const SIZE: usize = PUBLIC_KEY_LENGTH;
}

impl Write for PublicKey {
    fn write(&self, buf: &mut impl BufMut) {
        self.raw.write(buf);
    }
}

impl Read for PublicKey {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _cfg: &()) -> Result<Self, Error> {
        let raw = <[u8; PUBLIC_KEY_LENGTH]>::read_cfg(buf, &())?;
        // The zero address can never be recovered from a valid signature
        // (`ecrecover` failure sentinel) — reject it as an identity outright.
        if raw == [0u8; PUBLIC_KEY_LENGTH] {
            return Err(Error::Invalid("ecdsa::PublicKey", "zero address"));
        }
        Ok(PublicKey { raw })
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

impl From<Address> for PublicKey {
    fn from(address: Address) -> Self {
        Self { raw: address.0.0 }
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != PUBLIC_KEY_LENGTH {
            return Err(Error::InvalidLength(value.len()));
        }
        Self::read_cfg(&mut &*value, &())
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

impl Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_hex(&self.raw, f)
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_hex(&self.raw, f)
    }
}

impl Verifier for PublicKey {
    type Signature = Signature;

    fn verify(&self, namespace: &[u8], message: &[u8], signature: &Signature) -> bool {
        self.verify_message(Some(namespace), message, signature)
    }
}

impl CPublicKey for PublicKey {}

impl PublicKey {
    /// Builds a public key from a `0x`-prefixed (or bare) hex Ethereum address
    /// string, e.g. from an operator's public key-file.
    ///
    /// Returns `None` on malformed input or the zero address.
    pub fn from_address_str(address: &str) -> Option<Self> {
        let address: Address = address.trim().parse().ok()?;
        if address == Address::ZERO {
            return None;
        }
        Some(PublicKey::from(address))
    }

    /// Returns the Ethereum address.
    pub fn address(&self) -> Address {
        Address::from(self.raw)
    }

    /// Verifies a signature over a raw 32-byte digest (the on-chain path):
    /// recover the signer address from the signature and compare.
    pub fn verify_digest(&self, digest: &[u8; DIGEST_LENGTH], signature: &Signature) -> bool {
        signature.recover(digest) == Some(self.address())
    }

    /// Verifies with the old `Option<&[u8]>` namespace semantics (see
    /// [`signing_payload`]). The 2026.5.0 [`Verifier`] trait delegates here with
    /// `Some(namespace)`; internal digest verification uses `None`.
    pub fn verify_message(
        &self,
        namespace: Option<&[u8]>,
        message: &[u8],
        signature: &Signature,
    ) -> bool {
        self.verify_digest(&signing_payload(namespace, message), signature)
    }
}

/// A secp256k1 ECDSA signature: 65 bytes `r || s || v` with `v` in {27, 28}.
///
/// Decoding enforces the canonical form (low-s, non-zero `r`/`s`, Electrum `v`),
/// so equal signatures always have equal bytes. The byte layout is exactly what
/// an Ethereum `ecrecover`-based verifier consumes on-chain.
#[derive(Clone, Eq, PartialEq)]
pub struct Signature {
    raw: [u8; SIGNATURE_LENGTH],
    sig: alloy_primitives::Signature,
}

impl Span for Signature {}

impl Array for Signature {}

impl FixedSize for Signature {
    const SIZE: usize = SIGNATURE_LENGTH;
}

impl Write for Signature {
    fn write(&self, buf: &mut impl BufMut) {
        self.raw.write(buf);
    }
}

impl Read for Signature {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _cfg: &()) -> Result<Self, Error> {
        let raw = <[u8; SIGNATURE_LENGTH]>::read_cfg(buf, &())?;
        // This runs on untrusted network bytes (via `Lazy<Signature>` decode and
        // certificate decode) — it must error, never panic, and must only accept
        // the canonical encoding so byte equality is signature equality.
        let v = raw[SIGNATURE_LENGTH - 1];
        if v != 27 && v != 28 {
            return Err(Error::Invalid("ecdsa::Signature", "v must be 27 or 28"));
        }
        let sig = alloy_primitives::Signature::from_raw_array(&raw)
            .map_err(|_| Error::Invalid("ecdsa::Signature", "invalid signature"))?;
        if sig.r() == U256::ZERO || sig.s() == U256::ZERO {
            return Err(Error::Invalid("ecdsa::Signature", "zero r or s"));
        }
        if sig.s() > HALF_CURVE_ORDER {
            return Err(Error::Invalid("ecdsa::Signature", "high-s (malleable)"));
        }
        Ok(Signature { raw, sig })
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

impl TryFrom<&[u8]> for Signature {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != SIGNATURE_LENGTH {
            return Err(Error::InvalidLength(value.len()));
        }
        Self::read_cfg(&mut &*value, &())
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

impl Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_hex(&self.raw, f)
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_hex(&self.raw, f)
    }
}

impl CSignature for Signature {}

impl Signature {
    /// Wraps an alloy signature, capturing the canonical 65-byte encoding
    /// (`as_bytes` writes `v` as `27 + y_parity`).
    fn from_alloy(sig: alloy_primitives::Signature) -> Self {
        Self {
            raw: sig.as_bytes(),
            sig,
        }
    }

    /// Recovers the signer's Ethereum address from a raw 32-byte digest.
    ///
    /// Returns `None` for cryptographically invalid signatures (recovery failure).
    pub fn recover(&self, digest: &[u8; DIGEST_LENGTH]) -> Option<Address> {
        self.sig
            .recover_address_from_prehash(&B256::from(*digest))
            .ok()
    }
}

/// Builds a signer from a private-key string (an operator key-file format).
///
/// Accepts a `0x`-prefixed hex scalar (the EigenLayer `*.private.ecdsa.key.json`
/// format), a bare 64-char hex scalar, or a decimal scalar (legacy key-file
/// format).
///
/// # Panics
///
/// Panics on a malformed key. Only call with operator-provided configuration at
/// startup, never with network input.
pub fn get_signer(key: &str) -> Ecdsa {
    let key = key.trim();
    let bytes: [u8; PRIVATE_KEY_LENGTH] = if let Some(hex_key) = key.strip_prefix("0x") {
        let decoded = hex::decode(hex_key).expect("invalid hex private key");
        decoded
            .as_slice()
            .try_into()
            .expect("hex private key must be 32 bytes")
    } else if key.len() == 2 * PRIVATE_KEY_LENGTH && key.chars().all(|c| c.is_ascii_hexdigit()) {
        let decoded = hex::decode(key).expect("invalid hex private key");
        decoded
            .as_slice()
            .try_into()
            .expect("hex private key must be 32 bytes")
    } else {
        let value = U256::from_str_radix(key, 10).expect("invalid decimal private key");
        value.to_be_bytes()
    };
    let signing_key = SigningKey::from_slice(&bytes).expect("private key is not a valid scalar");
    Ecdsa::from_signing_key(signing_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_codec::{DecodeExt, Encode};
    use commonware_utils::test_rng;

    /// A fixed digest for deterministic tests.
    fn digest() -> [u8; DIGEST_LENGTH] {
        let mut hasher = Sha256::new();
        hasher.update(b"commonware-restaking ecdsa test digest");
        hasher.finalize().as_ref().try_into().unwrap()
    }

    #[test]
    fn sign_digest_roundtrip() {
        let signer = get_signer("12345");
        let sig = signer.sign_digest(&digest());
        assert!(signer.public_key().verify_digest(&digest(), &sig));

        // Wrong digest is rejected.
        let mut wrong = digest();
        wrong[0] ^= 0x01;
        assert!(!signer.public_key().verify_digest(&wrong, &sig));

        // Wrong key is rejected.
        let other = get_signer("54321");
        assert!(!other.public_key().verify_digest(&digest(), &sig));
    }

    #[test]
    fn signature_is_ecrecover_canonical() {
        // The 65-byte encoding must be `r || s || v` with v in {27, 28} and low-s —
        // exactly what an on-chain `ecrecover`-based verifier consumes. If this test
        // breaks, certificates will no longer verify on-chain.
        for key in ["101", "202", "303", "404"] {
            let signer = get_signer(key);
            let sig = signer.sign_digest(&digest());
            let raw = sig.as_ref();
            assert_eq!(raw.len(), 65);
            let v = raw[64];
            assert!(v == 27 || v == 28, "v must be Electrum notation");
            let s = U256::from_be_slice(&raw[32..64]);
            assert!(s <= HALF_CURVE_ORDER, "s must be low (EIP-2)");
            // Recovery yields the signer's address.
            assert_eq!(sig.recover(&digest()), Some(signer.address()));
        }
    }

    #[test]
    fn get_signer_parses_hex_and_decimal() {
        // 0x-prefixed hex, bare hex, and decimal all derive the same key.
        let decimal = get_signer("12345");
        let hex_key = format!("0x{}", hex::encode(decimal.private_key().as_ref()));
        let from_hex = get_signer(&hex_key);
        assert_eq!(from_hex.address(), decimal.address());
        let bare = hex_key.strip_prefix("0x").unwrap().to_string();
        assert_eq!(get_signer(&bare).address(), decimal.address());
    }

    #[test]
    fn address_matches_alloy_signer() {
        // Parity with alloy's own local-signer wallet: same private key, same
        // address.
        let key = get_signer("31337");
        let hex_key = format!("0x{}", hex::encode(key.private_key().as_ref()));
        let alloy_signer: alloy_signer_local::PrivateKeySigner = hex_key.parse().unwrap();
        assert_eq!(key.address(), alloy_signer.address());
    }

    #[test]
    fn sign_digest_matches_legacy_none_namespace_semantics() {
        let signer = get_signer("999");
        let raw = signer.sign_digest(&digest());
        let legacy = signer.sign_message(None, &digest());
        assert_eq!(raw.as_ref(), legacy.as_ref());
    }

    #[test]
    fn namespaced_signing_is_domain_separated() {
        let signer = get_signer("777");
        let public = signer.public_key();
        let message = b"some message";

        let sig = Signer::sign(&signer, b"ns-a", message);
        assert!(Verifier::verify(&public, b"ns-a", message, &sig));
        // Different namespace or message fails.
        assert!(!Verifier::verify(&public, b"ns-b", message, &sig));
        assert!(!Verifier::verify(&public, b"ns-a", b"other message", &sig));

        // The namespaced path never collides with the raw digest path, even when
        // the message is 32 bytes.
        let sig32 = Signer::sign(&signer, b"ns-a", &digest());
        assert_ne!(sig32.as_ref(), signer.sign_digest(&digest()).as_ref());
    }

    #[test]
    fn none_namespace_non_digest_message_is_hashed() {
        let signer = get_signer("31337");
        let message = b"not thirty-two bytes";
        let sig = signer.sign_message(None, message);

        let mut hasher = Sha256::new();
        hasher.update(message);
        let hashed: [u8; DIGEST_LENGTH] = hasher.finalize().as_ref().try_into().unwrap();
        assert_eq!(sig.as_ref(), signer.sign_digest(&hashed).as_ref());
    }

    #[test]
    fn public_key_codec_roundtrip() {
        let public = get_signer("424242").public_key();
        let encoded = public.encode();
        assert_eq!(encoded.len(), PublicKey::SIZE);
        let decoded = PublicKey::decode(encoded).unwrap();
        assert_eq!(decoded, public);
    }

    #[test]
    fn signature_codec_roundtrip() {
        let sig = get_signer("424242").sign_digest(&digest());
        let encoded = sig.encode();
        assert_eq!(encoded.len(), Signature::SIZE);
        let decoded = Signature::decode(encoded).unwrap();
        assert_eq!(decoded, sig);
        // The decoded signature still recovers.
        assert_eq!(
            decoded.recover(&digest()),
            Some(get_signer("424242").address())
        );
    }

    #[test]
    fn private_key_codec_roundtrip() {
        let private = get_signer("13").private_key();
        let encoded = private.encode();
        assert_eq!(encoded.len(), PrivateKey::SIZE);
        let decoded = PrivateKey::decode(encoded).unwrap();
        assert_eq!(decoded, private);
        // Re-derived signer produces identical signatures.
        let rebuilt = Ecdsa::new(decoded);
        assert_eq!(
            rebuilt.sign_digest(&digest()).as_ref(),
            get_signer("13").sign_digest(&digest()).as_ref()
        );
    }

    #[test]
    fn signature_decode_rejects_non_canonical() {
        let sig = get_signer("7").sign_digest(&digest());
        let raw: [u8; SIGNATURE_LENGTH] = sig.as_ref().try_into().unwrap();

        // Bad parity byte.
        let mut bad_v = raw;
        bad_v[64] = 29;
        assert!(Signature::decode(&bad_v[..]).is_err());
        let mut zero_v = raw;
        zero_v[64] = 0;
        assert!(
            Signature::decode(&zero_v[..]).is_err(),
            "y-parity form (v=0/1) must be rejected to keep the encoding canonical"
        );

        // High-s twin (same ecrecover result under a naive verifier) is rejected.
        let curve_order = U256::from_be_slice(&[
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xfe, 0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b, 0xbf, 0xd2, 0x5e, 0x8c,
            0xd0, 0x36, 0x41, 0x41,
        ]);
        let s = U256::from_be_slice(&raw[32..64]);
        let high_s = curve_order - s;
        let mut malleated = raw;
        malleated[32..64].copy_from_slice(&high_s.to_be_bytes::<32>());
        malleated[64] = if raw[64] == 27 { 28 } else { 27 };
        assert!(Signature::decode(&malleated[..]).is_err());

        // Zero r / zero s.
        let mut zero_r = raw;
        zero_r[0..32].fill(0);
        assert!(Signature::decode(&zero_r[..]).is_err());
        let mut zero_s = raw;
        zero_s[32..64].fill(0);
        assert!(Signature::decode(&zero_s[..]).is_err());
    }

    #[test]
    fn malformed_bytes_error_instead_of_panicking() {
        assert!(PublicKey::decode(&[0u8; PUBLIC_KEY_LENGTH][..]).is_err()); // zero address
        assert!(Signature::decode(&[0xffu8; SIGNATURE_LENGTH][..]).is_err());
        assert!(PrivateKey::decode(&[0u8; PRIVATE_KEY_LENGTH][..]).is_err()); // zero scalar
        assert!(PrivateKey::decode(&[0xffu8; PRIVATE_KEY_LENGTH][..]).is_err()); // >= order
    }

    #[test]
    fn try_from_rejects_bad_lengths() {
        assert!(matches!(
            PublicKey::try_from(&[1u8; 5][..]),
            Err(Error::InvalidLength(5))
        ));
        assert!(matches!(
            Signature::try_from(&[1u8; 33][..]),
            Err(Error::InvalidLength(33))
        ));
        assert!(matches!(
            PrivateKey::try_from(&[1u8; 31][..]),
            Err(Error::InvalidLength(31))
        ));
    }

    #[test]
    fn from_address_str_roundtrip() {
        let signer = get_signer("2718281828");
        let hex_addr = format!("{}", signer.public_key());
        let rebuilt = PublicKey::from_address_str(&hex_addr).unwrap();
        assert_eq!(rebuilt, signer.public_key());

        assert!(PublicKey::from_address_str("not an address").is_none());
        assert!(
            PublicKey::from_address_str("0x0000000000000000000000000000000000000000").is_none()
        );
    }

    #[test]
    fn random_signers_are_distinct_and_functional() {
        let mut rng = test_rng();
        let a = Ecdsa::random(&mut rng);
        let b = Ecdsa::random(&mut rng);
        assert_ne!(a.public_key(), b.public_key());
        let sig = a.sign_digest(&digest());
        assert!(a.public_key().verify_digest(&digest(), &sig));

        // `from_seed` (provided by the Signer trait via Random) is deterministic.
        let s1 = Ecdsa::from_seed(7);
        let s2 = Ecdsa::from_seed(7);
        assert_eq!(s1.public_key(), s2.public_key());
    }
}
