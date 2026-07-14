//! Submitter: turns certified heights into on-chain signature-check calls.
//!
//! Consumes certificate observations from the [`crate::reporter::CertReporter`] and
//! resolves each one:
//!
//! - `skip_digest(namespace, height)`: the quorum abandoned the height — log and
//!   notify the sequencer ([`ResolutionKind::Skipped`]); nothing goes on-chain.
//! - No assignment, or a digest that matches neither the assignment nor the skip
//!   digest: pre-restart leftovers — log and notify ([`ResolutionKind::Foreign`]);
//!   the sequencer re-assigns any in-flight task.
//! - Expected digest: assemble the EigenLayer submission (signer bitmap →
//!   registered G1/G2 keys → operator addresses → `NonSignerStakesAndSignature`)
//!   and call [`BlsSignatureVerificationHandler::handle_verification`], with
//!   bounded retries.
//!
//! The certificate already carries the aggregated signature and signer bitmap
//! assembled by the engine, so no signature aggregation happens here — only the
//! EigenLayer read flow (block pinning, memoized `pubkeyHashToOperator` resolution
//! with parallel cache misses, `getNonSignerStakesAndSignature`) and the handoff to
//! the application's handler.

use crate::executor::{BlsSignatureVerificationHandler, ExecutionResult};
use crate::reporter::{CertifiedHeight, CertifiedReceiver};
use crate::sequencer::{Resolution, ResolutionKind, ResolutionSender, SharedAssignments};
use alloy::eips::BlockId;
use alloy::network::Ethereum;
use alloy::sol_types::SolValue;
use alloy_primitives::{Address, Bytes, FixedBytes, U256, keccak256};
use alloy_provider::Provider;
use anyhow::Result;
use commonware_avs_bindings::ReadOnlyProvider;
use commonware_avs_bindings::bls_apk_registry::BLSApkRegistry::BLSApkRegistryInstance;
use commonware_avs_bindings::bls_sig_check_operator_state_retriever::{
    BLSSigCheckOperatorStateRetriever::BLSSigCheckOperatorStateRetrieverInstance, BN254::G1Point,
};
use commonware_avs_core::bn254::{Bn254Certificate, Bn254Scheme, G1PublicKey, PublicKey};
use commonware_avs_core::wire::{TaskData, skip_digest};
use commonware_cryptography::sha256::Digest;
use eigen_crypto_bls::convert_to_g1_point;
use std::collections::HashMap;
use std::future::IntoFuture;
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Retries after the first failed submission attempt.
const MAX_RETRIES: u32 = 2;

/// Delay before the first retry; doubles per attempt.
const INITIAL_RETRY_BACKOFF: Duration = Duration::from_secs(2);

/// Computes the `BLSApkRegistry` pubkey hash used to key `pubkeyHashToOperator`.
///
/// This is `keccak256(abi_encode(G1Point { X, Y }))` for the operator's G1 public
/// key, matching the hash the registry stores on-chain.
fn pubkey_hash(g1_pubkey: &G1PublicKey) -> Result<FixedBytes<32>> {
    let g1_point = G1Point {
        X: U256::from_str(&g1_pubkey.get_x())
            .map_err(|e| anyhow::anyhow!("Failed to parse X coordinate: {}", e))?,
        Y: U256::from_str(&g1_pubkey.get_y())
            .map_err(|e| anyhow::anyhow!("Failed to parse Y coordinate: {}", e))?,
    };
    Ok(keccak256(g1_point.abi_encode()))
}

