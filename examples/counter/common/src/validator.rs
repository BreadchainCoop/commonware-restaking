use alloy::sol_types::SolValue;
use alloy_primitives::U256;
use alloy_provider::ProviderBuilder;
use anyhow::Result;
use commonware_avs_bindings::ReadOnlyProvider;
use commonware_avs_core::validator::ValidatorTrait;
use commonware_cryptography::sha256::Digest;
use commonware_cryptography::{Hasher, Sha256};
use std::env;

use crate::config::CounterDeployment;
use crate::types::CounterTaskData;
use counter_bindings::Counter;

/// The digest a correct node signs for `round`: `sha256(abi_encode(U256::from(round)))`.
///
/// MUST stay byte-identical to what the deployed `Counter` contract recomputes
/// on-chain when BLS-verifying an `increment` call — the preimage is exactly the
/// abi-encoded round number, sha256-hashed, with no other domain separation. Do
/// not change this formula without a corresponding contract upgrade; it is shared
/// with the router's task source so the announced digest matches what nodes sign.
pub fn expected_digest(round: u64) -> Digest {
    let payload = U256::from(round).abi_encode();
    let mut hasher = Sha256::new();
    hasher.update(&payload);
    hasher.finalize()
}

/// Validates a [`CounterTaskData`] against the deployed `Counter` contract: the
/// task's round must equal the contract's current `number()`.
pub struct CounterValidator {
    counter: Counter::CounterInstance<ReadOnlyProvider, alloy::network::Ethereum>,
}

impl CounterValidator {
    pub async fn new() -> Result<Self> {
        let http_rpc = env::var("HTTP_RPC").expect("HTTP_RPC must be set");
        let provider = ProviderBuilder::new().connect_http(
            url::Url::parse(&http_rpc)
                .map_err(|e| anyhow::anyhow!("Failed to parse RPC URL '{}': {}", http_rpc, e))?,
        );

        let deployment = CounterDeployment::load()
            .map_err(|e| anyhow::anyhow!("Failed to load AVS deployment: {}", e))?;
        let counter_address = deployment
            .counter_address()
            .map_err(|e| anyhow::anyhow!("Failed to get counter address: {}", e))?;
        let counter = Counter::new(counter_address, provider);

        Ok(Self { counter })
    }
}

#[async_trait::async_trait]
impl ValidatorTrait<CounterTaskData> for CounterValidator {
    /// Rejects a task whose round does not match the contract's current
    /// `number()`. This is retryable — the node's automaton retries within its
    /// budget, and a task that never catches up eventually resolves as a skip.
    async fn expected_digest(&self, task: &CounterTaskData) -> Result<Digest> {
        let current_number = self.counter.number().call().await?;
        let current_number = current_number.to::<u64>();
        if task.round != current_number {
            return Err(anyhow::anyhow!(
                "stale round: task carries {}, contract is at {}",
                task.round,
                current_number
            ));
        }
        Ok(expected_digest(task.round))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ON-CHAIN PARITY ANCHOR: the digest formula must stay byte-identical to an
    // independently computed sha256(abi_encode(U256::from(round))) — the preimage
    // the deployed Counter contract's `increment` recomputes when BLS-verifying a
    // certificate. If this test breaks, do not "fix" it by changing the formula.
    #[test]
    fn digest_matches_independent_abi_encoding() {
        let round = 12_345u64;
        let independently_computed = {
            let payload = U256::from(round).abi_encode();
            let mut hasher = Sha256::new();
            hasher.update(&payload);
            hasher.finalize()
        };
        assert_eq!(expected_digest(round), independently_computed);
    }

    #[test]
    fn digest_is_deterministic_and_distinct_per_round() {
        assert_eq!(expected_digest(7), expected_digest(7));
        assert_ne!(expected_digest(7), expected_digest(8));
    }
}
