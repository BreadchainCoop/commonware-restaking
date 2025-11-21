use anyhow::Result;
use commonware_cryptography::sha256::Digest;

#[async_trait::async_trait]
pub trait ValidatorTrait: Send + Sync {
    /// Validates a message and returns the expected hash.
    async fn validate_and_return_expected_hash(&self, msg: &[u8]) -> Result<Digest>;

    /// Extracts and hashes the payload from a message.
    async fn get_payload_from_message(&self, msg: &[u8]) -> Result<Digest>;
}

/// Generic validator wrapper that delegates to a ValidatorTrait implementation.
pub struct Validator<T: ValidatorTrait> {
    pub validator_impl: T,
}

impl<T: ValidatorTrait> Validator<T> {
    #[allow(dead_code)]
    pub fn new(validator_impl: T) -> Self {
        Self { validator_impl }
    }

    #[allow(dead_code)]
    pub async fn validate_and_return_expected_hash(&self, msg: &[u8]) -> Result<Digest> {
        self.validator_impl
            .validate_and_return_expected_hash(msg)
            .await
    }

    #[allow(dead_code)]
    pub async fn get_payload_from_message(&self, msg: &[u8]) -> Result<Digest> {
        self.validator_impl.get_payload_from_message(msg).await
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod tests;

#[cfg(any(test, feature = "test-utils"))]
pub use tests::mock::MockValidator;
