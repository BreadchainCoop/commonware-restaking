//! Validator module for the commonware-avs-router.

// Public modules
pub mod generic;
pub mod interface;

// Test utilities module (available in shared crate tests and when feature `test-utils` is enabled)
#[cfg(any(test, feature = "test-utils"))]
pub mod tests;

// Re-export the main types for easy access
#[allow(unused_imports)]
pub use generic::Validator;
#[allow(unused_imports)]
pub use interface::ValidatorTrait;

// Re-export test utilities
#[cfg(any(test, feature = "test-utils"))]
#[allow(unused_imports)]
pub use tests::mock::MockValidator;
