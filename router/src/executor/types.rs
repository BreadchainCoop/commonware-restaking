use bytes::Bytes;
use commonware_avs_core::bn254::{G1PublicKey, PublicKey, Signature};

use crate::executor::traits::FromBlsAggregation;

/// Generic verification data that can be used by different verification methods
///
/// This type is executor-agnostic and can be converted to executor-specific
/// types by the executor implementation.
#[derive(Debug, Clone)]
pub struct VerificationData {
    pub signatures: Vec<Bytes>,
    pub public_keys: Vec<Bytes>,
    /// Additional context data that might be needed by specific verification methods
    pub context: Option<Bytes>,
}

impl VerificationData {
    pub fn new(signatures: Vec<Bytes>, public_keys: Vec<Bytes>) -> Self {
        Self {
            signatures,
            public_keys,
            context: None,
        }
    }

    pub fn with_context(mut self, context: Bytes) -> Self {
        self.context = Some(context);
        self
    }
}

impl FromBlsAggregation for VerificationData {
    fn from_bls_aggregation(
        signatures: Vec<Signature>,
        public_keys: Vec<PublicKey>,
        g1_public_keys: Vec<G1PublicKey>,
    ) -> Self {
        let signatures = signatures.iter().map(|s| Bytes::from(s.to_vec())).collect();
        let public_keys = public_keys
            .iter()
            .map(|pk| Bytes::from(pk.to_vec()))
            .collect();
        let mut context = Vec::new();
        for g1_pubkey in &g1_public_keys {
            context.extend_from_slice(g1_pubkey);
        }
        Self::new(signatures, public_keys).with_context(Bytes::from(context))
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
