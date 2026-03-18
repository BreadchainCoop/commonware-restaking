use crate::contributor::types::AggregationData;
use crate::contributor::{AggregationInput, Contribute, ContributorBase};
use anyhow::Result;
use bytes::Bytes;
use commonware_avs_core::bn254::{
    Bn254 as EllipticCurve, PublicKey as PubKey, Signature as Sig, aggregate_signatures,
    aggregate_verify,
};
use commonware_avs_core::validator::ValidatorTrait;
use commonware_avs_core::wire::{self, aggregation::Payload};
use commonware_codec::{EncodeSize, ReadExt, Write};
use commonware_cryptography::Signer;
use commonware_p2p::{Receiver, Sender};
use commonware_utils::hex;
use dotenv::dotenv;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::info;

pub struct Contributor<T>
where
    T: Clone + Send + Sync + Write + commonware_codec::Read<Cfg = ()> + EncodeSize,
{
    orchestrator: PubKey,
    signer: EllipticCurve,
    me: usize,
    aggregation_data: Option<AggregationData>,
    validator: Option<Arc<dyn ValidatorTrait>>,
    _marker: PhantomData<T>,
}

impl<T> crate::contributor::ContributorBase for Contributor<T>
where
    T: Clone + Send + Sync + Write + commonware_codec::Read<Cfg = ()> + EncodeSize,
{
    type PublicKey = PubKey;
    type Signer = EllipticCurve;
    type Signature = Sig;

    fn is_orchestrator(&self, sender: &Self::PublicKey) -> bool {
        &self.orchestrator == sender
    }

    fn get_contributor_index(&self, public_key: &Self::PublicKey) -> Option<&usize> {
        match &self.aggregation_data {
            Some(data) => data.ordered_contributors.get(public_key),
            None => None,
        }
    }
}

