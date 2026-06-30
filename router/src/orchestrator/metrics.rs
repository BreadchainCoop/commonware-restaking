//! Round-lifecycle metrics for the orchestrator.

use commonware_runtime::Metrics;
use commonware_runtime::telemetry::metrics::{Counter, Histogram, histogram::Buckets, raw, status};

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
        let context = context.child("orchestrator");

        let rounds_started = context.register(
            "rounds_started",
            "Rounds for which a new signature-collection entry was created",
            raw::Counter::default(),
        );

        let round_broadcasts = context.register(
            "round_broadcasts",
            "Start broadcasts sent, including re-broadcasts of still-open rounds",
            raw::Counter::default(),
        );

        let round_timeouts = context.register(
            "round_timeouts",
            "Aggregation windows that expired before the round completed",
            raw::Counter::default(),
        );

        let round_executions = context.register(
            "round_executions",
            "Outcome of execution attempts made when the signature threshold is met",
            status::Raw::default(),
        );

        let signatures = context.register(
            "signatures",
            "Handling outcome of received signature messages",
            status::Raw::default(),
        );

        let time_to_quorum = context.register(
            "time_to_quorum_seconds",
            "Time from a round's first broadcast to reaching the signature threshold",
            raw::Histogram::new(Buckets::NETWORK),
        );

        let signature_arrival = context.register(
            "signature_arrival_seconds",
            "Time from a round's first broadcast to each accepted signature",
            raw::Histogram::new(Buckets::NETWORK),
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
