use alloy::sol_types::SolValue;
use alloy::{network::Ethereum, providers::Provider};
use alloy_primitives::{Address, Bytes, FixedBytes, U256};
use anyhow::Result;
use async_trait::async_trait;
use commonware_avs_core::bn254::{G1PublicKey, PublicKey, Signature, get_points};
use commonware_utils::hex;
use eigen_crypto_bls::convert_to_g1_point;
use std::{collections::HashMap, str::FromStr};
use tracing::debug;

use super::traits::{BlsExecutorTrait, BlsSignatureVerificationHandler};
use super::types::BlsVerificationData;
use crate::executor::{ExecutionResult, VerificationData, VerificationExecutor};
use commonware_avs_bindings::{
    ReadOnlyProvider,
    blsapkregistry::BLSApkRegistry::BLSApkRegistryInstance,
    blssigcheckoperatorstateretriever::{
        BLSSigCheckOperatorStateRetriever::{
            BLSSigCheckOperatorStateRetrieverInstance, getNonSignerStakesAndSignatureReturn,
        },
        BN254::G1Point,
    },
};

/// Configuration for read-side blockchain operations on a specific chain.
/// Contains the provider and EigenLayer contract instances needed to look up
/// operator state and block numbers.
pub struct ReadSideConfig {
    pub provider: ReadOnlyProvider,
    pub bls_apk_registry: BLSApkRegistryInstance<ReadOnlyProvider, Ethereum>,
    pub bls_operator_state_retriever:
        BLSSigCheckOperatorStateRetrieverInstance<ReadOnlyProvider, Ethereum>,
    pub registry_coordinator_address: Address,
}

pub struct BlsEigenlayerExecutor<H> {
    view_only_provider: ReadOnlyProvider,
    bls_apk_registry: BLSApkRegistryInstance<ReadOnlyProvider, Ethereum>,
    bls_operator_state_retriever:
        BLSSigCheckOperatorStateRetrieverInstance<ReadOnlyProvider, Ethereum>,
    registry_coordinator_address: Address,
    contract_handler: H,
    g1_hash_map: HashMap<PublicKey, Address>,
    alternate_read_sides: HashMap<String, ReadSideConfig>,
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
            alternate_read_sides: HashMap::new(),
        }
    }

    /// Register an alternate read-side configuration for a specific chain.
    /// The key is used by `BlsSignatureVerificationHandler::resolve_read_side()`
    /// to select which config to use per-task.
    pub fn add_read_side(mut self, key: String, config: ReadSideConfig) -> Self {
        self.alternate_read_sides.insert(key, config);
        self
    }

}

async fn lookup_operator_address(
    g1_hash_map: &mut HashMap<PublicKey, Address>,
    contributor: &PublicKey,
    g1_pubkey: &G1PublicKey,
    registry: &BLSApkRegistryInstance<ReadOnlyProvider, Ethereum>,
) -> Result<Address> {
    if let Some(address) = g1_hash_map.get(contributor) {
        return Ok(*address);
    }

    let g1_point = G1Point {
        X: U256::from_str(&g1_pubkey.get_x())
            .map_err(|e| anyhow::anyhow!("Failed to parse X coordinate: {}", e))?,
        Y: U256::from_str(&g1_pubkey.get_y())
            .map_err(|e| anyhow::anyhow!("Failed to parse Y coordinate: {}", e))?,
    };
    let hex_string = format!(
        "0x{}",
        hex(alloy_primitives::keccak256(g1_point.abi_encode()).as_ref())
    );
    let address = registry
        .pubkeyHashToOperator(
            FixedBytes::<32>::from_str(&hex_string)
                .map_err(|e| anyhow::anyhow!("Failed to parse hex string: {}", e))?,
        )
        .call()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get operator from pubkey hash: {}", e))?;

    g1_hash_map.insert(contributor.clone(), address);
    Ok(address)
}

#[async_trait]
impl<H: BlsSignatureVerificationHandler> VerificationExecutor<H::TaskData, VerificationData>
    for BlsEigenlayerExecutor<H>
{
    async fn execute_verification(
        &mut self,
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
                Signature::try_from(bytes.as_slice())
                    .map_err(|e| anyhow::anyhow!("Failed to deserialize signature: {:?}", e))
            })
            .collect::<Result<Vec<_>>>()?;

        // Deserialize public keys
        let public_keys: Vec<PublicKey> = verification_data
            .public_keys
            .iter()
            .map(|bytes| {
                PublicKey::try_from(bytes.as_slice())
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

        self.execute_bls_verification(payload_hash, bls_verification_data, task_data)
            .await
    }
}

#[async_trait]
impl<H: BlsSignatureVerificationHandler> BlsExecutorTrait<H::TaskData>
    for BlsEigenlayerExecutor<H>
{
    async fn execute_bls_verification(
        &mut self,
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

        // Resolve which read-side config to use for this task
        let read_side_key = self.contract_handler.resolve_read_side(task_data);
        let alt_config = read_side_key
            .as_ref()
            .and_then(|key| self.alternate_read_sides.get(key));

        // Select provider, registry, retriever, and coordinator from alternate or default
        let provider = alt_config
            .map(|c| &c.provider)
            .unwrap_or(&self.view_only_provider);
        let registry = alt_config
            .map(|c| &c.bls_apk_registry)
            .unwrap_or(&self.bls_apk_registry);
        let retriever = alt_config
            .map(|c| &c.bls_operator_state_retriever)
            .unwrap_or(&self.bls_operator_state_retriever);
        let coordinator_address = alt_config
            .map(|c| c.registry_coordinator_address)
            .unwrap_or(self.registry_coordinator_address);

        // Get or populate operator addresses
        let mut operators = Vec::new();
        for (contributor, g1_pubkey) in participating.iter().zip(participating_g1.iter()) {
            let address =
                lookup_operator_address(&mut self.g1_hash_map, contributor, g1_pubkey, registry)
                    .await?;
            operators.push(address);
        }

        let current_block_number = provider
            .get_block_number()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get block number: {}", e))?;
        let quorum_numbers = Bytes::from_str("0x00")
            .map_err(|e| anyhow::anyhow!("Failed to parse quorum numbers: {}", e))?;

        // Call the BLS operator state retriever to get the non-signer data
        let non_signer_result = retriever
            .getNonSignerStakesAndSignature(
                coordinator_address,
                quorum_numbers.clone(),
                sigma_struct,
                operators,
                current_block_number as u32,
            )
            .call()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get non-signer stakes and signature: {}", e))?;

        // Wrap the result to match the trait signature
        let non_signer_return = getNonSignerStakesAndSignatureReturn {
            _0: non_signer_result,
        };

        // Delegate the contract-specific execution to the handler
        let result = self
            .contract_handler
            .handle_verification(
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
