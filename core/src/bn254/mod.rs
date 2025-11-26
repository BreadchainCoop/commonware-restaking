use ark_bn254::{Fq, Fq2, Fr as Scalar, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{AffineRepr, CurveGroup, PrimeGroup, pairing::Pairing};
use ark_ff::AdditiveGroup;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use bytes::Buf;
use bytes::buf::BufMut;
use commonware_codec::{Error, FixedSize, Read, Write};
use commonware_cryptography::{
    Hasher as _, PublicKey as CPublicKey, Sha256, Signature as CSignature, Signer, Verifier,
};
use commonware_utils::{Array, Span, hex, union_unique};
use eigen_crypto_bn254::utils::map_to_curve;
use std::str::FromStr;
use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    ops::Deref,
};

const DIGEST_LENGTH: usize = 32;
const PRIVATE_KEY_LENGTH: usize = 32;
const G1_LENGTH: usize = 32;
const SIGNATURE_LENGTH: usize = G1_LENGTH;
const G2_LENGTH: usize = 64;
const PUBLIC_KEY_LENGTH: usize = G2_LENGTH;

/// If message provided is exactly 32 bytes, it is assumed to be a hash digest.
#[derive(Clone)]
pub struct Bn254 {
    private: Scalar,
    public: G2Affine,
}

impl Signer for Bn254 {
    type Signature = Signature;
    type PublicKey = PublicKey;

    fn sign(&self, namespace: Option<&[u8]>, message: &[u8]) -> Signature {
        // Generate payload
        let hash: [u8; DIGEST_LENGTH] = if namespace.is_none() && message.len() == DIGEST_LENGTH {
            message.try_into().unwrap()
        } else {
            let payload = match namespace {
                Some(namespace) => Cow::Owned(union_unique(namespace, message)),
                None => Cow::Borrowed(message),
            };
            let mut hasher = Sha256::new();
            hasher.update(payload.as_ref());
            let hash = hasher.finalize();
            hash.as_ref().try_into().unwrap()
        };

        // Map to curve
        let msg_on_g1 = map_to_curve(&hash);

        // Generate signature
        let sig = msg_on_g1 * self.private;
        let sig = sig.into_affine();

        // Serialize signature
        Signature::from(sig)
    }

    fn public_key(&self) -> PublicKey {
        PublicKey::from(self.public)
    }
}

impl Verifier for Bn254 {
    type Signature = Signature;

