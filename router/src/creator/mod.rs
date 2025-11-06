// Core traits and types
pub mod core;

// Test module
#[cfg(test)]
pub mod tests;

// Re-export test utilities
#[cfg(test)]
#[allow(unused_imports)]
pub use tests::mock::MockCreator;
