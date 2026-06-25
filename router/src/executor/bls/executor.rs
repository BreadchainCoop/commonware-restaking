use alloy::eips::BlockId;
use alloy::sol_types::SolValue;
use alloy::{network::Ethereum, providers::Provider};
use alloy_primitives::{Address, Bytes, FixedBytes, U256};
use anyhow::Result;
use async_trait::async_trait;
use commonware_avs_core::bn254::{G1PublicKey, PublicKey, Signature, get_points};
use commonware_runtime::Metrics;
use eigen_crypto_bls::convert_to_g1_point;
use std::{collections::HashMap, future::IntoFuture, str::FromStr, time::Instant};
use tracing::debug;

use super::metrics::ExecutorMetrics;
use super::traits::{BlsExecutorTrait, BlsSignatureVerificationHandler};
use super::types::BlsVerificationData;
use crate::executor::{ExecutionResult, VerificationData, VerificationExecutor};
use commonware_avs_bindings::{
    ReadOnlyProvider,
    bls_apk_registry::BLSApkRegistry::BLSApkRegistryInstance,
    bls_sig_check_operator_state_retriever::{
        BLSSigCheckOperatorStateRetriever::{
            BLSSigCheckOperatorStateRetrieverInstance, getNonSignerStakesAndSignatureReturn,
        },
        BN254::G1Point,
    },
};

pub struct BlsEigenlayerExecutor<H> {
    view_only_provider: ReadOnlyProvider,
    bls_apk_registry: BLSApkRegistryInstance<ReadOnlyProvider, Ethereum>,
    bls_operator_state_retriever:
        BLSSigCheckOperatorStateRetrieverInstance<ReadOnlyProvider, Ethereum>,
    registry_coordinator_address: Address,
    contract_handler: H,
    g1_hash_map: HashMap<PublicKey, Address>,
    metrics: Option<ExecutorMetrics>,
}

impl<H> BlsEigenlayerExecutor<H> {
    pub fn new(
        view_only_provider: ReadOnlyProvider,
        bls_apk_registry: BLSApkRegistryInstance<ReadOnlyProvider, Ethereum>,
        bls_operator_state_retriever: BLSSigCheckOperatorStateRetrieverInstance<
            ReadOnlyProvider,
            Ethereum,
        >,
        registry_coordinator_address: Address,
        contract_handler: H,
    ) -> Self {
        Self {
            view_only_provider,
            bls_apk_registry,
            bls_operator_state_retriever,
            registry_coordinator_address,
            contract_handler,
            g1_hash_map: HashMap::new(),
            metrics: None,
        }
    }

    /// Registers executor metrics on `context` (under an `executor` label scope)
    /// and enables observation of EigenLayer state-retrieval timing and operator
    /// cache misses.
    pub fn with_metrics(mut self, context: &impl Metrics) -> Self {
        self.metrics = Some(ExecutorMetrics::new(context));
        self
    }
}

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
    Ok(alloy_primitives::keccak256(g1_point.abi_encode()))
}