    fn verify(&self, namespace: Option<&[u8]>, message: &[u8], signature: &Signature) -> bool {
        // Generate payload
        let hash: [u8; DIGEST_LENGTH] = if namespace.is_none() && message.len() == DIGEST_LENGTH {
            message.try_into().unwrap()
        } else {
            let payload = match namespace {
                Some(namespace) => Cow::Owned(union_unique(namespace, message)),
                None => Cow::Borrowed(message),
            };
            let mut hasher = Sha256::new();
            hasher.update(payload.as_ref());
            let hash = hasher.finalize();
            hash.as_ref().try_into().unwrap()
        };

        // Map to curve
        let msg_on_g1 = map_to_curve(&hash);

        // Pairing check
        let lhs = ark_bn254::Bn254::pairing(msg_on_g1, self.public);
        let rhs = ark_bn254::Bn254::pairing(signature.sig, G2Affine::generator());
        lhs == rhs
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct PrivateKey {
    raw: [u8; PRIVATE_KEY_LENGTH],
    key: Scalar,
}

impl Span for PrivateKey {}

impl Array for PrivateKey {}

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
        let mut raw = <[u8; PRIVATE_KEY_LENGTH]>::read_cfg(buf, &())?;
        let dst: &[u8] = &mut raw;
        let key = Scalar::deserialize_compressed(dst).expect("Wrong Private Key");
        Ok(PrivateKey { raw, key })
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

impl From<Scalar> for PrivateKey {
    fn from(key: Scalar) -> Self {
        let mut raw = [0u8; PRIVATE_KEY_LENGTH];
        key.serialize_compressed(&mut raw[..]).unwrap();
        Self { raw, key }
    }
}

impl TryFrom<&[u8]> for PrivateKey {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let raw: [u8; PRIVATE_KEY_LENGTH] = value.try_into().expect("Invalid Private Key Length");
        let key = Scalar::deserialize_compressed(value).expect("Invalid Private Key");
        if key == Scalar::ZERO {
            return Err(Error::InvalidUsize);
        }
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
        write!(f, "{}", hex(&self.raw))
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex(&self.raw))
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct PublicKey {
    raw: [u8; PUBLIC_KEY_LENGTH],
    key: G2Affine,
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
        let mut raw = <[u8; PUBLIC_KEY_LENGTH]>::read_cfg(buf, &())?;
        let dst: &[u8] = &mut raw;
        let key = G2Affine::deserialize_compressed(dst).expect("Wrong Public Key");
        Ok(PublicKey { raw, key })
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

impl From<G2Affine> for PublicKey {
    fn from(key: G2Affine) -> Self {
        let mut raw = [0u8; PUBLIC_KEY_LENGTH];
        key.serialize_compressed(&mut raw[..]).unwrap();
        Self { raw, key }
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let raw: [u8; PUBLIC_KEY_LENGTH] =
            TryInto::<[u8; PUBLIC_KEY_LENGTH]>::try_into(value).expect("Invalid Public Key Length");
        let key = G2Affine::deserialize_compressed(value).expect("Invalid Public Key");
        if !key.is_in_correct_subgroup_assuming_on_curve() || !key.is_on_curve() || key.is_zero() {
            return Err(Error::InvalidUsize);
        }
        Ok(Self { raw, key })
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
        write!(f, "{}", hex(&self.raw))
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex(&self.raw))
    }
}

impl Verifier for PublicKey {
    type Signature = Signature;

    fn verify(&self, namespace: Option<&[u8]>, message: &[u8], signature: &Signature) -> bool {
        // Generate payload
        let hash: [u8; DIGEST_LENGTH] = if namespace.is_none() && message.len() == DIGEST_LENGTH {
            message.try_into().unwrap()
        } else {
            let payload = match namespace {
                Some(namespace) => Cow::Owned(union_unique(namespace, message)),
                None => Cow::Borrowed(message),
            };
            let mut hasher = Sha256::new();
            hasher.update(payload.as_ref());
            let hash = hasher.finalize();
            hash.as_ref().try_into().unwrap()
        };

        // Map to curve
        let msg_on_g1 = map_to_curve(&hash);

        // Pairing check
        let lhs = ark_bn254::Bn254::pairing(msg_on_g1, self.key);
        let rhs = ark_bn254::Bn254::pairing(signature.sig, G2Affine::generator());
        lhs == rhs
    }
}

impl CPublicKey for PublicKey {}

impl PublicKey {
    pub fn create_from_g2_coordinates(x1: &str, x2: &str, y1: &str, y2: &str) -> Option<Self> {
        // Convert string coordinates to Fq elements
        let x1_fq = Fq::from_str(x1).ok()?;
        let x2_fq = Fq::from_str(x2).ok()?;
        let y1_fq = Fq::from_str(y1).ok()?;
        let y2_fq = Fq::from_str(y2).ok()?;

        // Create Fq2 elements for G2
        let x_fq2 = Fq2::new(x1_fq, x2_fq);
        let y_fq2 = Fq2::new(y1_fq, y2_fq);

        // Create G2 point from coordinates
        let g2_point = G2Affine::new_unchecked(x_fq2, y_fq2);

        // Convert to PublicKey
        Some(PublicKey::from(g2_point))
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Signature {
    raw: [u8; SIGNATURE_LENGTH],
    sig: G1Affine,
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
        let mut raw = <[u8; SIGNATURE_LENGTH]>::read_cfg(buf, &())?;
        let dst: &[u8] = &mut raw;
        let sig = G1Affine::deserialize_compressed(dst).expect("Wrong Signature");
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

impl From<G1Affine> for Signature {
    fn from(sig: G1Affine) -> Self {
        let mut raw = [0u8; SIGNATURE_LENGTH];
        sig.serialize_compressed(&mut raw[..]).unwrap();
        Self { raw, sig }
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let raw: [u8; SIGNATURE_LENGTH] =
            TryInto::<[u8; SIGNATURE_LENGTH]>::try_into(value).expect("Invalid Signature Length");
        let sig = G1Affine::deserialize_compressed(value).expect("Invalid Signature");
        if !sig.is_in_correct_subgroup_assuming_on_curve() || !sig.is_on_curve() || sig.is_zero() {
            return Err(Error::InvalidBool);
        }
        Ok(Self { raw, sig })
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
        write!(f, "{}", hex(&self.raw))
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex(&self.raw))
    }
}

impl CSignature for Signature {}

// TODO: cleanup handling of G1 vs G2 public keys (+ unify with signature)
#[derive(Clone, Eq, PartialEq)]
pub struct G1PublicKey {
    raw: [u8; G1_LENGTH],
    key: G1Affine,
}

impl Span for G1PublicKey {}

impl Array for G1PublicKey {}

impl FixedSize for G1PublicKey {
    const SIZE: usize = G1_LENGTH;
}

impl Write for G1PublicKey {
    fn write(&self, buf: &mut impl BufMut) {
        self.raw.write(buf);
    }
}

impl Read for G1PublicKey {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _cfg: &()) -> Result<Self, Error> {
        let mut raw = <[u8; G1_LENGTH]>::read_cfg(buf, &())?;
        let dst: &[u8] = &mut raw;
        let key = G1Affine::deserialize_compressed(dst).expect("Wrong G1 Public Key");
        Ok(G1PublicKey { raw, key })
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

impl From<G1Affine> for G1PublicKey {
    fn from(key: G1Affine) -> Self {
        let mut raw = [0u8; G1_LENGTH];
        key.serialize_compressed(&mut raw[..]).unwrap();
        Self { raw, key }
    }
}

impl G1PublicKey {
    pub fn create_from_g1_coordinates(x: &str, y: &str) -> Option<Self> {
        let x_fq = Fq::from_str(x).ok()?;
        let y_fq = Fq::from_str(y).ok()?;
        let g1_affine = G1Affine::new_unchecked(x_fq, y_fq);
        let g1_pub_key = G1PublicKey::from(g1_affine);
        Some(g1_pub_key)
    }

    /// Returns the x-coordinate of the G1 point as a string
    pub fn get_x(&self) -> String {
        self.key.x.to_string()
    }

    /// Returns the y-coordinate of the G1 point as a string
    pub fn get_y(&self) -> String {
        self.key.y.to_string()
    }

    /// Returns the raw G1Affine point
    pub fn get_point(&self) -> G1Affine {
        self.key
    }
}

impl TryFrom<&[u8]> for G1PublicKey {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let raw: [u8; G1_LENGTH] =
            TryInto::<[u8; G1_LENGTH]>::try_into(value).expect("Invalid G1 Public Key Length");
        let key = G1Affine::deserialize_compressed(value).expect("Invalid G1 Public Key");
        if !key.is_in_correct_subgroup_assuming_on_curve() || !key.is_on_curve() || key.is_zero() {
            return Err(Error::EndOfBuffer);
        }
        Ok(Self { raw, key })
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

impl Debug for G1PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex(&self.raw))
    }
}

impl Display for G1PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex(&self.raw))
    }
}