/// Consumes certified heights and drives the on-chain submission.
pub struct Submitter<T, H>
where
    T: TaskData,
    H: BlsSignatureVerificationHandler<TaskData = T>,
{
    /// Verifier scheme: maps certificate signer bitmaps to registered keys.
    scheme: Bn254Scheme,
    /// L1 read-side provider (operator state lives on L1 only).
    view_only_provider: ReadOnlyProvider,
    bls_apk_registry: BLSApkRegistryInstance<ReadOnlyProvider, Ethereum>,
    bls_operator_state_retriever:
        BLSSigCheckOperatorStateRetrieverInstance<ReadOnlyProvider, Ethereum>,
    registry_coordinator_address: Address,
    /// Executes the final on-chain verification call.
    handler: H,
    /// Memoized `pubkeyHashToOperator` results: an operator's address never
    /// changes for a registered G2 key, so entries never expire.
    operator_cache: HashMap<PublicKey, Address>,
    /// Height → expected digest + task, written by the sequencer.
    assignments: SharedAssignments<T>,
    /// Verified certificates from the reporter.
    certified: CertifiedReceiver<Bn254Scheme>,
    /// Final dispositions back to the sequencer.
    resolutions: ResolutionSender,
    /// Application namespace bound into [`skip_digest`], matching the namespace
    /// the sequencer uses for the same deployment.
    namespace: Vec<u8>,
    /// Quorum numbers passed to `getNonSignerStakesAndSignature` and the handler
    /// (e.g. `Bytes::from_static(&[0x00])` for quorum 0).
    quorum_numbers: Bytes,
}

