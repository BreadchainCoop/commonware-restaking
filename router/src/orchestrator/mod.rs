//! Generic orchestrator module for the commonware-avs-router.
//!
//! This module provides a generic interface for orchestration behavior,
//! allowing different implementations to be swapped without changing
//! the consuming code.

// Public modules
pub mod builder;
pub mod generic;
pub mod interface;

// Test module
#[cfg(test)]
pub mod tests;

// Re-export main types for easy access
pub use builder::OrchestratorBuilder;