pub fn get_points(
    g1: &[G1PublicKey],
    g2: &[PublicKey],
    signatures: &[Signature],
) -> Option<(G1Affine, G2Affine, G1Affine)> {
    let mut agg_public_g1 = G1Projective::ZERO;
    for public in g1 {
        agg_public_g1 += public.key.into_group();
    }
    let agg_public_g1 = agg_public_g1.into_affine();

    let mut agg_public_g2 = G2Projective::ZERO;
    for public in g2 {
        agg_public_g2 += public.key.into_group();
    }
    let agg_public_g2 = agg_public_g2.into_affine();

    let mut agg_signature = G1Projective::ZERO;
    for signature in signatures {
        agg_signature += signature.sig.into_group();
    }
    let agg_signature = agg_signature.into_affine();
    Some((agg_public_g1, agg_public_g2, agg_signature))
}

pub fn aggregate_signatures(signatures: &[Signature]) -> Option<Signature> {
    let mut agg_signature = G1Projective::ZERO;
    for signature in signatures {
        agg_signature += signature.sig.into_group();
    }
    Some(Signature::from(agg_signature.into_affine()))
}

pub fn aggregate_verify(
    publics: &[PublicKey],
    namespace: Option<&[u8]>,
    message: &[u8],
    signature: &Signature,
) -> bool {
    // Aggregate public keys
    let mut agg_public = G2Projective::ZERO;
    for public in publics {
        agg_public += public.key.into_group();
    }
    let agg_public = agg_public.into_affine();
    let _public = PublicKey::from(agg_public);

    // Verify signature
    let verifier = Bn254 {
        private: Scalar::ZERO, // dummy value for verification
        public: agg_public,
    };
    verifier.verify(namespace, message, signature)
}

impl Bn254 {
    pub fn new(private_key: PrivateKey) -> Result<Self, Error> {
        let public_g2 = (ark_bn254::G2Projective::generator() * private_key.key).into_affine();
        Ok(Bn254 {
            private: private_key.key,
            public: public_g2,
        })
    }

    // Alternative constructor from scalar directly
    pub fn from_scalar(scalar: Scalar) -> Self {
        let public_g2 = (ark_bn254::G2Projective::generator() * scalar).into_affine();
        Bn254 {
            private: scalar,
            public: public_g2,
        }
    }

    pub fn public_g1(&self) -> G1PublicKey {
        let pk = G1Projective::generator() * self.private;
        G1PublicKey::from(pk.into_affine())
    }
}

pub fn get_signer_from_fr(key: &str) -> Bn254 {
    let fr = Scalar::from_str(key).expect("Invalid decimal string for private key");
    let key = PrivateKey::from(fr);
    Bn254::new(key).expect("Failed to create signer")
}

pub fn get_signer(key: &str) -> Bn254 {
    let fr = Scalar::from_str(key).expect("Invalid decimal string for private key");
    let key = PrivateKey::from(fr);
    Bn254::new(key).expect("Failed to create signer")
}
