use bn254::{PublicKey, Signature};

/// Generic verification data that can be used by different verification methods
#[derive(Debug, Clone)]
pub struct VerificationData {
    pub signatures: Vec<Signature>,
    pub public_keys: Vec<PublicKey>,
    /// Additional context data that might be needed by specific verification methods
    pub context: Option<Vec<u8>>,
}

impl VerificationData {
    pub fn new(signatures: Vec<Signature>, public_keys: Vec<PublicKey>) -> Self {
        Self {
            signatures,
            public_keys,
            context: None,
        }
    }

    pub fn with_context(mut self, context: Vec<u8>) -> Self {
        self.context = Some(context);
        self
    }
}
