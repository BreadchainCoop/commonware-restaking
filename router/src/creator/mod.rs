// Core traits and types
pub mod queue;
pub mod traits;

pub use queue::{CreatorConfig, SimpleTaskQueue, TaskQueue};
pub use traits::Creator;

// Test module
#[cfg(test)]
pub mod tests;

// Re-export test utilities
#[cfg(test)]
#[allow(unused_imports)]
pub use tests::mock::MockCreator;
