/// Generic verification data that can be used by different verification methods
///
/// This type is executor-agnostic and can be converted to executor-specific
/// types by the executor implementation.
#[derive(Debug, Clone)]
pub struct VerificationData {
    pub signatures: Vec<Vec<u8>>,
    pub public_keys: Vec<Vec<u8>>,
    /// Additional context data that might be needed by specific verification methods
    pub context: Option<Vec<u8>>,
}

impl VerificationData {
    pub fn new(signatures: Vec<Vec<u8>>, public_keys: Vec<Vec<u8>>) -> Self {
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

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub transaction_hash: String,
    pub block_number: Option<u64>,
    pub gas_used: Option<u64>,
    pub status: Option<bool>,
    pub contract_address: Option<String>,
}
