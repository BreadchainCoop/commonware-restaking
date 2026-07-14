//! [`TaskSource`] implementation polling the deployed `Counter` contract.
//!
//! The sequencer only calls [`TaskSource::next_task`] once the previous height has
//! fully resolved (certificate + execution), which makes polling self-healing: a
//! successful `increment` advances `Counter.number()`, so the next task carries the
//! new round; a skipped or failed height leaves the round unchanged, so the same
//! round is legitimately re-issued.

use async_trait::async_trait;
use commonware_avs_router::sequencer::{SequencedTask, TaskSource};
use counter_common::types::CounterTaskData;
use counter_common::validator::expected_digest;
use std::time::Duration;
use tracing::warn;

use crate::provider::CounterProvider;

/// Polls the `Counter` contract for its current round and hands it to the
/// sequencer as the next task to assign.
pub struct CounterTaskSource {
    provider: CounterProvider,
    poll_interval: Duration,
}

impl CounterTaskSource {
    pub fn new(provider: CounterProvider, poll_interval: Duration) -> Self {
        Self {
            provider,
            poll_interval,
        }
    }

    /// Reads `Counter.number()`, retrying (with a log) on RPC error.
    async fn read_round(&self) -> u64 {
        loop {
            match self.provider.get_current_round().await {
                Ok(round) => return round,
                Err(error) => {
                    warn!(%error, "failed to read counter round; retrying");
                    tokio::time::sleep(self.poll_interval).await;
                }
            }
        }
    }
}

#[async_trait]
impl TaskSource<CounterTaskData> for CounterTaskSource {
    async fn next_task(&mut self) -> Option<SequencedTask<CounterTaskData>> {
        // Give the previous height's on-chain effect (or lack thereof) a moment to
        // settle before reading the round again.
        tokio::time::sleep(self.poll_interval).await;
        let round = self.read_round().await;
        let task = CounterTaskData { round };
        Some(SequencedTask {
            announce: task.clone(),
            digest: expected_digest(round),
            task,
        })
    }
}
