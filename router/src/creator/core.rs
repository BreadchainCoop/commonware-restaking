use anyhow::Result;
use async_trait::async_trait;
use commonware_codec::{EncodeSize, Read, Write};

/// The main creator trait. Implementations produce a payload and a round number.
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
