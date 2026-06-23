use bytes::Bytes;
use commonware_avs_core::bn254::{
    Bn254, G1PublicKey, PublicKey, Signature as Bn254Signature, aggregate_signatures,
    aggregate_verify,
};
use commonware_avs_core::validator::ValidatorTrait;
use commonware_avs_core::wire::{Aggregation, aggregation::Payload};
use commonware_codec::{EncodeSize, ReadExt, Write};
use commonware_cryptography::{Hasher, Sha256, Verifier};
use commonware_p2p::{Receiver, Sender};
use commonware_runtime::telemetry::metrics::histogram::HistogramExt;
use commonware_runtime::telemetry::metrics::status::{CounterExt, Status};
use commonware_runtime::{Clock, Metrics};
use commonware_utils::hex;
use std::{
    collections::HashMap,
    marker::PhantomData,
    time::{Duration, SystemTime},
};
use tracing::info;

use crate::creator::Creator;
use crate::executor::{FromBlsAggregation, VerificationData, VerificationExecutor};
use crate::orchestrator::metrics::OrchestratorMetrics;
use crate::orchestrator::traits::OrchestratorTrait;

/// Configuration for the generic orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Max duration to wait for operator signatures before abandoning a round.
    pub round_timeout: Duration,
    /// How often to re-send the `Start` broadcast for an in-flight round.
    pub rebroadcast_interval: Duration,
    pub contributors: Vec<PublicKey>,
    pub g1_map: HashMap<PublicKey, G1PublicKey>,
    pub threshold: usize,
}

/// Generic orchestrator that accepts trait-based dependencies.
///
/// This struct provides a flexible implementation of the orchestration process
/// that can work with any implementation of the required traits. It follows
/// the Dependency Inversion Principle by depending on abstractions rather
/// than concrete implementations.
///
/// # Type Parameters
/// * `TC` - Task creator implementation that implements `Creator`
/// * `E` - Executor implementation that implements `VerificationExecutor`
/// * `V` - Validator implementation that implements `ValidatorTrait`
/// * `C` - Runtime handle implementing `Clock` for timing and `Metrics` for metric registration
/// * `VD` - Verification-data container the executor consumes (defaults to `VerificationData`)
pub struct Orchestrator<TC, E, V, C, VD = VerificationData>
where
    TC: Creator,
    E: VerificationExecutor<TC::TaskData, VD>,
    V: ValidatorTrait,
    C: Clock + Metrics,
    VD: FromBlsAggregation + Send + Sync,
{
    runtime: C,
    #[allow(dead_code)]
    signer: Bn254,
    round_timeout: Duration,
    rebroadcast_interval: Duration,
    contributors: Vec<PublicKey>,
    g1_map: HashMap<PublicKey, G1PublicKey>, // g2 (PublicKey) -> g1 (PublicKey)
    ordered_contributors: HashMap<PublicKey, usize>,
    t: usize,
    task_creator: TC,
    executor: E,
    validator: V,
    metrics: OrchestratorMetrics,
    _verification_data: PhantomData<fn() -> VD>,
}

/// Per-round signature-collection state.
struct RoundState {
    /// Time of the round's first broadcast, the origin for time-to-quorum and
    /// signature-arrival measurements.
    started: SystemTime,
    /// Verified signatures collected so far, keyed by contributor index.
    signatures: HashMap<usize, Bn254Signature>,
}

