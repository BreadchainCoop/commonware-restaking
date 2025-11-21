use ark_bn254::Fr;
use bn254::{Bn254, PrivateKey};
use clap::{Arg, Command};
use commonware_avs_node::contributor::{AggregationInput, Contribute};
use commonware_eigenlayer::network_configuration::{EigenStakingClient, QuorumInfo};
use commonware_p2p::authenticated::lookup::{self, Network};
use commonware_runtime::{
    Metrics, Runner, Spawner,
    tokio::{self},
};
use commonware_usecase_counter::CounterValidator;
use commonware_usecase_counter::types::CounterTaskData;
use commonware_utils::NZU32;
use eigen_logging::log_level::LogLevel;
use governor::Quota;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct KeyConfig {
    privateKey: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct OrchestratorConfig {
    g2_x1: String,
    g2_x2: String,
    g2_y1: String,
    g2_y2: String,
    #[serde(default = "default_address")]
    address: String,
    port: String,
}

fn default_address() -> String {
    "localhost".to_string()
}

fn get_signer(key: &str) -> Bn254 {
    let fr = Fr::from_str(key).expect("Invalid decimal string for private key");
    let key = PrivateKey::from(fr);
    Bn254::new(key).expect("Failed to create signer")
}

fn load_key_from_file(path: &str) -> String {
    let contents = fs::read_to_string(path).expect("Could not read key file");
    let config: KeyConfig = serde_json::from_str(&contents).expect("Could not parse key file");
    config.privateKey
}

fn load_orchestrator_config(path: &str) -> OrchestratorConfig {
    let contents = fs::read_to_string(path).expect("Could not read key file");
    let config: OrchestratorConfig =
        serde_json::from_str(&contents).expect("Could not parse key file");
    config
}

// Unique namespace to avoid message replay attacks.
const APPLICATION_NAMESPACE: &[u8] = b"_COMMONWARE_AGGREGATION_";

fn configure_identity(matches: &clap::ArgMatches) -> (Bn254, u16) {
    let key_file = matches
        .get_one::<String>("key-file")
        .expect("Please provide key file");
    let port = matches
        .get_one::<String>("port")
        .expect("Please provide port");
    let key = load_key_from_file(key_file);
    let me = format!("{key}@{port}");
    let parts = me.split('@').collect::<Vec<&str>>();
    if parts.len() != 2 {
        panic!("Identity not well-formed");
    }
    let key = parts[0];
    let signer = get_signer(key);

    let port = parts[1].parse::<u16>().expect("Port not well-formed");

    (signer, port)
}

fn configure_orchestrator(matches: &clap::ArgMatches) -> OrchestratorConfig {
    let orchestrator_file = matches
        .get_one::<String>("orchestrator")
        .expect("No orchestrator addr");
    load_orchestrator_config(orchestrator_file)
}

async fn get_operator_states() -> Result<Vec<QuorumInfo>, Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let http_rpc = env::var("HTTP_RPC").expect("HTTP_RPC must be set");
    let ws_rpc = env::var("WS_RPC").expect("WS_RPC must be set");
    let avs_deployment_path =
        env::var("AVS_DEPLOYMENT_PATH").expect("AVS_DEPLOYMENT_PATH must be set");

    let client = EigenStakingClient::new(http_rpc, ws_rpc, avs_deployment_path).await?;

    client.get_operator_states().await
}

