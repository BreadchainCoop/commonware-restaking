use alloy::{primitives::U256, sol_types::SolValue};
use anyhow::Result;

use commonware_avs_shared::bindings::{WalletProvider as AlloyProvider, counter::Counter};

/// Concrete provider for the counter usecase.
pub struct CounterProvider {
    counter: Counter::CounterInstance<(), AlloyProvider>,
}

impl CounterProvider {
    pub fn new(counter_address: alloy::primitives::Address, provider: AlloyProvider) -> Self {
        let counter = Counter::new(counter_address, provider);
        Self { counter }
    }

    /// Reads the current on-chain number as the round.
    pub async fn get_current_round(&self) -> Result<u64> {
        let current = self.counter.number().call().await?;
        Ok(current._0.to::<u64>())
    }

    /// Encodes the round into ABI-encoded bytes for hashing/signing.
    pub fn encode_round(&self, round: u64) -> Vec<u8> {
        U256::from(round).abi_encode()
    }
}
