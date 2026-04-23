use bytes::Bytes;
use commonware_avs_core::bn254::{
    Bn254, G1PublicKey, PublicKey, Signature as Bn254Signature, aggregate_signatures,
    aggregate_verify,
};
use commonware_avs_core::validator::ValidatorTrait;
use commonware_avs_core::wire::{Aggregation, aggregation::Payload};
use commonware_codec::{EncodeSize, ReadExt, Write};
use commonware_cryptography::{Hasher, Sha256, Verifier};
use commonware_macros::select;
use commonware_p2p::{Receiver, Sender};
use commonware_runtime::Clock;
use commonware_utils::hex;
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use tracing::info;

use crate::creator::Creator;
use crate::executor::{VerificationData, VerificationExecutor};
use crate::orchestrator::traits::OrchestratorTrait;

/// Configuration for the generic orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub aggregation_frequency: Duration,
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
/// * `C` - Clock implementation that implements `Clock`
pub struct Orchestrator<TC, E, V, C>
where
    TC: Creator,
    E: VerificationExecutor<TC::TaskData, VerificationData>,
    V: ValidatorTrait,
    C: Clock,
{
    runtime: C,
    #[allow(dead_code)]
    signer: Bn254,
    aggregation_frequency: Duration,
    contributors: Vec<PublicKey>,
    g1_map: HashMap<PublicKey, G1PublicKey>, // g2 (PublicKey) -> g1 (PublicKey)
    ordered_contributors: HashMap<PublicKey, usize>,
    t: usize,
    task_creator: TC,
    executor: E,
    validator: V,
}

impl<TC, E, V, C> Orchestrator<TC, E, V, C>
where
    TC: Creator,
    E: VerificationExecutor<TC::TaskData, VerificationData>,
    V: ValidatorTrait,
    C: Clock,
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

        Self {
            runtime,
            signer,
            aggregation_frequency: config.aggregation_frequency,
            contributors,
            g1_map: config.g1_map,
            ordered_contributors,
            t: config.threshold,
            task_creator,
            executor,
            validator,
        }
    }
}

