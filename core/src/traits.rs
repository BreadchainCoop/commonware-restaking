use anyhow::Result;
use async_trait::async_trait;
use commonware_codec::{EncodeSize, Read, Write};

use crate::types::VerificationData;

#[async_trait]
pub trait Creator: Send + Sync {
    /// Associated type for task metadata. Each creator implementation defines its own data structure.
    type TaskData: Send + Sync + Clone + Write + Read<Cfg = ()> + EncodeSize;

    /// Compute and return the payload bytes and associated round.
    async fn get_payload_and_round(&self) -> Result<(Vec<u8>, u64)>;

    /// Get task metadata as the creator's specific data type.
    ///
    /// These metadata fields are used in the wire protocol messages and are typically
    /// specific to the use case being implemented (e.g., counter, voting, etc.).
    /// Each creator implementation defines its own TaskData type for type safety.
    ///
    /// # Returns
    /// * `Self::TaskData` - The task metadata in the creator's specific format
    fn get_task_metadata(&self) -> Self::TaskData;
}

#[async_trait]
pub trait VerificationExecutor<T = ()>: Send + Sync
where
    T: Send + Sync,
{
    async fn execute_verification(
        &mut self,
        payload_hash: &[u8],
        verification_data: VerificationData,
        task_data: Option<&T>,
    ) -> Result<ExecutionResult>;
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub transaction_hash: String,
    pub block_number: Option<u64>,
    pub gas_used: Option<u64>,
    pub status: Option<bool>,
    pub contract_address: Option<String>,
}
