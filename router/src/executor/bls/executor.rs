use crate::executor::core::{ExecutionResult, VerificationData, VerificationExecutor};
use alloy::providers::Provider;
use alloy::sol_types::SolValue;
use alloy_primitives::{Address, Bytes, FixedBytes, U256};
use anyhow::Result;
use async_trait::async_trait;
use bn254::{G1PublicKey, PublicKey};
use commonware_avs_bindings::{
    ReadOnlyProvider,
    blsapkregistry::BLSApkRegistry::BLSApkRegistryInstance,
    blssigcheckoperatorstateretriever::{
        BLSSigCheckOperatorStateRetriever::BLSSigCheckOperatorStateRetrieverInstance,
        BN254::G1Point,
    },
};
use commonware_utils::hex;
use eigen_crypto_bls::convert_to_g1_point;
use std::{collections::HashMap, str::FromStr};
use tracing::debug;

use super::traits::{BlsExecutorTrait, BlsSignatureVerificationHandler};
use super::types::BlsVerificationData;

pub struct BlsEigenlayerExecutor<H> {
    view_only_provider: ReadOnlyProvider,
    bls_apk_registry: BLSApkRegistryInstance<(), ReadOnlyProvider>,
    bls_operator_state_retriever: BLSSigCheckOperatorStateRetrieverInstance<(), ReadOnlyProvider>,
    registry_coordinator_address: Address,
    contract_handler: H,
    g1_hash_map: HashMap<PublicKey, Address>,
}

impl<H> BlsEigenlayerExecutor<H> {
    pub fn new(
        view_only_provider: ReadOnlyProvider,
        bls_apk_registry: BLSApkRegistryInstance<(), ReadOnlyProvider>,
        bls_operator_state_retriever: BLSSigCheckOperatorStateRetrieverInstance<
            (),
            ReadOnlyProvider,
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
        }
    }

    async fn ensure_g1_hash_map_entry(
        &mut self,
        contributor: &PublicKey,
        g1_pubkey: &G1PublicKey,
    ) -> Result<Address> {
        if let Some(address) = self.g1_hash_map.get(contributor) {
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
        let address = self
            .bls_apk_registry
            .pubkeyHashToOperator(
                FixedBytes::<32>::from_str(&hex_string)
                    .map_err(|e| anyhow::anyhow!("Failed to parse hex string: {}", e))?,
            )
            .call()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get operator from pubkey hash: {}", e))?
            .operator;

        self.g1_hash_map.insert(contributor.clone(), address);
        Ok(address)
    }
}

#[async_trait]
impl<H: BlsSignatureVerificationHandler> VerificationExecutor<H::TaskData>
    for BlsEigenlayerExecutor<H>
{
    async fn execute_verification(
        &mut self,
        payload_hash: &[u8],
        verification_data: VerificationData,
        task_data: Option<&H::TaskData>,
    ) -> Result<ExecutionResult> {
        let g1_public_keys = if let Some(context) = verification_data.context {
            // Each G1 public key is stored in compressed format (32 bytes)
            const G1_COMPRESSED_SIZE: usize = 32;
            let num_public_keys = verification_data.public_keys.len();
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
                // Deserialize the compressed G1 public key directly
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

        let bls_verification_data = BlsVerificationData::new(
            verification_data.signatures,
            verification_data.public_keys,
            g1_public_keys,
        );

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
        let (_apk, _apk_g2, asig) = bn254::get_points(participating_g1, participating, signatures)
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

        // Get or populate operator addresses
        let mut operators = Vec::new();
        for (contributor, g1_pubkey) in participating.iter().zip(participating_g1.iter()) {
            let address = self
                .ensure_g1_hash_map_entry(contributor, g1_pubkey)
                .await?;
            operators.push(address);
        }

        let current_block_number = self
            .view_only_provider
            .get_block_number()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get block number: {}", e))?;
        let quorum_numbers = Bytes::from_str("0x00")
            .map_err(|e| anyhow::anyhow!("Failed to parse quorum numbers: {}", e))?;

        // Call the BLS operator state retriever to get the non-signer data
        let non_signer_return = self
            .bls_operator_state_retriever
            .getNonSignerStakesAndSignature(
                self.registry_coordinator_address,
                quorum_numbers.clone(),
                sigma_struct,
                operators,
                current_block_number as u32,
            )
            .call()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get non-signer stakes and signature: {}", e))?;

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
