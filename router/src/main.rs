mod creator;
mod executor;
mod ingress;
mod orchestrator;
mod usecases;
use crate::orchestrator::interface::OrchestratorTrait;
use ark_bn254::Fr;
use bn254::Bn254;
use bn254::PrivateKey;
use clap::{Arg, Command, value_parser};
use commonware_cryptography::Signer;
use commonware_eigenlayer::network_configuration::{EigenStakingClient, QuorumInfo};
use commonware_p2p::authenticated::lookup::{self, Network};
use commonware_runtime::{
    Metrics, Runner, Spawner,
    tokio::{self},
};
use commonware_utils::NZU32;
use eigen_logging::log_level::LogLevel;
use governor::Quota;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct KeyConfig {
    privateKey: String,
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

// Unique namespace to avoid message replay attacks.
const APPLICATION_NAMESPACE: &[u8] = b"_COMMONWARE_AGGREGATION_";

async fn get_operator_states() -> Result<Vec<QuorumInfo>, Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let http_rpc = env::var("HTTP_RPC").expect("HTTP_RPC must be set");
    let ws_rpc = env::var("WS_RPC").expect("WS_RPC must be set");
    let avs_deployment_path =
        env::var("AVS_DEPLOYMENT_PATH").expect("AVS_DEPLOYMENT_PATH must be set");
    println!("pre init");
    let client = EigenStakingClient::new(http_rpc, ws_rpc, avs_deployment_path).await?;
    println!("init passed");
    client.get_operator_states().await
}

fn main() {
    // Initialize runtime
    let runtime_cfg = tokio::Config::default();
    let runner = tokio::Runner::new(runtime_cfg.clone());

    // Parse arguments
    let matches = Command::new("orchestrator")
        .about("generate and verify BN254 Multi-Signatures")
        .arg(
            Arg::new("bootstrappers")
                .long("bootstrappers")
                .required(false)
                .value_delimiter(',')
                .value_parser(value_parser!(String)),
        )
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
        .get_matches();

    // // Create logger
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .init();

    // Configure my identity
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
    tracing::info!(port, "loaded port");

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

    // Start runtime
    runner.start(|context| async move {
        let (mut network, mut oracle) = Network::new(context.with_label("network"), p2p_cfg);
        let mut recipients: Vec<(bn254::PublicKey, SocketAddr)>;
        let quorum_infos;
        {
            eigen_logging::init_logger(LogLevel::Debug);
            // Get operator states and configure allowed peers
            quorum_infos = get_operator_states()
                .await
                .expect("Failed to get operator states");
            recipients = Vec::new();
            let participants = quorum_infos[0].operators.clone(); //TODO: Fix hardcoded quorum_number
            if participants.is_empty() {
                panic!("Please provide at least one participant");
            }
            for participant in participants {
                let verifier = participant.pub_keys.unwrap().g2_pub_key;
                if let Some(socket) = participant.socket {

                    // Try to resolve hostname:port to socket addresses
                    use std::net::ToSocketAddrs;
                    match socket.to_socket_addrs() {
                        Ok(mut addrs) => {
                            if let Some(socket_addr) = addrs.next() {
                                recipients.push((verifier, socket_addr));
                            } else {
                                panic!("No addresses found for socket: {socket}");
                            }
                        }
                        Err(e) => {
                            // If resolution fails, try parsing as direct IP:PORT
                            match SocketAddr::from_str(&socket) {
                                Ok(socket_addr) => {
                                    recipients.push((verifier, socket_addr));
                                }
                                Err(parse_err) => {
                                    tracing::error!("Failed to resolve '{}': {:?}, and failed to parse as IP: {:?}",
                                                  socket, e, parse_err);
                                    panic!("Bootstrapper address not well-formed: {socket}");
                                }
                            }
                        }
                    }
                }
            }
            let orchestrator_verifier = signer.public_key();
            recipients.push((orchestrator_verifier, my_addr));
        }
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(std::io::stdout)
            .finish();
        let _ = tracing::subscriber::set_default(subscriber);

        // Provide authorized peers
        oracle.register(0, recipients).await;

        // Parse contributors from operator states
        let mut contributors = Vec::new();
        let mut contributors_map = HashMap::new();
        let operators = &quorum_infos[0].operators;
        if operators.is_empty() {
            panic!("Please provide at least one contributor");
        }
        for operator in operators {
            let verifier = operator.pub_keys.as_ref().unwrap().g2_pub_key.clone();
            let verifier_g1 = operator.pub_keys.as_ref().unwrap().g1_pub_key.clone();
            tracing::info!(key = ?verifier, "registered contributor",);
            contributors.push(verifier.clone());
            contributors_map.insert(verifier, verifier_g1);
        }

        // Infer threshold
        let threshold = 3; //hardcoded for now

        // Run as the orchestrator using the builder pattern
        const DEFAULT_MESSAGE_BACKLOG: usize = 256;

        let (sender, receiver) =
            network.register(0, Quota::per_second(NZU32!(1)), DEFAULT_MESSAGE_BACKLOG);

        // Use the builder pattern to create the orchestrator
        let builder = crate::orchestrator::OrchestratorBuilder::new(context.clone(), signer)
            .with_contributors(contributors)
            .with_g1_map(contributors_map)
            .with_threshold(threshold)
            .load_from_env(); // Read configuration from environment variables

        let orchestrator = crate::usecases::counter::CounterOrchestratorBuilder::build(builder)
            .await
            .expect("Failed to build orchestrator");

        context.spawn(|_| async move { orchestrator.run(sender, receiver).await });

        let _ = network.start().await;
    });
}