impl<TC, E, V, C, VD> Orchestrator<TC, E, V, C, VD>
where
    TC: Creator,
    E: VerificationExecutor<TC::TaskData, VD>,
    V: ValidatorTrait,
    C: Clock + Metrics,
    VD: FromBlsAggregation + Send + Sync,
{
    /// Creates a new Orchestrator instance with the given dependencies.
    ///
    /// This constructor takes ownership of all the required components
    /// and initializes the orchestrator with the provided configuration.
    ///
    /// # Arguments
    /// * `runtime` - The clock implementation for timing operations
    /// * `signer` - The BLS signer for cryptographic operations
    /// * `config` - The orchestrator configuration
    /// * `task_creator` - The task creator implementation
    /// * `executor` - The executor implementation
    /// * `validator` - The validator implementation
    ///
    /// # Returns
    /// * `Self` - The new Orchestrator instance
    pub fn new(
        runtime: C,
        signer: Bn254,
        config: OrchestratorConfig,
        task_creator: TC,
        executor: E,
        validator: V,
    ) -> Self {
        let mut contributors = config.contributors.clone();
        contributors.sort();
        let mut ordered_contributors = HashMap::new();
        for (idx, contributor) in contributors.iter().enumerate() {
            ordered_contributors.insert(contributor.clone(), idx);
        }

        let metrics = OrchestratorMetrics::new(&runtime);
        Self {
            runtime,
            signer,
            round_timeout: config.round_timeout,
            rebroadcast_interval: config.rebroadcast_interval,
            contributors,
            g1_map: config.g1_map,
            ordered_contributors,
            t: config.threshold,
            task_creator,
            executor,
            validator,
            metrics,
            _verification_data: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<TC, E, V, C, VD> OrchestratorTrait for Orchestrator<TC, E, V, C, VD>
where
    TC: Creator + Send + Sync,
    E: VerificationExecutor<TC::TaskData, VD> + Send + Sync,
    V: ValidatorTrait + Send + Sync,
    C: Clock + Metrics + Send + Sync,
    VD: FromBlsAggregation + Send + Sync,
{
    async fn run(
        mut self,
        mut sender: impl Sender,
        mut receiver: impl Receiver<PublicKey = PublicKey>,
    ) {
        // Backoff between wait_for_new_round retries; bounds hot-spinning when a creator
        // fails immediately (e.g. CounterCreator when the provider is temporarily down).
        const WAIT_RETRY_BACKOFF: Duration = Duration::from_secs(1);

        let mut signatures: HashMap<u64, RoundState> = HashMap::new();
        // Carries the (payload, round) returned by wait_for_new_round after a successful
        // execution so the outer loop can use it directly without an extra get_payload_and_round
        // call. None on the first iteration and after every aggregation timeout.
        let mut cached_round: Option<(Vec<u8>, u64)> = None;

        loop {
            let (payload, current_round) = match cached_round.take() {
                Some(val) => val,
                None => match self.task_creator.get_payload_and_round().await {
                    Ok(val) => val,
                    Err(e) => {
                        tracing::error!("get_payload_and_round failed: {e}");
                        continue;
                    }
                },
            };

            // Create a new hasher for each iteration
            let mut hasher = Sha256::new();
            hasher.update(&payload);
            let payload = hasher.finalize();
            info!(
                state = current_round.to_string(),
                msg = hex(&payload),
                "generated payload for state"
            );

            // Broadcast payload
            let task_data = self.task_creator.get_task_metadata();
            let message =
                Aggregation::<TC::TaskData>::new(current_round, task_data, Some(Payload::Start));
            let mut buf = Vec::with_capacity(message.encode_size());
            message.write(&mut buf);
            sender
                .send(commonware_p2p::Recipients::All, Bytes::from(buf), true)
                .await
                .expect("failed to broadcast message");
            self.metrics.round_broadcasts.inc();

            // Only create a new signature entry if one doesn't exist for this round
            use std::collections::hash_map::Entry;
            match signatures.entry(current_round) {
                Entry::Vacant(e) => {
                    e.insert(RoundState {
                        started: self.runtime.current(),
                        signatures: HashMap::new(),
                    });
                    self.metrics.rounds_started.inc();
                    info!(
                        "Created signatures entry for state: {}, threshold is: {}",
                        current_round, self.t
                    );
                }
                Entry::Occupied(_) => {}
            }

            // Collect signatures until the round times out or reaches the threshold.
            // The rebroadcast timer re-sends `Start` for the same round on a shorter
            // cadence so nodes that missed the first broadcast can still sign; it is
            // independent of round_timeout so operators can set a fast recovery window
            // without flooding the network with Start messages.
            //
            // biased: round_timeout is checked first so it wins when both it and the
            // rebroadcast timer fire simultaneously (round_timeout == rebroadcast_interval),
            // preserving the original single-knob behaviour.
            let round_timeout_deadline = self.runtime.current() + self.round_timeout;
            let mut executed_round: Option<u64> = None;
            let mut rebroadcast_fut = std::pin::pin!(
                self.runtime.sleep_until(self.runtime.current() + self.rebroadcast_interval)
            );
            loop {
                tokio::select! {
                    biased;
                    _ = self.runtime.sleep_until(round_timeout_deadline) => {
                        self.metrics.round_timeouts.inc();
                        break;
                    },
                    _ = &mut rebroadcast_fut => {
                        let task_data = self.task_creator.get_task_metadata();
                        let rebroadcast_msg = Aggregation::<TC::TaskData>::new(
                            current_round,
                            task_data,
                            Some(Payload::Start),
                        );
                        let mut buf = Vec::with_capacity(rebroadcast_msg.encode_size());
                        rebroadcast_msg.write(&mut buf);
                        sender
                            .send(commonware_p2p::Recipients::All, Bytes::from(buf), true)
                            .await
                            .expect("failed to rebroadcast Start");
                        self.metrics.round_broadcasts.inc();
                        rebroadcast_fut.set(
                            self.runtime
                                .sleep_until(self.runtime.current() + self.rebroadcast_interval),
                        );
                    },
                    msg = receiver.recv() => {
                        // Parse message
                        let (sender, msg) = match msg {
                            Ok(msg) => msg,
                            Err(_) => continue,
                        };

                        // Get contributor
                        let Some(contributor) = self.ordered_contributors.get(&sender) else {
                            info!("Received message from unknown sender: {:?}", sender);
                            self.metrics.signatures.inc(Status::Dropped);
                            continue;
                        };

                        // Check if round exists
                        let Ok(msg): Result<Aggregation<TC::TaskData>, _> = Aggregation::read(&mut std::io::Cursor::new(msg)) else {
                            info!("Failed to decode message from sender: {:?}", sender);
                            self.metrics.signatures.inc(Status::Invalid);
                            continue;
                        };
                        let Some(round) = signatures.get_mut(&msg.round) else {
                            info!("Received signature for unknown round: {} from contributor: {:?}", msg.round, contributor);
                            self.metrics.signatures.inc(Status::Dropped);
                            continue;
                        };

                        // Check if contributor has already signed
                        if round.signatures.contains_key(contributor) {
                            info!("Contributor already signed for round: {} contributor: {:?}", msg.round, contributor);
                            self.metrics.signatures.inc(Status::Dropped);
                            continue;
                        }

                        // Extract signature
                        let signature = match msg.payload.clone() {
                            Some(Payload::Signature(signature)) => {
                                info!("Received signature for round: {} from contributor: {:?}", msg.round, contributor);
                                signature
                            },
                            _ => {
                                info!("Received non-signature payload from contributor: {:?}", contributor);
                                self.metrics.signatures.inc(Status::Dropped);
                                continue;
                            }
                        };
                        let Ok(signature) = Bn254Signature::try_from(signature) else {
                            info!("Failed to parse signature from contributor: {:?}", contributor);
                            self.metrics.signatures.inc(Status::Invalid);
                            continue;
                        };

                        let mut buf = Vec::with_capacity(msg.encode_size());
                        msg.write(&mut buf);
                        let expected_digest = match self.validator.validate_and_return_expected_hash(&buf).await {
                            Ok(d) => d,
                            Err(e) => {
                                info!(
                                    "Validator rejected message for round {} from contributor {:?}: {}",
                                    msg.round, contributor, e
                                );
                                // Likely a stale signature after the round was executed; ignore safely.
                                self.metrics.signatures.inc(Status::Invalid);
                                continue;
                            }
                        };
                        info!("Verifying signature for round: {} from contributor: {:?}, expected digest: {}",
                              msg.round, contributor, hex(&expected_digest));

                        // Get the contributor's public key for verification
                        let contributor_pubkey = &self.contributors[*contributor];
                        if !contributor_pubkey.verify(None, &expected_digest, &signature) {
                            info!("Signature verification failed for contributor: {:?}", contributor);
                            self.metrics.signatures.inc(Status::Invalid);
                            continue;
                        }

                        info!("Signature verification succeeded for contributor: {:?}", contributor);

                        // Insert signature
                        round.signatures.insert(*contributor, signature);
                        self.metrics.signatures.inc(Status::Success);
                        self.metrics
                            .signature_arrival
                            .observe_between(round.started, self.runtime.current());

                        // Check if should aggregate
                        info!("Current signatures count for round {}: {}, threshold: {}",
                              msg.round, round.signatures.len(), self.t);
                        if round.signatures.len() < self.t {
                            continue;
                        }
                        // The signature count only ever grows by one, so equality marks the
                        // first time this round reaches the threshold; a retriggered
                        // execution after a failure arrives with a higher count.
                        if round.signatures.len() == self.t {
                            self.metrics
                                .time_to_quorum
                                .observe_between(round.started, self.runtime.current());
                        }

                        // Aggregate signatures
                        let mut participating = Vec::new();
                        let mut participating_g1 = Vec::new();
                        let mut agg_signatures = Vec::new();
                        for i in 0..self.contributors.len() {
                            let Some(signature) = round.signatures.get(&i) else {
                                continue;
                            };
                            let contributor = &self.contributors[i];
                            let g1_pubkey : G1PublicKey= self.g1_map[contributor].clone();
                            participating_g1.push(g1_pubkey.clone());
                            participating.push(contributor.clone());
                            agg_signatures.push(signature.clone());
                        }
                        let agg_signature = aggregate_signatures(&agg_signatures).unwrap();

                        // Verify aggregated signature (already verified individual signatures so should never fail)
                        if !aggregate_verify(&participating, None, &expected_digest, &agg_signature) {
                            panic!("failed to verify aggregated signature");
                        }

                        // Build the executor's verification-data container from the aggregation output.
                        let verification_data = VD::from_bls_aggregation(
                            agg_signatures,
                            participating,
                            participating_g1,
                        );

                        match self.executor.execute_verification(
                            msg.round,
                            &expected_digest,
                            verification_data,
                            Some(&msg.metadata),
                        ).await {
                            Ok(result) => {
                                info!(
                                    round = msg.round,
                                    "Successfully executed verification with aggregated signature. Result: {:?}",
                                    result
                                );
                                self.metrics.round_executions.inc(Status::Success);
                                signatures.remove(&msg.round);
                                executed_round = Some(msg.round);
                                break;
                            },
                            Err(e) => {
                                info!(
                                    round = msg.round,
                                    "Failed to execute verification with aggregated signature: {:?}",
                                    e
                                );
                                self.metrics.round_executions.inc(Status::Failure);
                                // Leave the round open so a subsequent signature above threshold
                                // can retrigger execution.
                            }
                        }
                    },
                }
            }

            // After the inner loop exits: if execution succeeded, wait until the on-chain
            // round advances before broadcasting again. The receiver is drained concurrently
            // to prevent P2P buffer build-up during the inter-round wait. Transient errors
            // from wait_for_new_round (e.g. a queue-backed creator timing out between tasks)
            // are logged and retried rather than panicking.
            if let Some(ex_round) = executed_round {
                let mut wait_fut = std::pin::pin!(self.task_creator.wait_for_new_round(ex_round));
                cached_round = Some(loop {
                    tokio::select! {
                        biased;
                        result = &mut wait_fut => {
                            match result {
                                Ok(val) => break val,
                                Err(e) => {
                                    tracing::warn!(
                                        round = ex_round,
                                        "wait_for_new_round error: {e}; retrying"
                                    );
                                    self.runtime
                                        .sleep_until(
                                            self.runtime.current() + WAIT_RETRY_BACKOFF,
                                        )
                                        .await;
                                    wait_fut.set(
                                        self.task_creator.wait_for_new_round(ex_round)
                                    );
                                }
                            }
                        },
                        received = receiver.recv() => {
                            let _ = received;
                        }
                    }
                });
            }
        }
    }
}

impl<TC, E, V, C, VD> Orchestrator<TC, E, V, C, VD>
where
    TC: Creator,
    E: VerificationExecutor<TC::TaskData, VD>,
    V: ValidatorTrait,
    C: Clock + Metrics,
    VD: FromBlsAggregation + Send + Sync,
{
    /// Get a reference to the task creator
    #[allow(dead_code)]
    pub fn task_creator(&self) -> &TC {
        &self.task_creator
    }

    /// Get a reference to the executor
    #[allow(dead_code)]
    pub fn executor(&self) -> &E {
        &self.executor
    }

    /// Get a reference to the validator
    #[allow(dead_code)]
    pub fn validator(&self) -> &V {
        &self.validator
    }
}
