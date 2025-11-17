use anyhow::Result;
use commonware_cryptography::sha256::Digest;

#[async_trait::async_trait]
pub trait ValidatorTrait: Send + Sync {
    /// Validates a message and returns the expected hash.
    async fn validate_and_return_expected_hash(&self, msg: &[u8]) -> Result<Digest>;

    /// Extracts and hashes the payload from a message.
    async fn get_payload_from_message(&self, msg: &[u8]) -> Result<Digest>;
}
