//! Executor module for the commonware-avs-router.

// Public modules
pub mod bls;
pub mod core;

// Test module
#[cfg(test)]
pub mod tests;

// Re-export the main types for easy access
#[allow(unused_imports)]
pub use bls::{BlsEigenlayerExecutor, convert_non_signer_data};
#[allow(unused_imports)]
pub use core::{ExecutionResult, VerificationData, VerificationExecutor};

// Re-export test utilities
#[cfg(test)]
#[allow(unused_imports)]
pub use tests::mock::MockExecutor;
