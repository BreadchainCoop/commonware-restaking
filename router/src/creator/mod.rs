// Core traits and types
pub mod queue;
pub mod traits;
pub mod types;

pub use queue::{SimpleTaskQueue, TaskQueue};
pub use traits::Creator;
pub use types::CreatorConfig;

// Test module
#[cfg(test)]
pub mod tests;

// Re-export test utilities
#[cfg(test)]
pub use tests::mock::MockCreator;