#[async_trait]
impl<H: BlsSignatureVerificationHandler> VerificationExecutor<H::TaskData, VerificationData>
    for BlsEigenlayerExecutor<H>
{
    async fn execute_verification(
        &mut self,
        round: u64,
        payload_hash: &[u8],
        verification_data: VerificationData,
        task_data: Option<&H::TaskData>,
    ) -> Result<ExecutionResult> {
        // Convert generic VerificationData to BLS-specific BlsVerificationData
        // Deserialize signatures
        let signatures: Vec<Signature> = verification_data
            .signatures
            .iter()
            .map(|bytes| {
                Signature::try_from(bytes.as_ref())
                    .map_err(|e| anyhow::anyhow!("Failed to deserialize signature: {:?}", e))
            })
            .collect::<Result<Vec<_>>>()?;

        // Deserialize public keys
        let public_keys: Vec<PublicKey> = verification_data
            .public_keys
            .iter()
            .map(|bytes| {
                PublicKey::try_from(bytes.as_ref())
                    .map_err(|e| anyhow::anyhow!("Failed to deserialize public key: {:?}", e))
            })
            .collect::<Result<Vec<_>>>()?;

        // Deserialize G1 public keys from context
        let g1_public_keys = if let Some(context) = verification_data.context {
            const G1_COMPRESSED_SIZE: usize = 32;
            let num_public_keys = public_keys.len();
            if num_public_keys == 0 {
                return Err(anyhow::anyhow!("No public keys provided"));
            }

            if context.len() != num_public_keys * G1_COMPRESSED_SIZE {
                return Err(anyhow::anyhow!(
                    "Invalid context length: {} does not match expected size for {} public keys ({} bytes each)",
                    context.len(),
                    num_public_keys,
                    G1_COMPRESSED_SIZE
                ));
            }

            let mut g1_keys = Vec::new();
            for chunk in context.chunks(G1_COMPRESSED_SIZE) {
                let g1_pubkey = G1PublicKey::try_from(chunk)
                    .map_err(|e| anyhow::anyhow!("Failed to deserialize G1 public key: {:?}", e))?;
                g1_keys.push(g1_pubkey);
            }
            g1_keys
        } else {
            return Err(anyhow::anyhow!(
                "BLS verification requires G1 public keys in context"
            ));
        };

        let bls_verification_data =
            BlsVerificationData::new(signatures, public_keys, g1_public_keys);

        self.execute_bls_verification(round, payload_hash, bls_verification_data, task_data)
            .await
    }
}

/// Runs BLS verification on the provided `BlsVerificationData`.
#[async_trait]
impl<H: BlsSignatureVerificationHandler> VerificationExecutor<H::TaskData, BlsVerificationData>
    for BlsEigenlayerExecutor<H>
{
    async fn execute_verification(
        &mut self,
        round: u64,
        payload_hash: &[u8],
        verification_data: BlsVerificationData,
        task_data: Option<&H::TaskData>,
    ) -> Result<ExecutionResult> {
        self.execute_bls_verification(round, payload_hash, verification_data, task_data)
            .await
    }
}

