//! [`TaskSource`] implementation for the Solana counter demo.
//!
//! The EVM example polls `Counter.number()` between rounds; the Solana demo's
//! consumer is the NCN program's STATELESS `VerifyCertificate` (the demo
//! VoteCounter moves out of the NCN program in Phase 1 — INTERFACES.md §2), so
//! there is no on-chain round to poll yet. Rounds are locally monotonic; the
//! digest binds `(domain, ncn, round)`, and replay safety is consumer-local by
//! design (D-MSG) — a re-verified digest mutates nothing. When the settlement
//! program (§4) lands, this source is where its `transition_count` read plugs
//! in, restoring the EVM example's self-healing poll loop.

use async_trait::async_trait;
use commonware_avs_router::sequencer::{SequencedTask, TaskSource};
use counter_solana_common::{RoundTaskData, expected_digest};
use solana_sdk::pubkey::Pubkey;
use std::time::Duration;

/// Hands monotonically increasing rounds to the sequencer, one per poll
/// interval.
pub struct RoundSource {
    ncn: Pubkey,
    next_round: u64,
    poll_interval: Duration,
}

impl RoundSource {
    pub fn new(ncn: Pubkey, start_round: u64, poll_interval: Duration) -> Self {
        Self {
            ncn,
            next_round: start_round,
            poll_interval,
        }
    }
}

#[async_trait]
impl TaskSource<RoundTaskData> for RoundSource {
    async fn next_task(&mut self) -> Option<SequencedTask<RoundTaskData>> {
        // Give the previous height's submission a moment to settle before
        // issuing the next round (mirrors the EVM example's pacing).
        tokio::time::sleep(self.poll_interval).await;
        let round = self.next_round;
        self.next_round = self.next_round.checked_add(1)?;
        let task = RoundTaskData { round };
        Some(SequencedTask {
            announce: task.clone(),
            digest: expected_digest(&self.ncn, round),
            task,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rounds_are_monotonic_and_digests_match_the_shared_formula() {
        let ncn = Pubkey::new_unique();
        let mut source = RoundSource::new(ncn, 5, Duration::from_millis(1));
        for expected_round in 5..8 {
            let task = source.next_task().await.expect("source yields");
            assert_eq!(task.task.round, expected_round);
            assert_eq!(task.announce, task.task);
            assert_eq!(task.digest, expected_digest(&ncn, expected_round));
        }
    }

    #[tokio::test]
    async fn source_terminates_on_round_overflow() {
        let ncn = Pubkey::new_unique();
        let mut source = RoundSource::new(ncn, u64::MAX, Duration::from_millis(1));
        assert!(source.next_task().await.is_none());
    }
}