pub fn main() {
    // Initialize runtime
    let runtime_cfg = tokio::Config::default();
    let runner = tokio::Runner::new(runtime_cfg.clone());

    // Parse arguments
    let matches = Command::new("commonware-aggregation")
        .about("generate and verify BN254 Multi-Signatures")
        .arg(
            Arg::new("key-file")
                .long("key-file")
                .required(true)
                .help("Path to the YAML file containing the private key"),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .required(true)
                .help("Port to run the service on"),
        )
        .arg(
            Arg::new("orchestrator")
                .long("orchestrator")
                .required(false)
                .help("Path to orchestrator key file"),
        )
        .arg(
            Arg::new("aggregation")
                .short('a')
                .required(false)
                .num_args(0)
                .help("turn on aggregation"),
        )
        .get_matches();

    // Configure my identity
    let (signer, port) = configure_identity(&matches);
    let orchestrator_config = configure_orchestrator(&matches);
    let aggregation: bool = matches.contains_id("aggregation");

    // Start runtime
    runner.start(|context: tokio::Context| async move {
        let mut recipients: Vec<(bn254::PublicKey, SocketAddr)> = Vec::new();
        // Scoped to avoid configuring two loggers
        let orchestrator_pub_key;
        {
            eigen_logging::init_logger(LogLevel::Debug);
            let quorum_infos = get_operator_states()
                .await
                .expect("Failed to get operator states");
            // Configure allowed peers
            let participants = quorum_infos[0].operators.clone(); //TODO: Fix hardcoded quorum_number
            if participants.is_empty() {
                panic!("Please provide at least one participant");
            }
            for participant in &participants {
                let verifier = participant.pub_keys.as_ref().unwrap().g2_pub_key.clone();
                if let Some(socket) = &participant.socket {
                    // Try to resolve hostname:port to socket addresses
                    match socket.to_socket_addrs() {
                        Ok(mut addrs) => {
                            if let Some(socket_addr) = addrs.next() {
                                recipients.push((verifier, socket_addr));
                            } else {
                                panic!("No addresses found for socket: {socket}");
                            }
                        }
                        Err(_) => {
                            // If resolution fails, try parsing as direct IP:PORT
                            match SocketAddr::from_str(socket) {
                                Ok(socket_addr) => {
                                    recipients.push((verifier, socket_addr));
                                }
                                Err(_) => {
                                    panic!("Contributor address not well-formed: {socket}");
                                }
                            }
                        }
                    }
                }
            }
            orchestrator_pub_key = bn254::PublicKey::create_from_g2_coordinates(
                &orchestrator_config.g2_x1,
                &orchestrator_config.g2_x2,
                &orchestrator_config.g2_y1,
                &orchestrator_config.g2_y2,
            )
            .unwrap();

            let orchestrator_addr = orchestrator_config
                .address
                .parse::<IpAddr>()
                .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

            let local_addr = SocketAddr::new(
                orchestrator_addr,
                orchestrator_config
                    .port
                    .parse::<u16>()
                    .expect("Port not well-formed"),
            );

            recipients.push((orchestrator_pub_key.clone(), local_addr));
        }

        // Configure network
        const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1 MB
        let my_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let my_local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        let p2p_cfg = lookup::Config::aggressive(
            signer.clone(),
            APPLICATION_NAMESPACE,
            my_addr,
            my_local_addr,
            MAX_MESSAGE_SIZE,
        );
        let (mut network, mut oracle) = Network::new(context.with_label("network"), p2p_cfg);

        // Provide authorized peers
        oracle.register(0, recipients).await;

        // Parse contributors from operator states
        let mut contributors = Vec::new();
        let mut contributors_map = HashMap::new();
        let quorum_infos = get_operator_states()
            .await
            .expect("Failed to get operator states");
        let operators = &quorum_infos[0].operators;
        if operators.is_empty() {
            panic!("Please provide at least one contributor");
        }
        for operator in operators {
            let verifier = operator.pub_keys.as_ref().unwrap().g2_pub_key.clone();
            let verifier_g1 = operator.pub_keys.as_ref().unwrap().g1_pub_key.clone();
            contributors.push(verifier.clone());
            contributors_map.insert(verifier, verifier_g1);
        }

        // Check if I am the orchestrator
        const DEFAULT_MESSAGE_BACKLOG: usize = 256;

        // Create contributor
        let (sender, receiver) =
            network.register(0, Quota::per_second(NZU32!(1)), DEFAULT_MESSAGE_BACKLOG);

        let mut aggregation_input: Option<AggregationInput> = None;
        if aggregation {
            let signatures_needed = contributors.len();
            aggregation_input = Some(AggregationInput::new(signatures_needed, contributors_map));
        }

        let counter_validator = CounterValidator::new()
            .await
            .expect("Failed to construct CounterValidator");

        let contributor: commonware_avs_node::contributor::Contributor<CounterTaskData> =
            commonware_avs_node::contributor::Contributor::new(
                orchestrator_pub_key,
                signer,
                contributors,
                aggregation_input,
            )
            .with_validator(Arc::new(counter_validator));
        context.spawn(|_| async move { contributor.run(sender, receiver).await });

        let _ = network.start().await;
    });
}
