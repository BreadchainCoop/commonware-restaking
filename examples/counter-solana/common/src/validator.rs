//! Digest formula + node-side validator for the Solana counter example.

use anyhow::Result;
use commonware_avs_core::validator::ValidatorTrait;
use commonware_cryptography::sha256::Digest;
use commonware_cryptography::{Hasher, Sha256};
use solana_sdk::pubkey::Pubkey;

use crate::types::RoundTaskData;

/// Domain tag of the demo digest preimage (versioned).
const DIGEST_DOMAIN: &[u8] = b"COUNTER-SOLANA-TASK-V1";

/// The digest a correct node signs for `round` against `ncn`:
/// `sha256(domain ‖ ncn ‖ round_le)`.
///
/// Binding the NCN pubkey into the preimage keeps certificates from one
/// deployment meaningless to another; the round makes each task's digest
/// unique. This is shared by the router's task source and the nodes'
/// validator so the announced digest matches what nodes sign — and it is the
/// exact 32-byte `MessageDigest` the NCN program's `VerifyCertificate`
/// verifies the aggregate signature over.
pub fn expected_digest(ncn: &Pubkey, round: u64) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(DIGEST_DOMAIN);
    hasher.update(ncn.to_bytes().as_slice());
    hasher.update(&round.to_le_bytes());
    hasher.finalize()
}

/// Validates a [`RoundTaskData`] by recomputing the digest from the node's own
/// deployment config (never trusting the router's bytes).
///
/// The demo consumer is stateless, so there is no on-chain round to
/// cross-check (the EVM example polls `Counter.number()`); once the
/// settlement program lands, this is where its `transition_count` read plugs
/// in.
pub struct RoundValidator {
    ncn: Pubkey,
}

impl RoundValidator {
    pub fn new(ncn: Pubkey) -> Self {
        Self { ncn }
    }
}

#[async_trait::async_trait]
impl ValidatorTrait<RoundTaskData> for RoundValidator {
    async fn expected_digest(&self, task: &RoundTaskData) -> Result<Digest> {
        Ok(expected_digest(&self.ncn, task.round))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_is_deterministic_and_distinct_per_round_and_ncn() {
        let ncn_a = Pubkey::new_unique();
        let ncn_b = Pubkey::new_unique();
        assert_eq!(expected_digest(&ncn_a, 7), expected_digest(&ncn_a, 7));
        assert_ne!(expected_digest(&ncn_a, 7), expected_digest(&ncn_a, 8));
        assert_ne!(expected_digest(&ncn_a, 7), expected_digest(&ncn_b, 7));
    }

    #[test]
    fn validator_recomputes_the_shared_formula() {
        let ncn = Pubkey::new_unique();
        let validator = RoundValidator::new(ncn);
        let task = RoundTaskData { round: 99 };
        let digest = futures_digest(&validator, &task);
        assert_eq!(digest, expected_digest(&ncn, 99));
    }

    /// Minimal sync driver for the async trait method in tests.
    fn futures_digest(validator: &RoundValidator, task: &RoundTaskData) -> Digest {
        futures_lite_block_on(ValidatorTrait::expected_digest(validator, task))
            .expect("validation is infallible for the demo")
    }

    /// Tiny block_on to avoid a runtime dev-dependency: the future is
    /// immediately ready (no I/O in the validator).
    fn futures_lite_block_on<F: Future>(future: F) -> F::Output {
        use std::pin::pin;
        use std::task::{Context, Poll, Waker};
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        let mut future = pin!(future);
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(output) => output,
            Poll::Pending => unreachable!("validator future is immediately ready"),
        }
    }
}
