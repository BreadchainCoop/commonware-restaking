use bn254::{G1PublicKey, PublicKey, Signature};

/// BLS-specific verification data that includes G1 public keys
#[derive(Debug, Clone)]
pub struct BlsVerificationData {
    pub signatures: Vec<Signature>,
    pub public_keys: Vec<PublicKey>,
    pub g1_public_keys: Vec<G1PublicKey>,
}

impl BlsVerificationData {
    pub fn new(
        signatures: Vec<Signature>,
        public_keys: Vec<PublicKey>,
        g1_public_keys: Vec<G1PublicKey>,
    ) -> Self {
        Self {
            signatures,
            public_keys,
            g1_public_keys,
        }
    }
}
