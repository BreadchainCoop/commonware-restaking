use crate::executor::ExecutionResult;
use alloy_primitives::{Bytes, FixedBytes};
use anyhow::Result;
use async_trait::async_trait;
use commonware_avs_bindings::blssigcheckoperatorstateretriever::BLSSigCheckOperatorStateRetriever::getNonSignerStakesAndSignatureReturn;

use super::types::BlsVerificationData;

/// BLS-specific executor trait that handles the lower-level BLS signature verification
#[async_trait]
pub trait BlsExecutorTrait<T = ()>: Send + Sync
where
    T: Send + Sync,
{
    async fn execute_bls_verification(
        &mut self,
        payload_hash: &[u8],
        verification_data: BlsVerificationData,
        task_data: Option<&T>,
    ) -> Result<ExecutionResult>;
}

/// Contract-specific handler for BLS signature verification in EigenLayer context
#[async_trait]
pub trait BlsSignatureVerificationHandler: Send + Sync {
    type TaskData: Send + Sync;

    async fn handle_verification(
        &mut self,
        msg_hash: FixedBytes<32>,
        quorum_numbers: Bytes,
        current_block_number: u32,
        non_signer_data: getNonSignerStakesAndSignatureReturn,
        task_data: Option<&Self::TaskData>,
    ) -> Result<ExecutionResult>;

    /// Returns a key identifying which read-side config to use for this task.
    /// The key must match one registered via `BlsEigenlayerExecutor::add_read_side()`.
    /// Returns `None` to use the executor's default read-side config.
    fn resolve_read_side(&self, _task_data: Option<&Self::TaskData>) -> Option<String> {
        None
    }
}