#[async_trait]
impl<H: BlsSignatureVerificationHandler> BlsExecutorTrait<H::TaskData>
    for BlsEigenlayerExecutor<H>
{
    async fn execute_bls_verification(
        &mut self,
        round: u64,
        payload_hash: &[u8],
        verification_data: BlsVerificationData,
        task_data: Option<&H::TaskData>,
    ) -> Result<ExecutionResult> {
        let participating_g1 = &verification_data.g1_public_keys;
        let participating = &verification_data.public_keys;
        let signatures = &verification_data.signatures;
        let (_apk, _apk_g2, asig) = get_points(participating_g1, participating, signatures)
            .ok_or_else(|| anyhow::anyhow!("Failed to get points"))?;
        let asig_g1 = convert_to_g1_point(asig)
            .map_err(|e| anyhow::anyhow!("Failed to convert to G1 point: {}", e))?;
        let sigma_struct = G1Point {
            X: U256::from_str(&asig_g1.X.to_string())
                .map_err(|e| anyhow::anyhow!("Failed to parse X coordinate: {}", e))?,
            Y: U256::from_str(&asig_g1.Y.to_string())
                .map_err(|e| anyhow::anyhow!("Failed to parse Y coordinate: {}", e))?,
        };

        let msg_hash = FixedBytes::<32>::from_slice(payload_hash);

        // Measures the full EigenLayer read stretch: the current-block query,
        // operator-address resolution, and the non-signer stakes/signature retrieval.
        let retrieval_start = Instant::now();

        // Fetch the current block number once up front. All subsequent eth_calls are
        // pinned to this block so the entire verification reads a consistent EVM state
        // snapshot — both the operator registry lookups and the non-signer stakes call.
        let current_block_number = self
            .view_only_provider
            .get_block_number()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get block number: {}", e))?;
        let reference_block = BlockId::from(current_block_number);

        // Resolve an operator address for every participant.
        //
        // Phase 1: take cache hits synchronously. Cache misses — the first round,
        // or whenever a new operator joins — each need a `pubkeyHashToOperator`
        // RPC call, so record their slot index and precomputed pubkey hash.
        let mut operators: Vec<Option<Address>> = Vec::with_capacity(participating.len());
        let mut misses: Vec<(usize, PublicKey, FixedBytes<32>)> = Vec::new();
        for (index, (contributor, g1_pubkey)) in participating
            .iter()
            .zip(participating_g1.iter())
            .enumerate()
        {
            match self.g1_hash_map.get(contributor) {
                Some(address) => operators.push(Some(*address)),
                None => {
                    operators.push(None);
                    misses.push((index, contributor.clone(), pubkey_hash(g1_pubkey)?));
                }
            }
        }

        // Phase 2: resolve all cache misses with a single round-trip of latency
        // by firing the lookups concurrently, then write them back to the cache.
        // Steady-state (all operators cached) skips this entirely.
        if !misses.is_empty() {
            debug!(
                round,
                cache_misses = misses.len(),
                "resolving operator addresses for cache misses in parallel"
            );
            if let Some(m) = &self.metrics {
                m.operator_cache_misses.inc_by(misses.len() as u64);
            }
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

            for ((index, contributor, _), result) in misses.into_iter().zip(resolved) {
                let address = result.map_err(|e| {
                    anyhow::anyhow!("Failed to get operator from pubkey hash: {}", e)
                })?;
                self.g1_hash_map.insert(contributor, address);
                operators[index] = Some(address);
            }
        }

        let operators = operators
            .into_iter()
            .collect::<Option<Vec<Address>>>()
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve all operator addresses"))?;

        let quorum_numbers = Bytes::from_str("0x00")
            .map_err(|e| anyhow::anyhow!("Failed to parse quorum numbers: {}", e))?;

        // Pinning the call to reference_block ensures the eth_call executes against
        // the same EVM state snapshot used for operator resolution above.
        let non_signer_result = self
            .bls_operator_state_retriever
            .getNonSignerStakesAndSignature(
                self.registry_coordinator_address,
                quorum_numbers.clone(),
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

        // Wrap the result to match the trait signature
        let non_signer_return = getNonSignerStakesAndSignatureReturn {
            _0: non_signer_result,
        };

        if let Some(m) = &self.metrics {
            m.state_retrieval
                .observe(retrieval_start.elapsed().as_secs_f64());
        }

        // Delegate the contract-specific execution to the handler
        let result = self
            .contract_handler
            .handle_verification(
                round,
                msg_hash,
                quorum_numbers,
                current_block_number
                    .try_into()
                    .map_err(|e| anyhow::anyhow!("Failed to convert block number: {}", e))?,
                non_signer_return,
                task_data,
            )
            .await?;

        debug!(
            transaction_hash = %result.transaction_hash,
            block_number = ?result.block_number,
            gas_used = ?result.gas_used,
            status = ?result.status,
            contract_address = ?result.contract_address,
            "Contract execution result"
        );

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::providers::ProviderBuilder;
    use alloy::sol_types::SolValue;
    use alloy_provider::mock::Asserter;
    use alloy_primitives::U64;
    use commonware_avs_bindings::{
        bls_apk_registry::BLSApkRegistry,
        bls_sig_check_operator_state_retriever::{
            BLSSigCheckOperatorStateRetriever, BN254, IBLSSignatureCheckerTypes,
        },
    };
    use commonware_avs_core::bn254::Bn254;
    use commonware_cryptography::Signer;
    use std::sync::{Arc, Mutex};

    struct CapturingHandler(Arc<Mutex<Option<u32>>>);

    #[async_trait]
    impl BlsSignatureVerificationHandler for CapturingHandler {
        type TaskData = ();
        async fn handle_verification(
            &mut self,
            _round: u64,
            _msg_hash: FixedBytes<32>,
            _quorum_numbers: Bytes,
            current_block_number: u32,
            _non_signer_data: getNonSignerStakesAndSignatureReturn,
            _task_data: Option<&Self::TaskData>,
        ) -> anyhow::Result<ExecutionResult> {
            *self.0.lock().unwrap() = Some(current_block_number);
            Ok(ExecutionResult {
                transaction_hash: "0x".to_string(),
                block_number: None,
                gas_used: None,
                status: Some(true),
                contract_address: None,
            })
        }
    }

    /// Asserts that `execute_bls_verification` fetches the block number once and
    /// passes that same value as `current_block_number` to the handler, confirming
    /// the block used for eth_call pinning and the argument to the contract are
    /// consistent with each other.
    #[tokio::test]
    async fn execute_bls_verification_passes_fetched_block_to_handler() {
        use ark_bn254::Fr as Scalar;

        let signer = Bn254::from_scalar(Scalar::from(1u64));
        let g1_key = signer.public_g1();
        let g2_key = signer.public_key();
        let signature = signer.sign(None, &[0x42u8; 32]);

        const EXPECTED_BLOCK: u64 = 99_999;

        // Push responses in the order execute_bls_verification issues them:
        // 1. eth_blockNumber
        // 2. eth_call for pubkeyHashToOperator (first-round cache miss)
        // 3. eth_call for getNonSignerStakesAndSignature
        let asserter = Asserter::new();
        asserter.push_success(&U64::from(EXPECTED_BLOCK));
        asserter.push_success(&Bytes::from(Address::ZERO.abi_encode()));
        let mock_return = IBLSSignatureCheckerTypes::NonSignerStakesAndSignature {
            nonSignerQuorumBitmapIndices: vec![],
            nonSignerPubkeys: vec![],
            quorumApks: vec![BN254::G1Point {
                X: U256::ZERO,
                Y: U256::ZERO,
            }],
            apkG2: BN254::G2Point {
                X: [U256::ZERO, U256::ZERO],
                Y: [U256::ZERO, U256::ZERO],
            },
            sigma: BN254::G1Point {
                X: U256::ZERO,
                Y: U256::ZERO,
            },
            quorumApkIndices: vec![0u32],
            totalStakeIndices: vec![0u32],
            nonSignerStakeIndices: vec![vec![]],
        };
        asserter.push_success(&Bytes::from(mock_return.abi_encode()));

        let provider = ProviderBuilder::new().connect_mocked_client(asserter);
        let bls_apk = BLSApkRegistry::new(Address::ZERO, provider.clone());
        let retriever = BLSSigCheckOperatorStateRetriever::new(Address::ZERO, provider.clone());

        let captured: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
        let mut executor = BlsEigenlayerExecutor::new(
            provider,
            bls_apk,
            retriever,
            Address::ZERO,
            CapturingHandler(captured.clone()),
        );

        let data = BlsVerificationData::new(vec![signature], vec![g2_key], vec![g1_key]);

        executor
            .execute_bls_verification(0, &[0x42u8; 32], data, None)
            .await
            .expect("execute_bls_verification should succeed with mock responses");

        assert_eq!(
            *captured.lock().unwrap(),
            Some(EXPECTED_BLOCK as u32),
            "handler must receive the block number returned by get_block_number()"
        );
    }

    /// Builds a `G1PublicKey` from decimal affine coordinates.
    fn g1(x: &str, y: &str) -> G1PublicKey {
        G1PublicKey::create_from_g1_coordinates(x, y).expect("valid on-curve G1 coordinates")
    }

    #[test]
    fn pubkey_hash_matches_keccak_of_abi_encoded_point() {
        // BN254 G1 generator (1, 2).
        let pk = g1("1", "2");
        let expected = alloy_primitives::keccak256(
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
