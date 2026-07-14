use anyhow::Result;
use commonware_codec::Encode;
use commonware_cryptography::sha256::Digest;
use commonware_cryptography::{Hasher, Sha256};
use std::sync::{Arc, Mutex};

use crate::validator::ValidatorTrait;
use crate::wire::TaskData;

/// Mock validator for testing: computes a predictable digest (sha256 of the task's
/// encoded bytes) without any external dependency, with configurable failure
/// behavior and an attempt counter.
pub struct MockValidator {
    /// Whether validation should succeed or fail
    should_succeed: bool,
    /// Custom error message to return on failure
    error_message: Option<String>,
    /// Counter for tracking validation attempts
    validation_count: Arc<Mutex<u64>>,
}

impl MockValidator {
    /// Creates a MockValidator that accepts any task and returns the sha256 of its
    /// encoded bytes.
    pub fn new_success() -> Self {
        Self {
            should_succeed: true,
            error_message: None,
            validation_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Creates a MockValidator that rejects every task with `error_message`.
    pub fn new_failure(error_message: String) -> Self {
        Self {
            should_succeed: false,
            error_message: Some(error_message),
            validation_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Updates the success/failure behavior.
    pub fn set_should_succeed(&mut self, should_succeed: bool) {
        self.should_succeed = should_succeed;
    }

    /// Updates the error message for failure scenarios.
    pub fn set_error_message(&mut self, error_message: Option<String>) {
        self.error_message = error_message;
    }

    /// Number of times `expected_digest` was called.
    pub fn get_validation_count(&self) -> u64 {
        *self.validation_count.lock().unwrap()
    }

    /// Resets the validation counter between test scenarios.
    pub fn reset_validation_count(&mut self) {
        let mut count = self.validation_count.lock().unwrap();
        *count = 0;
    }

    /// The digest this mock returns for `task` — sha256 of the task's encoding.
    pub fn digest_for<T: TaskData>(task: &T) -> Digest {
        let mut hasher = Sha256::new();
        hasher.update(&task.encode());
        hasher.finalize()
    }
}

#[async_trait::async_trait]
impl<T: TaskData> ValidatorTrait<T> for MockValidator {
    async fn expected_digest(&self, task: &T) -> Result<Digest> {
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

        Ok(Self::digest_for(task))
    }
}
