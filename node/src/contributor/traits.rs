use std::hash::Hash;

use anyhow::Result;
use commonware_cryptography::{PublicKey, Signer};
use commonware_p2p::{Receiver, Sender};

/// Base trait for common contributor functionality
pub trait ContributorBase {
    type PublicKey: PublicKey + Ord + Eq + Hash + Clone;
    type Signer: Signer<PublicKey = Self::PublicKey>;
    type Signature: Clone;

    // Common functionality
    fn is_orchestrator(&self, sender: &Self::PublicKey) -> bool;
    fn get_contributor_index(&self, public_key: &Self::PublicKey) -> Option<&usize>;
}

/// Main contributor trait that extends the base
pub trait Contribute: ContributorBase {
    type AggregationInput;

    fn new(
        orchestrator: Self::PublicKey,
        signer: Self::Signer,
        contributors: Vec<Self::PublicKey>,
        aggregation_data: Option<Self::AggregationInput>,
    ) -> Self;

    async fn run<S, R>(self, sender: S, receiver: R) -> Result<()>
    where
        S: Sender,
        R: Receiver<PublicKey = Self::PublicKey>;
}