impl<T, H> Submitter<T, H>
where
    T: TaskData,
    H: BlsSignatureVerificationHandler<TaskData = T>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        scheme: Bn254Scheme,
        view_only_provider: ReadOnlyProvider,
        bls_apk_registry: BLSApkRegistryInstance<ReadOnlyProvider, Ethereum>,
        bls_operator_state_retriever: BLSSigCheckOperatorStateRetrieverInstance<
            ReadOnlyProvider,
            Ethereum,
        >,
        registry_coordinator_address: Address,
        handler: H,
        assignments: SharedAssignments<T>,
        certified: CertifiedReceiver<Bn254Scheme>,
        resolutions: ResolutionSender,
        namespace: Vec<u8>,
        quorum_numbers: Bytes,
    ) -> Self {
        Self {
            scheme,
            view_only_provider,
            bls_apk_registry,
            bls_operator_state_retriever,
            registry_coordinator_address,
            handler,
            operator_cache: HashMap::new(),
            assignments,
            certified,
            resolutions,
            namespace,
            quorum_numbers,
        }
    }

    /// Runs until the reporter side of the certified channel closes.
    pub async fn run(mut self) {
        while let Some(certified) = self.certified.recv().await {
            self.handle_certified(certified).await;
        }
        info!("certified channel closed; submitter exiting");
    }

    /// Notifies the sequencer of a height's final disposition.
    fn notify(&self, height: u64, kind: ResolutionKind) {
        if self.resolutions.send(Resolution { height, kind }).is_err() {
            // The sequencer is gone; the process is shutting down.
            warn!(height, "resolution channel closed");
        }
    }

    async fn handle_certified(&mut self, certified: CertifiedHeight<Bn254Scheme>) {
        let CertifiedHeight {
            height,
            digest,
            certificate,
        } = certified;

        // Skip certificate: the quorum abandoned the height.
        if digest == skip_digest(&self.namespace, height) {
            info!(height, "skip certificate observed; nothing to submit");
            self.notify(height, ResolutionKind::Skipped);
            return;
        }

        // Look up the sequencer's assignment for this height.
        let assignment = match self.assignments.read() {
            Ok(assignments) => assignments.get(&height).cloned(),
            Err(_) => {
                error!(height, "assignments lock poisoned; treating as unassigned");
                None
            }
        };
        let Some(assignment) = assignment else {
            // Unassigned heights certify during journal replay (pre-restart
            // leftovers) or when nodes resolved a height from a previous router
            // life. There is nothing to submit — the task cache is gone.
            info!(
                height,
                digest = %digest,
                "certificate for unassigned height; nothing to submit"
            );
            self.notify(height, ResolutionKind::Foreign);
            return;
        };
        if assignment.digest != digest {
            // A certificate formed for a digest we did not announce (a pre-restart
            // leftover): the height is consumed, the in-flight task is not.
            warn!(
                height,
                expected = %assignment.digest,
                certified = %digest,
                "certificate digest does not match assignment; height consumed"
            );
            self.notify(height, ResolutionKind::Foreign);
            return;
        }

        // Expected digest: submit on-chain with bounded retries. Failures here
        // are either transient RPC issues (retry helps) or deterministic
        // rejections (retries burn two extra round-trips, then the height is
        // released as a failed execution).
        let mut backoff = INITIAL_RETRY_BACKOFF;
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            match self
                .submit(height, digest, &certificate, &assignment.task)
                .await
            {
                Ok(result) => {
                    info!(
                        height,
                        tx = %result.transaction_hash,
                        block = ?result.block_number,
                        status = ?result.status,
                        "verification submitted"
                    );
                    let success = result.status.unwrap_or(true);
                    self.notify(height, ResolutionKind::Executed { success });
                    return;
                }
                Err(error) if attempt <= MAX_RETRIES => {
                    warn!(
                        height,
                        attempt,
                        max_retries = MAX_RETRIES,
                        %error,
                        backoff_secs = backoff.as_secs_f64(),
                        "submission failed; retrying"
                    );
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                }
                Err(error) => {
                    error!(
                        height,
                        attempts = attempt,
                        %error,
                        "submission failed after retries; releasing height"
                    );
                    self.notify(height, ResolutionKind::Executed { success: false });
                    return;
                }
            }
        }
    }

    /// One end-to-end submission attempt for a certified task digest.
    async fn submit(
        &mut self,
        height: u64,
        digest: Digest,
        certificate: &Bn254Certificate,
        task: &T,
    ) -> Result<ExecutionResult> {
        // Bitmap → registered keys, in participant order. The certificate was
        // verified by the reporter, so the bitmap length matches the set.
        let participating = self.scheme.signer_g2_keys(&certificate.signers);
        let participating_g1 = self.scheme.signer_g1_points(&certificate.signers);
        if participating.is_empty() || participating.len() != participating_g1.len() {
            return Err(anyhow::anyhow!(
                "signer bitmap resolved to an inconsistent key set ({} G2 / {} G1)",
                participating.len(),
                participating_g1.len()
            ));
        }

        // The certificate's aggregated G1 signature is the on-chain sigma.
        let sigma = convert_to_g1_point(certificate.signature.get_point())
            .map_err(|e| anyhow::anyhow!("Failed to convert to G1 point: {}", e))?;
        let sigma_struct = G1Point {
            X: U256::from_str(&sigma.X.to_string())
                .map_err(|e| anyhow::anyhow!("Failed to parse X coordinate: {}", e))?,
            Y: U256::from_str(&sigma.Y.to_string())
                .map_err(|e| anyhow::anyhow!("Failed to parse Y coordinate: {}", e))?,
        };

        let msg_hash = FixedBytes::<32>::from_slice(digest.as_ref());

        // Fetch the current block number once up front. All subsequent eth_calls
        // are pinned to this block so the entire verification reads a consistent
        // EVM state snapshot — both the operator registry lookups and the
        // non-signer stakes call.
        let current_block_number = self
            .view_only_provider
            .get_block_number()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get block number: {}", e))?;
        let reference_block = BlockId::from(current_block_number);

        // Resolve an operator address for every participating signer.
        //
        // Phase 1: take cache hits synchronously. Cache misses — the first
        // certificate, or whenever a new operator joins — each need a
        // `pubkeyHashToOperator` RPC call, so record their slot index and
        // precomputed pubkey hash.
        let mut operators: Vec<Option<Address>> = Vec::with_capacity(participating.len());
        let mut misses: Vec<(usize, PublicKey, FixedBytes<32>)> = Vec::new();
        for (index, (operator, g1_pubkey)) in participating
            .iter()
            .zip(participating_g1.iter())
            .enumerate()
        {
            match self.operator_cache.get(operator) {
                Some(address) => operators.push(Some(*address)),
                None => {
                    operators.push(None);
                    misses.push((index, operator.clone(), pubkey_hash(g1_pubkey)?));
                }
            }
        }

        // Phase 2: resolve all cache misses with a single round-trip of latency
        // by firing the lookups concurrently, then write them back to the cache.
        // Steady-state (all operators cached) skips this entirely.
        if !misses.is_empty() {
            debug!(
                height,
                cache_misses = misses.len(),
                "resolving operator addresses for cache misses in parallel"
            );
            let calls: Vec<_> = misses
                .iter()
                .map(|(_, _, hash)| {
                    self.bls_apk_registry
                        .pubkeyHashToOperator(*hash)
                        .block(reference_block)
                })
                .collect();
            let resolved =
                futures::future::join_all(calls.iter().map(|call| call.call().into_future())).await;

            for ((index, operator, _), result) in misses.into_iter().zip(resolved) {
                let address = result.map_err(|e| {
                    anyhow::anyhow!("Failed to get operator from pubkey hash: {}", e)
                })?;
                self.operator_cache.insert(operator, address);
                operators[index] = Some(address);
            }
        }

        let operators = operators
            .into_iter()
            .collect::<Option<Vec<Address>>>()
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve all operator addresses"))?;

        // Pinning the call to reference_block ensures the eth_call executes
        // against the same EVM state snapshot used for operator resolution above.
        // The retriever's method returns the plain `NonSignerStakesAndSignature`
        // struct directly (no `FooReturn` wrapper) under this alloy version.
        let non_signer_data = self
            .bls_operator_state_retriever
            .getNonSignerStakesAndSignature(
                self.registry_coordinator_address,
                self.quorum_numbers.clone(),
                sigma_struct,
                operators,
                current_block_number
                    .try_into()
                    .map_err(|e| anyhow::anyhow!("block number overflows u32: {}", e))?,
            )
            .block(reference_block)
            .call()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get non-signer stakes and signature: {}", e))?;

        let block_u32: u32 = current_block_number
            .try_into()
            .map_err(|e| anyhow::anyhow!("Failed to convert block number: {}", e))?;

        self.handler
            .handle_verification(
                height,
                msg_hash,
                self.quorum_numbers.clone(),
                block_u32,
                non_signer_data,
                Some(task),
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a `G1PublicKey` from decimal affine coordinates.
    fn g1(x: &str, y: &str) -> G1PublicKey {
        G1PublicKey::create_from_g1_coordinates(x, y).expect("valid on-curve G1 coordinates")
    }

    #[test]
    fn pubkey_hash_matches_keccak_of_abi_encoded_point() {
        // BN254 G1 generator (1, 2).
        let pk = g1("1", "2");
        let expected = keccak256(
            G1Point {
                X: U256::from(1u8),
                Y: U256::from(2u8),
            }
            .abi_encode(),
        );
        assert_eq!(pubkey_hash(&pk).unwrap(), expected);
    }

    #[test]
    fn pubkey_hash_is_deterministic_and_distinct_per_point() {
        // Generator G and 2G — distinct on-curve points.
        let g = g1("1", "2");
        let two_g = g1(
            "1368015179489954701390400359078579693043519447331113978918064868415326638035",
            "9918110051302171585080402603319702774565515993150576347155970296011118125764",
        );
        assert_eq!(
            pubkey_hash(&g).unwrap(),
            pubkey_hash(&g).unwrap(),
            "hash must be deterministic"
        );
        assert_ne!(
            pubkey_hash(&g).unwrap(),
            pubkey_hash(&two_g).unwrap(),
            "distinct points must hash distinctly"
        );
    }
}
