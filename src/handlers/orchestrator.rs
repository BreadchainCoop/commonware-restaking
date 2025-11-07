use crate::handlers::creator::create_creator;
use crate::handlers::executor::create_executor;
use crate::handlers::listening_creator::create_listening_creator_with_server;
use crate::handlers::{TaskCreator, TaskCreatorEnum};
use crate::solana_helpers::get_raw_message_and_hash;
use crate::wire::{self, aggregation::Payload};

use bytes::Bytes;
use commonware_codec::{EncodeSize, ReadExt, Write};
use commonware_cryptography::Verifier;
use commonware_macros::select;
use commonware_p2p::{Receiver, Sender};
use commonware_runtime::Clock;
use commonware_utils::hex;
use dotenv::dotenv;
use jito_bls_ncn_core::bls::solana_bls_interface::{SolanaBN254G1, SolanaBN254G2, SolanaBN254Keypair, SolanaBN254PublicKey, SolanaBN254Signature};
use std::{collections::HashMap, time::Duration};
use tracing::info;
const DEFAULT_VAR_1: &str = "default_var1";
const DEFAULT_VAR_2: &str = "default_var2";
const DEFAULT_VAR_3: &str = "default_var3";

pub struct Orchestrator<E: Clock> {
    runtime: E,
    #[allow(dead_code)]
    signer: SolanaBN254Keypair,
    aggregation_frequency: Duration,
    contributors: Vec<SolanaBN254G2>,
    g1_map: HashMap<SolanaBN254G2, SolanaBN254PublicKey>, // g2 (PublicKey) -> g1 (PublicKey)
    ordered_contributors: HashMap<SolanaBN254G2, usize>,
    t: usize,
}

impl<E: Clock> Orchestrator<E> {
    pub async fn new(
        runtime: E,
        signer: SolanaBN254Keypair,
        aggregation_frequency: Duration,
        mut contributors: Vec<SolanaBN254G2>,
        g1_map: HashMap<SolanaBN254G2, SolanaBN254PublicKey>,
        t: usize,
    ) -> Self {
        dotenv().ok();

        contributors.sort();
        let mut ordered_contributors = HashMap::new();
        for (idx, contributor) in contributors.iter().enumerate() {
            ordered_contributors.insert(contributor.clone(), idx);
        }

        Self {
            runtime,
            signer,
            aggregation_frequency,
            contributors,
            g1_map,
            ordered_contributors,
            t,
        }
    }

    pub async fn run(
        self,
        mut sender: impl Sender,
        mut receiver: impl Receiver<PublicKey = SolanaBN254G2>,
    ) {
        let mut signatures = HashMap::new();
        let task_creator: TaskCreatorEnum;
        // Check if INGRESS flag is set to determine which creator to use
        let use_ingress = std::env::var("INGRESS").unwrap_or_default().to_lowercase() == "true";
        if use_ingress {
            info!("Using ListeningCreator with HTTP server on port 8080");
            let listening_creator =
                create_listening_creator_with_server("0.0.0.0:8080".to_string())
                    .await
                    .unwrap();
            task_creator = TaskCreatorEnum::ListeningCreator(listening_creator);
        } else {
            info!("Using Creator without ingress");
            let creator = create_creator().await.unwrap();
            task_creator = TaskCreatorEnum::Creator(creator);
        };
        let mut executor = create_executor().await.unwrap();
        // let validator = Validator::new().await.unwrap();

        loop {
            let (_, current_number) = task_creator.get_payload_and_round().await.unwrap();
            let (raw_message, _) = get_raw_message_and_hash(current_number).unwrap();

            info!(
                round = current_number.to_string(),
                msg = hex(&raw_message),
                "generated payload for round"
            );

            // Broadcast this raw message for signing
            let message = wire::Aggregation {
                round: current_number,
                var1: DEFAULT_VAR_1.to_string(),
                var2: DEFAULT_VAR_2.to_string(),
                var3: DEFAULT_VAR_3.to_string(),
                payload: Some(Payload::Start),
            };
            let mut buf = Vec::with_capacity(message.encode_size());
            message.write(&mut buf);
            sender
                .send(commonware_p2p::Recipients::All, Bytes::from(buf), true)
                .await
                .expect("failed to broadcast message");
            signatures.insert(current_number, HashMap::new());
            info!(
                "Created signatures entry for round: {}, threshold is: {}",
                current_number, self.t
            );

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

                        let msg = match wire::Aggregation::read(&mut std::io::Cursor::new(&msg)) {
                            Ok(m) => m,
                            Err(e) => {
                                info!("Failed to decode message from sender: {:?}, error: {:?}, raw bytes length: {}",
                                      sender, e, msg.len());
                                continue;
                            }
                        };
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
                        let Ok(signature) = SolanaBN254Signature::try_from(signature) else {
                            info!("Failed to parse signature from contributor: {:?}", contributor);
                            continue;
                        };

                        // Create the raw message from the round (same as contributors do)
                        let (raw_message, _) = get_raw_message_and_hash(current_number).unwrap();

                        info!("Verifying signature for round: {} from contributor: {:?}, raw message: {}",
                              msg.round, contributor, hex(&raw_message));

                        // Verify against the raw message, not the hashed wire message
                        let consensus_bytes = msg.round.to_le_bytes();
                        if !sender.verify(Some(&consensus_bytes), &raw_message, &signature) {
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
                        let mut signatures = Vec::new();
                        for i in 0..self.contributors.len() {
                            let Some(signature) = round.get(&i) else {
                                continue;
                            };
                            let contributor = &self.contributors[i];
                            let g1_pubkey : SolanaBN254G1 = self.g1_map[contributor].g1.clone();
                            let bls_pubkey: SolanaBN254PublicKey = SolanaBN254PublicKey::new(g1_pubkey, contributor.clone());

                            participating.push(bls_pubkey);
                            signatures.push(signature.clone());
                        }

                        match executor.execute_verification(
                            &raw_message,
                            &participating,
                            &signatures,
                        ).await {
                            Ok(result) => {
                                info!(
                                    round = msg.round,
                                    "Successfully executed increment with aggregated signature. Result: {:?}",
                                    result
                                );
                            },
                            Err(e) => {
                                info!(
                                    round = msg.round,
                                    "Failed to execute increment with aggregated signature: {:?}",
                                    e
                                );
                            }
                        }
                    },
                }
            }
        }
    }
}
