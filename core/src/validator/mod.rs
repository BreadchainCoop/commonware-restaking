use anyhow::Result;
use commonware_cryptography::sha256::Digest;

/// Application hook for independently validating an announced task.
///
/// Every node recomputes the digest for a task from its own view of the world
/// (rather than trusting anything the router announced) and signs only what it
/// computed itself. A router uses the same implementation to know which digest a
/// certificate for the task must carry.
///
/// Implementations should return an error when the task cannot be validated (yet) —
/// callers treat errors as retryable until their own budget expires, after which the
/// height is skipped rather than signed.
#[async_trait::async_trait]
pub trait ValidatorTrait<T>: Send + Sync {
    /// Validates `task` and returns the digest this participant is willing to sign
    /// for it.
    async fn expected_digest(&self, task: &T) -> Result<Digest>;
}

#[cfg(any(test, feature = "test-utils"))]
pub mod tests;

#[cfg(any(test, feature = "test-utils"))]
pub use tests::mock::MockValidator;
