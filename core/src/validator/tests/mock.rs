use alloy::sol_types::SolValue;
use alloy_primitives::U256;
use anyhow::Result;
use commonware_cryptography::sha256::Digest;
use commonware_cryptography::{Hasher, Sha256};
use std::sync::{Arc, Mutex};

use crate::validator::interface::ValidatorTrait;

/// Mock validator implementation for testing purposes.
///
/// This implementation provides a configurable mock that can be used
/// for unit testing without requiring a real connection to a node. It allows
/// for predictable behavior and easy test scenario setup.
#[allow(dead_code)]
pub struct MockValidator {
    /// The expected round number for validation
    expected_round: u64,
    /// Whether validation should succeed or fail
    should_succeed: bool,
    /// Custom error message to return on failure
    error_message: Option<String>,
    /// Counter for tracking validation attempts
    validation_count: Arc<Mutex<u64>>,
}

#[allow(dead_code)]
impl MockValidator {
    /// Creates a new MockValidator that always succeeds.
    ///
    /// This constructor creates a mock validator that will accept
    /// any message and return a predictable hash.
    ///
    /// # Arguments
    /// * `expected_round` - The round number that will be used for validation
    ///
    /// # Returns
    /// * `Self` - The new MockValidator instance
    pub fn new_success(expected_round: u64) -> Self {
        Self {
            expected_round,
            should_succeed: true,
            error_message: None,
            validation_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Creates a new MockValidator that always fails.
    ///
    /// This constructor creates a mock validator that will reject
    /// any message with a custom error message.
    ///
    /// # Arguments
    /// * `error_message` - The error message to return on validation failure
    ///
    /// # Returns
    /// * `Self` - The new MockValidator instance
    pub fn new_failure(error_message: String) -> Self {
        Self {
            expected_round: 0,
            should_succeed: false,
            error_message: Some(error_message),
            validation_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Creates a new MockValidator with custom configuration.
    ///
    /// This constructor allows for fine-grained control over the mock's behavior.
    ///
    /// # Arguments
    /// * `expected_round` - The round number that will be used for validation
    /// * `should_succeed` - Whether validation should succeed or fail
    /// * `error_message` - Optional error message for failure scenarios
    ///
    /// # Returns
    /// * `Self` - The new MockValidator instance
    pub fn new_custom(
        expected_round: u64,
        should_succeed: bool,
        error_message: Option<String>,
    ) -> Self {
        Self {
            expected_round,
            should_succeed,
            error_message,
            validation_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Updates the expected round number.
    ///
    /// This method allows changing the expected round number
    /// during test execution.
    ///
    /// # Arguments
    /// * `round` - The new expected round number
    pub fn set_expected_round(&mut self, round: u64) {
        self.expected_round = round;
    }

    /// Updates the success/failure behavior.
    ///
    /// This method allows changing whether validation should
    /// succeed or fail during test execution.
    ///
    /// # Arguments
    /// * `should_succeed` - Whether validation should succeed
    pub fn set_should_succeed(&mut self, should_succeed: bool) {
        self.should_succeed = should_succeed;
    }

    /// Updates the error message for failure scenarios.
    ///
    /// This method allows changing the error message that
    /// will be returned on validation failure.
    ///
    /// # Arguments
    /// * `error_message` - The new error message
    pub fn set_error_message(&mut self, error_message: Option<String>) {
        self.error_message = error_message;
    }

    /// Gets the current validation count.
    ///
    /// This method is useful for testing to verify how many times
    /// validation was attempted.
    ///
    /// # Returns
    /// * `u64` - The current validation count
    pub fn get_validation_count(&self) -> u64 {
        *self.validation_count.lock().unwrap()
    }

    /// Resets the validation count to zero.
    ///
    /// This method is useful for testing to reset the counter
    /// between test scenarios.
    pub fn reset_validation_count(&mut self) {
        let mut count = self.validation_count.lock().unwrap();
        *count = 0;
    }
}

#[async_trait::async_trait]
impl ValidatorTrait for MockValidator {
    async fn validate_and_return_expected_hash(&self, msg: &[u8]) -> Result<Digest> {
        {
            let mut count = self.validation_count.lock().unwrap();
            *count += 1;
        }

        if !self.should_succeed {
            let error_msg = self
                .error_message
                .clone()
                .unwrap_or_else(|| "Mock validation failed".to_string());
            return Err(anyhow::anyhow!(error_msg));
        }

        // For successful validation, delegate to get_payload_from_message
        self.get_payload_from_message(msg).await
    }

    async fn get_payload_from_message(&self, _msg: &[u8]) -> Result<Digest> {
        if !self.should_succeed {
            let error_msg = self
                .error_message
                .clone()
                .unwrap_or_else(|| "Mock payload extraction failed".to_string());
            return Err(anyhow::anyhow!(error_msg));
        }

        // For successful extraction, create a predictable hash based on the expected round
        let payload = U256::from(self.expected_round).abi_encode();
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        let payload_hash = hasher.finalize();

        Ok(payload_hash)
    }
}
