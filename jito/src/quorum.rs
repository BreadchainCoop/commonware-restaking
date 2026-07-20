//! Startup quorum reconciliation (INTERFACES.md §5).
//!
//! The chassis aggregation engine certifies with a COUNT-based N3f1 quorum
//! (`n - (n-1)/3` signers); the NCN program verifies with a STAKE-weighted
//! threshold (`signed_stake_bps >= consensus_threshold_bps`). Those are two
//! different quorum definitions, so a certificate the engine assembles is not
//! automatically submittable.
//!
//! The reconciliation rule: **the minimum total stake over all `(n - f)`-sized
//! signer subsets must clear the on-chain threshold** — i.e. even the lightest
//! engine quorum (the `n - f` smallest stakes) carries enough stake. If it
//! does not, the router must refuse to start rather than assemble
//! certificates the chain will reject.

use num::BigUint;

/// Basis-point denominator.
const BPS_DENOMINATOR: u64 = 10_000;

/// Why a participant set fails reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuorumError {
    /// No participants.
    Empty,
    /// The lightest engine quorum's stake is below the on-chain threshold.
    InsufficientLightestQuorum {
        /// Stake carried by the `n - f` smallest stakes.
        lightest_quorum_stake: u128,
        /// Total stake over all participants.
        total_stake: u128,
        /// The on-chain threshold the lightest quorum must clear.
        threshold_bps: u64,
        /// The lightest quorum's stake in bps of total (floor).
        lightest_quorum_bps: u64,
    },
    /// Total stake is zero (nothing snapshotted yet).
    ZeroTotalStake,
}

impl std::fmt::Display for QuorumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuorumError::Empty => write!(f, "participant set is empty"),
            QuorumError::ZeroTotalStake => {
                write!(f, "total snapshot stake is zero; run the snapshot keeper")
            }
            QuorumError::InsufficientLightestQuorum {
                lightest_quorum_stake,
                total_stake,
                threshold_bps,
                lightest_quorum_bps,
            } => write!(
                f,
                "lightest engine quorum carries {lightest_quorum_stake} of {total_stake} stake \
                 ({lightest_quorum_bps} bps) < consensus threshold {threshold_bps} bps; \
                 an engine-certified certificate could fail on-chain — refusing to start"
            ),
        }
    }
}

impl std::error::Error for QuorumError {}

/// Successful reconciliation summary (for logging).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuorumReconciliation {
    pub participants: usize,
    pub max_faults: usize,
    pub engine_quorum: usize,
    pub total_stake: u128,
    pub lightest_quorum_stake: u128,
    pub lightest_quorum_bps: u64,
    pub threshold_bps: u64,
}

/// Engine fault bound `f = (n - 1) / 3` (N3f1).
pub fn max_faults(participants: usize) -> usize {
    participants.saturating_sub(1) / 3
}

/// Stake carried by the lightest possible engine quorum: the sum of the
/// `n - f` SMALLEST stakes (any engine quorum contains `n - f` distinct
/// participants, so the minimum over all subsets is the ascending prefix sum).
pub fn lightest_quorum_stake(stakes: &[u128]) -> Option<u128> {
    if stakes.is_empty() {
        return None;
    }
    let quorum = stakes.len() - max_faults(stakes.len());
    let mut sorted = stakes.to_vec();
    sorted.sort_unstable();
    sorted
        .iter()
        .take(quorum)
        .try_fold(0u128, |acc, stake| acc.checked_add(*stake))
}

