use alloy::network::Ethereum;
use anyhow::Result;
use commonware_avs_bindings::ReadOnlyProvider;
use counter_bindings::Counter;

/// Thin wrapper over the deployed `Counter` contract's read-only calls, used by
/// [`crate::source::CounterTaskSource`] to poll for the next round.
pub struct CounterProvider {
    counter: Counter::CounterInstance<ReadOnlyProvider, Ethereum>,
}

impl CounterProvider {
    pub fn new(counter_address: alloy::primitives::Address, provider: ReadOnlyProvider) -> Self {
        let counter = Counter::new(counter_address, provider);
        Self { counter }
    }

    /// Reads the contract's current round (`Counter.number()`).
    pub async fn get_current_round(&self) -> Result<u64> {
        let current = self.counter.number().call().await?;
        Ok(current.to::<u64>())
    }
}
