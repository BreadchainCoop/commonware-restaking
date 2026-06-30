//! Metrics for the EigenLayer reads the BLS executor performs each execution.

use commonware_runtime::Metrics;
use commonware_runtime::telemetry::metrics::{Counter, Histogram, histogram::Buckets, raw};

/// Metrics covering the EigenLayer state reads that precede contract-handler
/// execution: operator-address resolution and the non-signer stakes/signature
/// retrieval.
///
/// Registered under an `executor`-labeled scope of the runtime context passed to
/// [`super::BlsEigenlayerExecutor::with_metrics`], so they appear wherever the
/// consumer encodes the runtime registry (`Metrics::encode`).
pub struct ExecutorMetrics {
    /// Time spent resolving operator addresses and fetching non-signer stakes and
    /// signature from EigenLayer, per execution.
    pub state_retrieval: Histogram,
    /// Operator addresses resolved via `pubkeyHashToOperator` because they were
    /// not yet in the executor's cache.
    pub operator_cache_misses: Counter,
}

impl ExecutorMetrics {
    /// Registers all executor metrics under an `executor` label scope on `context`.
    pub fn new(context: &impl Metrics) -> Self {
        let context = context.child("executor");

        let state_retrieval = context.register(
            "state_retrieval_seconds",
            "Time resolving operator addresses and fetching non-signer stakes and signature from EigenLayer",
            raw::Histogram::new(Buckets::NETWORK.into_iter()),
        );

        let operator_cache_misses = context.register(
            "operator_cache_misses",
            "Operator addresses resolved over RPC because they were not yet cached",
            raw::Counter::default(),
        );

        Self {
            state_retrieval,
            operator_cache_misses,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::tests::mocks::clock::MockClock;

    #[test]
    fn test_metrics_registered_and_observable() {
        let context = MockClock::new();
        let metrics = ExecutorMetrics::new(&context);

        metrics.state_retrieval.observe(0.25);
        metrics.operator_cache_misses.inc_by(3);

        let encoded = context.encode();
        for expected in [
            "executor_state_retrieval_seconds_count 1",
            "executor_operator_cache_misses_total 3",
        ] {
            assert!(
                encoded.contains(expected),
                "expected `{expected}` in encoded metrics:\n{encoded}"
            );
        }
    }
}
