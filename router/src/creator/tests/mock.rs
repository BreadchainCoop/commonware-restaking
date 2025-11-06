use anyhow::Result;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::creator::core::Creator;

/// Mock creator implementation for testing purposes.
///
/// This implementation provides a configurable mock that can be used
/// for unit testing without requiring real task creation logic. It allows
/// for predictable behavior and easy test scenario setup.
#[allow(dead_code)]
pub struct MockCreator<T> {
    /// Counter for generating sequential round numbers
    round_counter: Arc<Mutex<u64>>,
    /// Configurable metadata to return
    metadata: T,
    /// Whether task creation should succeed or fail
    should_succeed: bool,
    /// Custom error message to return on failure
    error_message: Option<String>,
}

#[allow(dead_code)]
impl<T> MockCreator<T>
where
    T: Clone + Default,
{
    /// Creates a new MockCreator that always succeeds.
    ///
    /// This constructor creates a mock creator that will generate
    /// sequential round numbers and return configurable metadata.
    ///
    /// # Returns
    /// * `Self` - The new MockCreator instance
    pub fn new() -> Self {
        Self {
            round_counter: Arc::new(Mutex::new(0)),
            metadata: T::default(),
            should_succeed: true,
            error_message: None,
        }
    }

    /// Creates a new MockCreator with custom metadata.
    ///
    /// This constructor allows for custom metadata configuration.
    ///
    /// # Arguments
    /// * `metadata` - The metadata to return from get_task_metadata
    ///
    /// # Returns
    /// * `Self` - The new MockCreator instance
    pub fn with_metadata(mut self, metadata: T) -> Self {
        self.metadata = metadata;
        self
    }

    /// Creates a new MockCreator that always fails.
    ///
    /// This constructor creates a mock creator that will fail
    /// task creation with a custom error message.
    ///
    /// # Arguments
    /// * `error_message` - The error message to return on failure
    ///
    /// # Returns
    /// * `Self` - The new MockCreator instance
    pub fn new_failure(error_message: String) -> Self {
        Self {
            round_counter: Arc::new(Mutex::new(0)),
            metadata: T::default(),
            should_succeed: false,
            error_message: Some(error_message),
        }
    }

    /// Creates a new MockCreator with custom configuration.
    ///
    /// This constructor allows for fine-grained control over the mock's behavior.
    ///
    /// # Arguments
    /// * `metadata` - The metadata to return from get_task_metadata
    /// * `should_succeed` - Whether task creation should succeed or fail
    /// * `error_message` - Optional error message for failure scenarios
    ///
    /// # Returns
    /// * `Self` - The new MockCreator instance
    pub fn new_custom(metadata: T, should_succeed: bool, error_message: Option<String>) -> Self {
        Self {
            round_counter: Arc::new(Mutex::new(0)),
            metadata,
            should_succeed,
            error_message,
        }
    }

    /// Updates the metadata.
    ///
    /// This method allows changing the metadata during test execution.
    ///
    /// # Arguments
    /// * `metadata` - The new metadata
    pub fn set_metadata(&mut self, metadata: T) {
        self.metadata = metadata;
    }

    /// Updates the success/failure behavior.
    ///
    /// This method allows changing whether task creation should
    /// succeed or fail during test execution.
    ///
    /// # Arguments
    /// * `should_succeed` - Whether task creation should succeed
    pub fn set_should_succeed(&mut self, should_succeed: bool) {
        self.should_succeed = should_succeed;
    }

    /// Updates the error message for failure scenarios.
    ///
    /// This method allows changing the error message that
    /// will be returned on task creation failure.
    ///
    /// # Arguments
    /// * `error_message` - The new error message
    pub fn set_error_message(&mut self, error_message: Option<String>) {
        self.error_message = error_message;
    }

    /// Gets the current round counter value.
    ///
    /// This method is useful for testing to verify the round progression.
    ///
    /// # Returns
    /// * `u64` - The current round counter value
    pub fn get_round_counter(&self) -> u64 {
        *self.round_counter.lock().unwrap()
    }
}

#[async_trait]
impl<T> Creator for MockCreator<T>
where
    T: Clone
        + Default
        + Send
        + Sync
        + commonware_codec::Write
        + commonware_codec::Read<Cfg = ()>
        + commonware_codec::EncodeSize,
{
    type TaskData = T;

    async fn get_payload_and_round(&self) -> Result<(Vec<u8>, u64)> {
        if !self.should_succeed {
            let error_msg = self
                .error_message
                .clone()
                .unwrap_or_else(|| "Mock task creation failed".to_string());
            return Err(anyhow::anyhow!(error_msg));
        }

        let mut counter = self.round_counter.lock().unwrap();
        *counter += 1;
        let round = *counter;

        // Create a simple payload: round number as bytes
        let payload = round.to_le_bytes().to_vec();
        Ok((payload, round))
    }

    fn get_task_metadata(&self) -> Self::TaskData {
        self.metadata.clone()
    }
}

impl<T> Default for MockCreator<T>
where
    T: Clone + Default,
{
    fn default() -> Self {
        Self::new()
    }
}
