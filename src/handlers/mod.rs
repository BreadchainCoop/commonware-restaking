pub mod creator;
pub mod executor;
pub mod listening_creator;
pub mod orchestrator;

pub use orchestrator::Orchestrator;

use crate::handlers::{creator::Creator, listening_creator::ListeningCreator};
use std::sync::Arc;

/// Shared trait for creators that can generate payloads and round numbers
pub trait TaskCreator: Send + Sync {
    /// Get the current payload and round number
    async fn get_payload_and_round(&self) -> anyhow::Result<(Vec<u8>, u64)>;
}
enum TaskCreatorEnum {
    Creator(Creator),
    ListeningCreator(Arc<ListeningCreator>),
}

impl TaskCreator for TaskCreatorEnum {
    async fn get_payload_and_round(&self) -> anyhow::Result<(Vec<u8>, u64)> {
        match self {
            TaskCreatorEnum::Creator(creator) => creator
                .get_payload_and_round()
                .await
                .map_err(|e| anyhow::anyhow!("Creator error: {}", e)),
            TaskCreatorEnum::ListeningCreator(listening_creator) => listening_creator
                .get_payload_and_round()
                .await
                .map_err(|e| anyhow::anyhow!("ListeningCreator error: {}", e)),
        }
    }
}
