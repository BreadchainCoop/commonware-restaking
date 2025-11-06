use anyhow::Result;
use async_trait::async_trait;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::executor::core::{ExecutionResult, VerificationData, VerificationExecutor};

/// Mock executor implementation for testing purposes.
///
/// This implementation provides a configurable mock that can be used
/// for unit testing without requiring real execution logic. It allows
/// for predictable behavior and easy test scenario setup.
#[allow(dead_code)]
pub struct MockExecutor<T = ()>
where
    T: Send + Sync,
{
    /// Counter for tracking execution attempts
    execution_count: Arc<Mutex<u64>>,
    /// Whether execution should succeed or fail
    should_succeed: bool,
    /// Custom error message to return on failure
    error_message: Option<String>,
    /// Custom execution result to return on success
    custom_result: Option<ExecutionResult>,
    /// Phantom data for the task data type
    _phantom: PhantomData<T>,
}

#[allow(dead_code)]
impl<T> MockExecutor<T>
where
    T: Send + Sync,
{
    /// Creates a new MockExecutor that always succeeds.
    ///
    /// This constructor creates a mock executor that will accept
    /// any verification request and return a default success result.
    ///
    /// # Returns
    /// * `Self` - The new MockExecutor instance
    pub fn new() -> Self {
        Self {
            execution_count: Arc::new(Mutex::new(0)),
            should_succeed: true,
            error_message: None,
            custom_result: None,
            _phantom: PhantomData,
        }
    }

    /// Creates a new MockExecutor with success/failure configuration.
    ///
    /// This constructor allows for basic success/failure control.
    ///
    /// # Arguments
    /// * `should_succeed` - Whether execution should succeed or fail
    ///
    /// # Returns
    /// * `Self` - The new MockExecutor instance
    pub fn with_success(mut self, should_succeed: bool) -> Self {
        self.should_succeed = should_succeed;
        self
    }

    /// Creates a new MockExecutor that always fails.
    ///
    /// This constructor creates a mock executor that will fail
    /// execution with a custom error message.
    ///
    /// # Arguments
    /// * `error_message` - The error message to return on failure
    ///
    /// # Returns
    /// * `Self` - The new MockExecutor instance
    pub fn new_failure(error_message: String) -> Self {
        Self {
            execution_count: Arc::new(Mutex::new(0)),
            should_succeed: false,
            error_message: Some(error_message),
            custom_result: None,
            _phantom: PhantomData,
        }
    }

    /// Creates a new MockExecutor with custom configuration.
    ///
    /// This constructor allows for fine-grained control over the mock's behavior.
    ///
    /// # Arguments
    /// * `should_succeed` - Whether execution should succeed or fail
    /// * `error_message` - Optional error message for failure scenarios
    /// * `custom_result` - Optional custom result for success scenarios
    ///
    /// # Returns
    /// * `Self` - The new MockExecutor instance
    pub fn new_custom(
        should_succeed: bool,
        error_message: Option<String>,
        custom_result: Option<ExecutionResult>,
    ) -> Self {
        Self {
            execution_count: Arc::new(Mutex::new(0)),
            should_succeed,
            error_message,
            custom_result,
            _phantom: PhantomData,
        }
    }

    /// Updates the success/failure behavior.
    ///
    /// This method allows changing whether execution should
    /// succeed or fail during test execution.
    ///
    /// # Arguments
    /// * `should_succeed` - Whether execution should succeed
    pub fn set_should_succeed(&mut self, should_succeed: bool) {
        self.should_succeed = should_succeed;
    }

    /// Updates the error message for failure scenarios.
    ///
    /// This method allows changing the error message that
    /// will be returned on execution failure.
    ///
    /// # Arguments
    /// * `error_message` - The new error message
    pub fn set_error_message(&mut self, error_message: Option<String>) {
        self.error_message = error_message;
    }

    /// Updates the custom result for success scenarios.
    ///
    /// This method allows changing the result that
    /// will be returned on successful execution.
    ///
    /// # Arguments
    /// * `custom_result` - The new custom result
    pub fn set_custom_result(&mut self, custom_result: Option<ExecutionResult>) {
        self.custom_result = custom_result;
    }

    /// Gets the current execution count.
    ///
    /// This method is useful for testing to verify how many times
    /// execution was attempted.
    ///
    /// # Returns
    /// * `u64` - The current execution count
    pub fn get_execution_count(&self) -> u64 {
        *self.execution_count.lock().unwrap()
    }

    /// Resets the execution count to zero.
    ///
    /// This method is useful for testing to reset the counter
    /// between test scenarios.
    pub fn reset_execution_count(&mut self) {
        let mut count = self.execution_count.lock().unwrap();
        *count = 0;
    }
}

#[async_trait]
impl<T> VerificationExecutor<T> for MockExecutor<T>
where
    T: Send + Sync,
{
    async fn execute_verification(
        &mut self,
        _digest: &[u8],
        _verification_data: VerificationData,
        _task_data: Option<&T>,
    ) -> Result<ExecutionResult> {
        let mut count = self.execution_count.lock().unwrap();
        *count += 1;

        if self.should_succeed {
            if let Some(custom_result) = &self.custom_result {
                Ok(custom_result.clone())
            } else {
                Ok(ExecutionResult {
                    transaction_hash: "mock_tx_hash".to_string(),
                    block_number: Some(12345),
                    gas_used: Some(100000),
                    status: Some(true),
                    contract_address: Some("mock_contract".to_string()),
                })
            }
        } else {
            let error_msg = self
                .error_message
                .clone()
                .unwrap_or_else(|| "Mock execution failed".to_string());
            Err(anyhow::anyhow!(error_msg))
        }
    }
}

impl<T> Default for MockExecutor<T>
where
    T: Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}