impl<T> Contribute for Contributor<T>
where
    T: Clone + Send + Sync + Write + commonware_codec::Read<Cfg = ()> + EncodeSize,
{
    type AggregationInput = AggregationInput;

    fn new(
        orchestrator: PubKey,
        signer: EllipticCurve,
        mut contributors: Vec<PubKey>,
        aggregation_input: Option<AggregationInput>,
    ) -> Self {
        dotenv().ok();
        contributors.sort();
        let mut ordered_contributors = HashMap::new();
        for (idx, contributor) in contributors.iter().enumerate() {
            ordered_contributors.insert(contributor.clone(), idx);
        }
        let me = *ordered_contributors.get(&signer.public_key()).unwrap();
        if let Some(aggregation_input) = aggregation_input {
            let threshold = aggregation_input.threshold();
            let g1_map = aggregation_input.g1_map().clone();
            Self {
                orchestrator,
                signer,
                me,
                aggregation_data: Some(AggregationData {
                    threshold,
                    g1_map,
                    contributors,
                    ordered_contributors,
                }),
                validator: None,
                _marker: PhantomData,
            }
        } else {
            Self {
                orchestrator,
                signer,
                me,
                aggregation_data: None,
                validator: None,
                _marker: PhantomData,
            }
        }
    }

    async fn run<S, R>(self, mut sender: S, mut receiver: R) -> Result<()>
    where
        S: Sender,
        R: Receiver<PublicKey = PubKey>,
    {
        let mut signed = HashSet::new();
        // Stores (original_metadata, contributor_index -> signature) per round.
        // The metadata is kept so we can reconstruct a valid re-broadcast message
        // when the orchestrator restarts and re-sends Start for a round we already signed.
        let mut signatures: HashMap<u64, (T, HashMap<usize, Sig>)> = HashMap::new();

        let validator = self
            .validator
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Validator not configured for Contributor"))?;

        while let Ok((s, message)) = receiver.recv().await {
            // Parse message
            let Ok(message): Result<wire::Aggregation<T>, _> =
                wire::Aggregation::read(&mut std::io::Cursor::new(message))
            else {
                continue;
            };
            let round = message.round;

            if let Some(AggregationData {
                threshold,
                ref g1_map,
                ref contributors,
                ..
            }) = self.aggregation_data
                && !self.is_orchestrator(&s)
            {
                // Get contributor
                let Some(contributor) = self.get_contributor_index(&s) else {
                    info!("contributor not found: {:?}", s);
                    continue;
                };

                // Check if contributor already signed
                let Some((_, round_sigs)) = signatures.get_mut(&round) else {
                    info!("signatures not found: {:?}", round);
                    continue;
                };
                if round_sigs.contains_key(contributor) {
                    info!("contributor already signed: {:?}", contributor);
                    continue;
                }

                // Extract signature
                let signature = match message.clone().payload {
                    Some(Payload::Signature(signature)) => signature,
                    _ => {
                        info!("signature not found: {:?}", message.clone().payload);
                        continue;
                    }
                };
                let Ok(signature) = Sig::try_from(signature.clone()) else {
                    info!("not a valid signature: {:?}", signature);
                    continue;
                };
                let mut buf = Vec::with_capacity(message.encode_size());
                message.write(&mut buf);
                let Ok(payload) = validator.validate_and_return_expected_hash(&buf).await else {
                    info!(
                        "failed to validate payload for contributor: {:?}",
                        contributor
                    );
                    continue;
                };
                // Verify signature from contributor using aggregate_verify with single public key
                if !aggregate_verify(std::slice::from_ref(&s), None, &payload, &signature) {
                    info!("invalid signature from contributor: {:?}", contributor);
                    continue;
                }

                // Insert signature
                round_sigs.insert(*contributor, signature);

                // Check if should aggregate
                if round_sigs.len() < threshold {
                    info!(
                        "current signatures aggregated: {:?}, needed: {:?}, continuing aggregation",
                        round_sigs.len(),
                        threshold
                    );
                    continue;
                }

                // Enough signatures, aggregate
                let mut participating = Vec::new();
                let mut participating_g1 = Vec::new();
                let mut sigs = Vec::new();
                for (i, contributor) in contributors.iter().enumerate() {
                    let Some(signature) = round_sigs.get(&i) else {
                        continue;
                    };
                    participating.push(contributor.clone());
                    participating_g1.push(g1_map[contributor].clone());
                    sigs.push(signature.clone());
                }
                let Some(agg_signature) = aggregate_signatures(&sigs) else {
                    info!("failed to aggregate signatures");
                    continue;
                };

                // Verify aggregated signature (already verified individual signatures so should never fail)
                if !aggregate_verify(&participating, None, &payload, &agg_signature) {
                    panic!("failed to verify aggregated signature");
                }
                info!(
                    round,
                    msg = hex(&payload),
                    ?participating,
                    signature = hex(&agg_signature),
                    "aggregated signatures",
                );
                continue;
            }

            // Handle message from orchestrator
            match message.payload {
                Some(Payload::Start) => (),
                _ => continue,
            };
            if !self.is_orchestrator(&s) {
                info!("not from orchestrator: {:?}", s);
                continue;
            }

            // Check if already signed at round
            if !signed.insert(round) {
                info!("already signed at round: {:?}, re-broadcasting existing signature", round);
                // Re-broadcast the existing signature so a restarted orchestrator can still
                // collect it. We use the originally-signed metadata so the hash remains valid.
                if let Some((stored_metadata, round_sigs)) = signatures.get(&round) {
                    if let Some(existing_sig) = round_sigs.get(&self.me) {
                        let resend = wire::Aggregation::<T> {
                            round,
                            metadata: stored_metadata.clone(),
                            payload: Some(Payload::Signature(existing_sig.to_vec())),
                        };
                        let mut buf = Vec::with_capacity(resend.encode_size());
                        resend.write(&mut buf);
                        if let Err(e) = sender
                            .send(commonware_p2p::Recipients::All, Bytes::from(buf), true)
                            .await
                        {
                            info!("Failed to re-broadcast existing signature for round {}: {}", round, e);
                        } else {
                            info!(round, "re-broadcast existing signature");
                        }
                    }
                }
                continue;
            }
            let mut buf = Vec::with_capacity(message.encode_size());
            message.write(&mut buf);
            let payload = validator.validate_and_return_expected_hash(&buf).await?;
            info!(
                "Generating signature for round: {}, payload hash: {}",
                round,
                hex(&payload)
            );
            let signature = self.signer.sign(None, &payload);

            // Store signature alongside original metadata (needed for re-broadcast on restart)
            signatures
                .entry(round)
                .or_insert_with(|| (message.metadata.clone(), HashMap::new()))
                .1
                .insert(self.me, signature.clone());

            // Return signature to orchestrator
            let message = wire::Aggregation::<T> {
                round,
                metadata: message.metadata.clone(),
                payload: Some(Payload::Signature(signature.to_vec())),
            };
            let mut buf = Vec::with_capacity(message.encode_size());
            message.write(&mut buf);
            info!("Sending signature for round: {}", round);

            // Broadcast to all (including orchestrator)
            sender
                .send(commonware_p2p::Recipients::All, Bytes::from(buf), true)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to broadcast signature: {}", e))?;
            info!(round, "broadcast signature");
        }

        Ok(())
    }
}

impl<T> Contributor<T>
where
    T: Clone + Send + Sync + Write + commonware_codec::Read<Cfg = ()> + EncodeSize,
{
    pub fn with_validator(mut self, validator: Arc<dyn ValidatorTrait>) -> Self {
        self.validator = Some(validator);
        self
    }
}