/// Asserts that the lightest engine quorum clears `threshold_bps` of total
/// stake. Call at startup with the stakes of the FULL participant set the
/// engine will run with (INTERFACES.md §5: refuse to start on failure).
pub fn reconcile(stakes: &[u128], threshold_bps: u64) -> Result<QuorumReconciliation, QuorumError> {
    if stakes.is_empty() {
        return Err(QuorumError::Empty);
    }
    // Totals can exceed u128 in adversarial configs; do the exact comparison in
    // arbitrary precision (this runs once at startup).
    let total: BigUint = stakes.iter().map(|s| BigUint::from(*s)).sum();
    if total == BigUint::ZERO {
        return Err(QuorumError::ZeroTotalStake);
    }
    let lightest = lightest_quorum_stake(stakes)
        .map(BigUint::from)
        .unwrap_or_else(|| {
            // checked_add overflowed u128: recompute exactly.
            let quorum = stakes.len() - max_faults(stakes.len());
            let mut sorted = stakes.to_vec();
            sorted.sort_unstable();
            sorted.iter().take(quorum).map(|s| BigUint::from(*s)).sum()
        });

    // lightest / total >= threshold_bps / 10_000, cross-multiplied.
    let clears =
        &lightest * BigUint::from(BPS_DENOMINATOR) >= &total * BigUint::from(threshold_bps);
    let lightest_quorum_bps: u64 = ((&lightest * BigUint::from(BPS_DENOMINATOR)) / &total)
        .try_into()
        .unwrap_or(u64::MAX);
    let lightest_u128: u128 = lightest.clone().try_into().unwrap_or(u128::MAX);
    let total_u128: u128 = total.clone().try_into().unwrap_or(u128::MAX);

    if !clears {
        return Err(QuorumError::InsufficientLightestQuorum {
            lightest_quorum_stake: lightest_u128,
            total_stake: total_u128,
            threshold_bps,
            lightest_quorum_bps,
        });
    }

    Ok(QuorumReconciliation {
        participants: stakes.len(),
        max_faults: max_faults(stakes.len()),
        engine_quorum: stakes.len() - max_faults(stakes.len()),
        total_stake: total_u128,
        lightest_quorum_stake: lightest_u128,
        lightest_quorum_bps,
        threshold_bps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_weights_clear_default_threshold() {
        // n=4, f=1: lightest quorum = 3/4 = 7500 bps >= 6667.
        let result = reconcile(&[100, 100, 100, 100], 6667).expect("clears");
        assert_eq!(result.engine_quorum, 3);
        assert_eq!(result.lightest_quorum_stake, 300);
        assert_eq!(result.lightest_quorum_bps, 7500);
    }

    #[test]
    fn n3_all_signers_always_clear() {
        // n=3, f=0: the lightest quorum is everyone — 10000 bps.
        let result = reconcile(&[1, 2, 3], 9999).expect("clears");
        assert_eq!(result.engine_quorum, 3);
        assert_eq!(result.lightest_quorum_bps, 10_000);
    }

    #[test]
    fn skewed_weights_refuse_to_start() {
        // n=4, f=1: one whale holds ~97% — dropping it leaves 3/103 of stake.
        let error = reconcile(&[1, 1, 1, 100], 6667).expect_err("must refuse");
        match error {
            QuorumError::InsufficientLightestQuorum {
                lightest_quorum_stake,
                total_stake,
                ..
            } => {
                assert_eq!(lightest_quorum_stake, 3);
                assert_eq!(total_stake, 103);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn boundary_exactly_at_threshold_clears() {
        // n=4, f=1: lightest quorum 75% == 7500 bps threshold.
        assert!(reconcile(&[100, 100, 100, 100], 7500).is_ok());
        // One bps above fails.
        assert!(reconcile(&[100, 100, 100, 100], 7501).is_err());
    }

    #[test]
    fn zero_and_empty_sets_rejected() {
        assert_eq!(reconcile(&[], 6667), Err(QuorumError::Empty));
        assert_eq!(
            reconcile(&[0, 0, 0], 6667),
            Err(QuorumError::ZeroTotalStake)
        );
    }

    #[test]
    fn zero_stake_operator_can_sink_the_lightest_quorum() {
        // An unsnapshotted (zero-stake) operator dilutes the lightest quorum:
        // n=4, f=1 → lightest = {0, 100, 100} = 200/300 = 6666 bps < 6667.
        assert!(reconcile(&[0, 100, 100, 100], 6667).is_err());
    }

    #[test]
    fn u128_overflow_falls_back_to_exact_arithmetic() {
        // Three near-max stakes: the u128 prefix sum overflows, the BigUint
        // path must still reconcile correctly (equal weights, n=3 → 10000 bps).
        let stakes = [u128::MAX - 1, u128::MAX - 1, u128::MAX - 1];
        let result = reconcile(&stakes, 6667).expect("clears via exact arithmetic");
        assert_eq!(result.lightest_quorum_bps, 10_000);
    }

    #[test]
    fn max_faults_matches_n3f1() {
        assert_eq!(max_faults(1), 0);
        assert_eq!(max_faults(3), 0);
        assert_eq!(max_faults(4), 1);
        assert_eq!(max_faults(7), 2);
        assert_eq!(max_faults(10), 3);
    }
}
