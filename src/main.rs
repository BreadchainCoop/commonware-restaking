mod handlers;
mod ingress;
mod validator;
mod wire;
mod solana_helpers;
use clap::{Arg, Command};
use commonware_avs_router::solana_helpers::get_operator_states;
use commonware_p2p::authenticated::lookup::{self, Network};
use commonware_runtime::{
    Metrics, Runner, Spawner,
    tokio::{self},
};
use commonware_utils::NZU32;
use eigen_logging::log_level::LogLevel;
use governor::Quota;
use jito_bls_ncn_core::{bls::solana_bls_interface::{SolanaBN254G2}};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use crate::solana_helpers::get_client_and_test_bls_ncn;

// Unique namespace to avoid message replay attacks.
const APPLICATION_NAMESPACE: &[u8] = b"_COMMONWARE_AGGREGATION_";

fn main() {
    dotenv::dotenv().ok();

    // Initialize runtime
    let runtime_cfg = tokio::Config::default();
    let runner = tokio::Runner::new(runtime_cfg.clone());

    // Parse arguments
    let matches = Command::new("orchestrator")
        .about("generate and verify BN254 Multi-Signatures")
        .arg(
            Arg::new("port")
                .long("port")
                .required(true)
                .help("Port to run the service on"),
        )
        .get_matches();

    let port_str = matches
        .get_one::<String>("port")
        .expect("Please provide port");
    let port = port_str.parse::<u16>().expect("Invalid port");
    let (client, test_bls_ncn) = get_client_and_test_bls_ncn().expect("Could not get test_ncn");
    let signer = test_bls_ncn.bls_orchestrator.bls_keypair;

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
        let mut recipients: Vec<(SolanaBN254G2, SocketAddr)>;
        let quorum_infos;
        {
            eigen_logging::init_logger(LogLevel::Debug);
            // Get operator states and configure allowed peers
            quorum_infos = get_operator_states(&client, &test_bls_ncn)
                .await
                .expect("Failed to get operator states");
            recipients = Vec::new();
            let participants = quorum_infos.clone(); //TODO: Fix hardcoded quorum_number
            if participants.is_empty() {
                panic!("Please provide at least one participant");
            }
            for participant in participants {
                let verifier = participant.bls_operator.g2;
                tracing::info!(key = ?verifier, "registered authorized key",);
                if let Some(socket) = participant.socket_address {
                    recipients.push((verifier, socket));
                }
            }
            let orchestrator_verifier = signer.public_key.g2;
            recipients.push((orchestrator_verifier, my_local_addr));
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
        let operators = quorum_infos.clone();
        if operators.is_empty() {
            panic!("Please provide at least one contributor");
        }
        for operator in operators {
            let verifier = operator.bls_operator.g2;
            tracing::info!(key = ?verifier, "registered contributor",);
            contributors.push(verifier.clone());
            contributors_map.insert(verifier, operator.bls_operator);
        }

        // Infer threshold
        let threshold = 3; //hardcoded for now

        // Run as the orchestrator
        const DEFAULT_MESSAGE_BACKLOG: usize = 256;
        const AGGREGATION_FREQUENCY: Duration = Duration::from_secs(30);

        let (sender, receiver) =
            network.register(0, Quota::per_second(NZU32!(1)), DEFAULT_MESSAGE_BACKLOG);
        let orchestrator = handlers::Orchestrator::new(
            context.clone(),
            signer,
            AGGREGATION_FREQUENCY,
            contributors,
            contributors_map,
            threshold as usize,
        )
        .await;

        context.spawn(|_| async move { orchestrator.run(sender, receiver).await });

        let _ = network.start().await;
    });
}
