// Core traits and types
pub mod core;
pub mod queue;

pub use queue::{CreatorConfig, SimpleTaskQueue, TaskQueue};

// Test module
#[cfg(test)]
pub mod tests;

// Re-export test utilities
#[cfg(test)]
#[allow(unused_imports)]
pub use tests::mock::MockCreator;