#[async_trait::async_trait]
impl<TC, E, V, C> OrchestratorTrait for Orchestrator<TC, E, V, C>
where
    TC: Creator + Send + Sync,
    E: VerificationExecutor<TC::TaskData, VerificationData> + Send + Sync,
    V: ValidatorTrait + Send + Sync,
    C: Clock + Send + Sync,
{
    async fn run(
        mut self,
        mut sender: impl Sender,
        mut receiver: impl Receiver<PublicKey = PublicKey>,
    ) {
        let mut signatures = HashMap::new();
        let mut executed_rounds: HashSet<u64> = HashSet::new();

        loop {
            let (payload, current_round) = self.task_creator.get_payload_and_round().await.unwrap();

            // Create a new hasher for each iteration
            let mut hasher = Sha256::new();
            hasher.update(&payload);
            let payload = hasher.finalize();
            info!(
                state = current_round.to_string(),
                msg = hex(&payload),
                "generated payload for state"
            );

            // Skip broadcasting for already-executed rounds; poll on a short interval until next round.
            if executed_rounds.contains(&current_round) {
                self.runtime.sleep(Duration::from_secs(2)).await;
                continue;
            }

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

            // Only create a new signature entry if one doesn't exist for this round
            use std::collections::hash_map::Entry;
            match signatures.entry(current_round) {
                Entry::Vacant(e) => {
                    e.insert(HashMap::new());
                    info!(
                        "Created signatures entry for state: {}, threshold is: {}",
                        current_round, self.t
                    );
                }
                Entry::Occupied(_) => {}
            }

            // Listen for messages until the next broadcast
            let continue_time = self.runtime.current() + self.aggregation_frequency;
            loop {
                select! {
                    _ = self.runtime.sleep_until(continue_time) => {break;},
                    msg = receiver.recv() => {
                        // Parse message
                        let (sender, msg) = match msg {
                            Ok(msg) => msg,
                            Err(_) => continue,
                        };

                        // Get contributor
                        let Some(contributor) = self.ordered_contributors.get(&sender) else {
                            info!("Received message from unknown sender: {:?}", sender);
                            continue;
                        };

                        // Check if round exists
                        let Ok(msg): Result<Aggregation<TC::TaskData>, _> = Aggregation::read(&mut std::io::Cursor::new(msg)) else {
                            info!("Failed to decode message from sender: {:?}", sender);
                            continue;
                        };
                        if executed_rounds.contains(&msg.round) {
                            info!("Ignoring signature for already-executed round: {} from contributor: {:?}", msg.round, contributor);
                            continue;
                        }
                        let Some(round) = signatures.get_mut(&msg.round) else {
                            info!("Received signature for unknown round: {} from contributor: {:?}", msg.round, contributor);
                            continue;
                        };

                        // Check if contributor has already signed
                        if round.contains_key(contributor) {
                            info!("Contributor already signed for round: {} contributor: {:?}", msg.round, contributor);
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
                                continue;
                            }
                        };
                        let Ok(signature) = Bn254Signature::try_from(signature) else {
                            info!("Failed to parse signature from contributor: {:?}", contributor);
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
                                continue;
                            }
                        };
                        info!("Verifying signature for round: {} from contributor: {:?}, expected digest: {}",
                              msg.round, contributor, hex(&expected_digest));

                        // Get the contributor's public key for verification
                        let contributor_pubkey = &self.contributors[*contributor];
                        if !contributor_pubkey.verify(None, &expected_digest, &signature) {
                            info!("Signature verification failed for contributor: {:?}", contributor);
                            continue;
                        }

                        info!("Signature verification succeeded for contributor: {:?}", contributor);

                        // Insert signature
                        round.insert(contributor, signature);

                        // Check if should aggregate
                        info!("Current signatures count for round {}: {}, threshold: {}",
                              msg.round, round.len(), self.t);
                        if round.len() < self.t {
                            continue;
                        }

                        // Aggregate signatures
                        let mut participating = Vec::new();
                        let mut participating_g1 = Vec::new();
                        let mut agg_signatures = Vec::new();
                        for i in 0..self.contributors.len() {
                            let Some(signature) = round.get(&i) else {
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

                        // Execute verification with the aggregated signature
                        // Serialize BLS-specific types to bytes for generic VerificationData
                        let serialized_signatures: Vec<Vec<u8>> = agg_signatures.iter().map(|s| s.to_vec()).collect();
                        let serialized_public_keys: Vec<Vec<u8>> = participating.iter().map(|pk| pk.to_vec()).collect();

                        // Serialize G1 public keys to context
                        let mut context = Vec::new();
                        for g1_pubkey in &participating_g1 {
                            context.extend_from_slice(g1_pubkey);
                        }

                        let verification_data = VerificationData::new(serialized_signatures, serialized_public_keys)
                            .with_context(context);

                        match self.executor.execute_verification(
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
                            },
                            Err(e) => {
                                info!(
                                    round = msg.round,
                                    "Failed to execute verification with aggregated signature: {:?}",
                                    e
                                );
                            }
                        }

                        // Drop per-round signature state now that this round has finished.
                        signatures.remove(&msg.round);
                        // Mark round complete so additional signatures are ignored and the outer
                        // loop does not re-broadcast Start while waiting for on-chain state to advance.
                        //
                        // Rounds advance monotonically, so only the latest executed round needs to
                        // be retained to suppress duplicate processing without unbounded growth.
                        executed_rounds.clear();
                        executed_rounds.insert(msg.round);
                    },
                }
            }
        }
    }
}

impl<TC, E, V, C> Orchestrator<TC, E, V, C>
where
    TC: Creator,
    E: VerificationExecutor<TC::TaskData, VerificationData>,
    V: ValidatorTrait,
    C: Clock,
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
