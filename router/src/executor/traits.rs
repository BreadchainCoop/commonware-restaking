use anyhow::Result;
use async_trait::async_trait;
use commonware_avs_core::bn254::{G1PublicKey, PublicKey, Signature};

use crate::executor::types::ExecutionResult;

#[async_trait]
pub trait VerificationExecutor<T = (), V = ()>: Send + Sync
where
    T: Send + Sync,
    V: Send + Sync,
{
    async fn execute_verification(
        &mut self,
        payload_hash: &[u8],
        verification_data: V,
        task_data: Option<&T>,
    ) -> Result<ExecutionResult>;
}

/// Builds a verification-data container from BLS aggregation output: the per-signer
/// signatures, their public keys, and their G1 public keys.
pub trait FromBlsAggregation {
    fn from_bls_aggregation(
        signatures: Vec<Signature>,
        public_keys: Vec<PublicKey>,
        g1_public_keys: Vec<G1PublicKey>,
    ) -> Self;
}
