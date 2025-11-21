use anyhow::Result;
use async_trait::async_trait;

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
