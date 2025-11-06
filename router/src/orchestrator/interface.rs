use async_trait::async_trait;
use bn254::PublicKey;
use commonware_p2p::{Receiver, Sender};

/// Trait defining the interface for orchestrator implementations.
///
/// This trait provides a generic interface for orchestration operations,
/// allowing different implementations to be swapped without changing
/// the consuming code. The orchestrator coordinates the entire aggregation
/// process including task creation, validation, and execution.
#[async_trait]
pub trait OrchestratorTrait: Send + Sync {
    /// Runs the orchestration process with the given sender and receiver.
    ///
    /// This method coordinates the entire aggregation process:
    /// - Creates tasks and payloads
    /// - Broadcasts messages to contributors
    /// - Collects and validates signatures
    /// - Executes verification when threshold is reached
    ///
    /// # Arguments
    /// * `sender` - The sender for broadcasting messages
    /// * `receiver` - The receiver for collecting messages
    ///
    /// # Returns
    /// * `()` - This method runs indefinitely until interrupted
    async fn run(self, sender: impl Sender, receiver: impl Receiver<PublicKey = PublicKey>);
}
