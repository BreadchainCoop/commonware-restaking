#[cfg(test)]
pub mod tests;

pub mod traits;
pub mod types;

pub use traits::{Contribute, ContributorBase};
pub use types::AggregationInput;
