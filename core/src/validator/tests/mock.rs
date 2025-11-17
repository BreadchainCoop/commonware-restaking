use alloy::sol_types::SolValue;
use anyhow::Result;
use commonware_cryptography::sha256::Digest;
use commonware_cryptography::{Hasher, Sha256};
use std::sync::{Arc, Mutex};

use crate::validator::interface::ValidatorTrait;

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
    pub fn new_success(expected_round: u64) -> Self {
        Self {
            expected_round,
            should_succeed: true,
            error_message: None,
            validation_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn new_failure(error_message: String) -> Self {
        Self {
            expected_round: 0,
            should_succeed: false,
            error_message: Some(error_message),
            validation_count: Arc::new(Mutex::new(0)),
        }
    }

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

    pub fn set_expected_round(&mut self, round: u64) {
        self.expected_round = round;
    }

    pub fn set_should_succeed(&mut self, should_succeed: bool) {
        self.should_succeed = should_succeed;
    }

    pub fn set_error_message(&mut self, error_message: Option<String>) {
        self.error_message = error_message;
    }

    pub fn get_validation_count(&self) -> u64 {
        *self.validation_count.lock().unwrap()
    }

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

        let payload = alloy::primitives::U256::from(self.expected_round).abi_encode();
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        let payload_hash = hasher.finalize();

        Ok(payload_hash)
    }
}
