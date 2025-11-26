use crate::bn254::{G1PublicKey, PublicKey};
use alloy_primitives::{Address, U256, address};
use alloy_provider::Provider;
use eigen_client_avsregistry::reader::AvsRegistryChainReader;
use eigen_common::get_provider;
use eigen_crypto_bls::{BlsG1Point, BlsG2Point};
use eigen_services_operatorsinfo::operator_info::OperatorInfoService;
use eigen_services_operatorsinfo::operatorsinfo_inmemory::OperatorInfoServiceInMemory;
use eigen_utils::rewardsv2::middleware::operator_state_retriever::OperatorStateRetriever;
use serde_json::Value;
use std::fs;
use std::sync::Arc;

#[derive(Debug)]
pub struct OperatorPubKeys {
    pub g1_pub_key: BlsG1Point,
    pub g2_pub_key: BlsG2Point,
}

#[derive(Clone)]
pub struct CommonwarePublicKeys {
    pub g1_pub_key: G1PublicKey,
    pub g2_pub_key: PublicKey,
}

impl CommonwarePublicKeys {
    pub fn from_string_coordinates(
        g2x1: &str,
        g2x2: &str,
        g2y1: &str,
        g2y2: &str,
        g1x: &str,
        g1y: &str,
    ) -> Option<Self> {
        let g2_pub_key = PublicKey::create_from_g2_coordinates(g2x1, g2x2, g2y1, g2y2)?;
        let g1_pub_key = G1PublicKey::create_from_g1_coordinates(g1x, g1y)?;
        Some(Self {
            g1_pub_key,
            g2_pub_key,
        })
    }
    pub fn from_bls_keys(g1_pub_key: BlsG1Point, g2_pub_key: BlsG2Point) -> Self {
        let g1_pub_key = G1PublicKey::from(g1_pub_key.g1());
        let g2_pub_key = PublicKey::from(g2_pub_key.g2());
        Self {
            g1_pub_key,
            g2_pub_key,
        }
    }
}

impl std::fmt::Debug for CommonwarePublicKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommonwarePublicKeys")
            .field("g1_pub_key", &format!("{:?}", self.g1_pub_key))
            .field("g2_pub_key", &format!("{:?}", self.g2_pub_key))
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct OperatorInfo {
    pub address: Address,
    pub stake: U256,
    pub pub_keys: Option<CommonwarePublicKeys>,
    pub socket: Option<String>,
    pub quorum_number: u8,
}

#[derive(Debug)]
pub struct QuorumInfo {
    pub quorum_number: u8,
    pub operator_count: usize,
    pub total_stake: U256,
    pub operators: Vec<OperatorInfo>,
}

/// source: https://github.com/Layr-Labs/eigenlayer-middleware
/// Contracts from middleware are supposed to be deployed for each AVS but
/// OperatorStateRetriever looks generic for everyone.
const OPERATOR_STATE_RETRIEVER_ADDRESS: Address =
    address!("0xB4baAfee917fb4449f5ec64804217bccE9f46C67"); // TODO Add handling for different chains

pub struct EigenStakingClient {
    http_endpoint: String,
    registry_coordinator_address: Address,
    registry_coordinator_deploy_block: u64,
    operator_info_service: Arc<OperatorInfoServiceInMemory>,
}

#[derive(Debug)]
pub struct AvsDeploymentConfig {
    pub registry_coordinator_address: Address,
    pub deploy_block: u64,
}

impl EigenStakingClient {
    fn read_avs_deployment_config(
        path: &str,
    ) -> Result<AvsDeploymentConfig, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&contents)?;

        let addresses = json["addresses"]
            .as_object()
            .ok_or("Missing addresses in deployment config")?;

        let registry_coordinator = addresses["registryCoordinator"]
            .as_str()
            .ok_or("Missing registryCoordinator address")?;

        let last_update = json["lastUpdate"]
            .as_object()
            .ok_or("Missing lastUpdate in deployment config")?;

        let deploy_block = last_update["block_number"]
            .as_str()
            .ok_or("Missing block_number in lastUpdate")?
            .parse::<u64>()?;

        let address = registry_coordinator
            .parse::<Address>()
            .map_err(|_| "Failed to parse registry coordinator address")?;

        Ok(AvsDeploymentConfig {
            registry_coordinator_address: address,
            deploy_block,
        })
    }

    pub async fn new(
        http_endpoint: String,
        ws_endpoint: String,
        avs_deployment_path: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Self::read_avs_deployment_config(&avs_deployment_path)?;
        let avs_registry_reader = AvsRegistryChainReader::new(
            config.registry_coordinator_address,
            OPERATOR_STATE_RETRIEVER_ADDRESS,
            http_endpoint.clone(),
        )
        .await?;
        let (operator_info_service, _rx) =
            OperatorInfoServiceInMemory::new(avs_registry_reader.clone(), ws_endpoint)
                .await
                .expect("Failed to create OperatorInfoServiceInMemory");

        Ok(Self {
            http_endpoint,
            registry_coordinator_address: config.registry_coordinator_address,
            registry_coordinator_deploy_block: config.deploy_block,
            operator_info_service: Arc::new(operator_info_service),
        })
    }

    pub async fn get_operator_states(&self) -> Result<Vec<QuorumInfo>, Box<dyn std::error::Error>> {
        // Query current block and backfill operator events
        let provider = get_provider(&self.http_endpoint);
        let current_block_number = provider.get_block_number().await?;
        self.operator_info_service
            .query_past_registered_operator_events_and_fill_db(
                self.registry_coordinator_deploy_block,
                current_block_number,
            )
            .await?;

        // Query operator states
        let operator_state_retriever =
            OperatorStateRetriever::new(OPERATOR_STATE_RETRIEVER_ADDRESS, provider);
        let quorum_numbers: Vec<u8> = vec![0];
        let operators_state = operator_state_retriever
            .getOperatorState_0(
                self.registry_coordinator_address,
                quorum_numbers.into(),
                current_block_number.try_into().unwrap(),
            )
            .call()
            .await?;

        let mut quorum_infos = Vec::new();

        for (quorum_number, operators) in operators_state.iter().enumerate() {
            let mut quorum_operators = Vec::new();
            let mut total_stake = U256::ZERO;

            for op in operators {
                let stake = U256::from(op.stake);
                total_stake += stake;

                let pub_keys = if let Ok(info) = self
                    .operator_info_service
                    .get_operator_info(op.operator)
                    .await
                {
                    info.map(|keys| {
                        CommonwarePublicKeys::from_bls_keys(keys.g1_pub_key, keys.g2_pub_key)
                    })
                } else {
                    None
                };

                let socket = self
                    .operator_info_service
                    .get_operator_socket(op.operator)
                    .await
                    .ok()
                    .flatten();

                quorum_operators.push(OperatorInfo {
                    address: op.operator,
                    stake,
                    pub_keys,
                    socket,
                    quorum_number: quorum_number as u8,
                });
            }

            quorum_infos.push(QuorumInfo {
                quorum_number: quorum_number as u8,
                operator_count: operators.len(),
                total_stake,
                operators: quorum_operators,
            });
        }

        Ok(quorum_infos)
    }
}
