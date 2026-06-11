//! Round-lifecycle metrics for the orchestrator.

use commonware_runtime::Metrics;
use commonware_runtime::telemetry::metrics::{histogram::Buckets, status};
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::histogram::Histogram;

/// Metrics covering the orchestrator's aggregation-round lifecycle.
///
/// All metrics are registered under an `orchestrator`-labeled scope of the runtime
/// context the orchestrator is built with, so they appear wherever the consumer
/// encodes the runtime registry (`Metrics::encode`) without additional wiring.
pub struct OrchestratorMetrics {
    /// Rounds for which a fresh signature-collection entry was created.
    pub rounds_started: Counter,
    /// `Start` broadcasts sent, including re-broadcasts of a still-open round.
    pub round_broadcasts: Counter,
    /// Aggregation windows that expired before the round completed; the round is
    /// re-broadcast on the next pass of the outer loop.
    pub round_timeouts: Counter,
    /// Outcome of each execution attempt made once the signature threshold is met.
    pub round_executions: status::Counter,
    /// Per-message signature handling outcome: `Success` for accepted signatures,
    /// `Invalid` for messages that fail decoding, validation, or BLS verification,
    /// and `Dropped` for messages ignored on purpose (duplicates, unknown senders,
    /// unknown or already-executed rounds, non-signature payloads).
    pub signatures: status::Counter,
    /// Time from a round's first broadcast to reaching the signature threshold.
    pub time_to_quorum: Histogram,
    /// Time from a round's first broadcast to each accepted signature.
    pub signature_arrival: Histogram,
}

impl OrchestratorMetrics {
    /// Registers all orchestrator metrics under an `orchestrator` label scope on
    /// `context`.
    pub fn new(context: &impl Metrics) -> Self {
        let context = context.with_label("orchestrator");

        let rounds_started = Counter::default();
        context.register(
            "rounds_started",
            "Rounds for which a new signature-collection entry was created",
            rounds_started.clone(),
        );

        let round_broadcasts = Counter::default();
        context.register(
            "round_broadcasts",
            "Start broadcasts sent, including re-broadcasts of still-open rounds",
            round_broadcasts.clone(),
        );

        let round_timeouts = Counter::default();
        context.register(
            "round_timeouts",
            "Aggregation windows that expired before the round completed",
            round_timeouts.clone(),
        );

        let round_executions = status::Counter::default();
        context.register(
            "round_executions",
            "Outcome of execution attempts made when the signature threshold is met",
            round_executions.clone(),
        );

        let signatures = status::Counter::default();
        context.register(
            "signatures",
            "Handling outcome of received signature messages",
            signatures.clone(),
        );

        let time_to_quorum = Histogram::new(Buckets::NETWORK.into_iter());
        context.register(
            "time_to_quorum_seconds",
            "Time from a round's first broadcast to reaching the signature threshold",
            time_to_quorum.clone(),
        );

        let signature_arrival = Histogram::new(Buckets::NETWORK.into_iter());
        context.register(
            "signature_arrival_seconds",
            "Time from a round's first broadcast to each accepted signature",
            signature_arrival.clone(),
        );

        Self {
            rounds_started,
            round_broadcasts,
            round_timeouts,
            round_executions,
            signatures,
            time_to_quorum,
            signature_arrival,
        }
    }
}
