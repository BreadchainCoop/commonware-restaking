use anyhow::Result;
use commonware_cryptography::sha256::Digest;

/// Trait defining the interface for validator implementations.
///
/// This trait provides a generic interface for validation operations,
/// allowing different implementations to be swapped without changing
/// the consuming code.
#[async_trait::async_trait]
pub trait ValidatorTrait: Send + Sync {
    /// Validates a message and returns the expected hash.
    ///
    /// This method performs validation on the given message and returns
    /// the expected hash that should be computed from the message payload.
    /// The validation typically includes checking message format, round numbers,
    /// and other business logic specific to the implementation.
    ///
    /// # Arguments
    /// * `msg` - The message bytes to validate
    ///
    /// # Returns
    /// * `Result<Digest>` - The expected hash if validation succeeds, or an error
    async fn validate_and_return_expected_hash(&self, msg: &[u8]) -> Result<Digest>;

    /// Extracts and hashes the payload from a message.
    ///
    /// This method decodes the message and extracts the payload,
    /// then computes a hash of that payload. This is typically used
    /// for creating deterministic hashes from message content.
    ///
    /// # Arguments
    /// * `msg` - The message bytes to extract payload from
    ///
    /// # Returns
    /// * `Result<Digest>` - The hash of the extracted payload, or an error
    async fn get_payload_from_message(&self, msg: &[u8]) -> Result<Digest>;
}
