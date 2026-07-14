//! Counter router: verifier-only aggregation engine + task sequencer + on-chain
//! submitter.
//!
//! The router is NOT a signing participant. It runs the commonware aggregation
//! engine with a verifier-only [`Bn254Scheme`] (`me() == None`): the engine
//! validates the nodes' acks on channel 0, assembles BN254 certificates at
//! quorum, journals them, and reports them to the `CertReporter`. Task flow:
//! poll `Counter.number()` → sequencer (assigns aggregation heights, broadcasts
//! `TaskDirective`s on channel 1) → nodes sign → engine certifies → submitter
//! calls `Counter.increment` on-chain.

mod executor;
mod factories;
mod provider;
mod source;

use ark_bn254::Fr;
use clap::{Arg, Command, value_parser};
use commonware_avs_core::bn254::{Bn254, Bn254Scheme, G1PublicKey, PrivateKey, PublicKey};
use commonware_avs_core::consensus::StaticEpochMonitor;
use commonware_avs_eigenlayer::{EigenStakingClient, QuorumInfo};
use commonware_avs_router::automaton::RouterAutomaton;
use commonware_avs_router::reporter::{CertReporter, certified_channel};
use commonware_avs_router::sequencer::{
    DispatchTime, Sequencer, TipReports, ingest_tip_reports, resolution_channel, shared_assignments,
};
use commonware_consensus::aggregation::{Config as AggregationConfig, Engine};
use commonware_consensus::types::{Epoch, EpochDelta, HeightDelta};
use commonware_cryptography::Signer;
use commonware_cryptography::certificate::{ConstantProvider, Scheme as _};
use commonware_p2p::authenticated::lookup::{self, Network};
use commonware_p2p::{Address, AddressableManager};
use commonware_parallel::Sequential;
use commonware_runtime::buffer::paged::CacheRef;
use commonware_runtime::{
    Runner, Spawner, Supervisor,
    tokio::{self},
};
use commonware_utils::ordered::{Map, Set};
use commonware_utils::{NZU16, NZU64, NZUsize, NonZeroDuration};
use counter_common::types::CounterTaskData;
use counter_common::{
    ack_messages_per_second, agg_activity_timeout, agg_window, p2p_message_backlog,
    p2p_quota_period, rebroadcast_interval, round_timeout, storage_directory,
};
use eigen_logging::log_level::LogLevel;
use governor::Quota;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

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

/// P2p channel carrying the aggregation engine's acks (engine-internal).
const ACK_CHANNEL: u64 = 0;

/// P2p channel on which the router broadcasts `TaskDirective`s to the nodes (and
/// receives their rate-limited TipReport replies).
const DIRECTIVE_CHANNEL: u64 = 1;

/// Journal partition for the router's verifier-only engine (a subdirectory of
/// the runtime storage directory).
const JOURNAL_PARTITION: &str = "aggregation-router";

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
    // Initialize runtime. A stable storage directory is REQUIRED: the engine's
    // certificate journal must survive restarts (the runtime default is a
    // random per-process temp dir, which would silently lose replay).
    let storage_dir = storage_directory().join("router");
    let runtime_cfg = tokio::Config::default().with_storage_directory(storage_dir.clone());
    let runner = tokio::Runner::new(runtime_cfg);

    // Parse arguments
    let matches = Command::new("router")
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

    // Configure my identity
    let key_file = matches
        .get_one::<String>("key-file")
        .expect("Please provide key file");
    let port = matches
        .get_one::<String>("port")
        .expect("Please provide port");
    let key = load_key_from_file(key_file);
    let signer = get_signer(&key);
    let port = port.parse::<u16>().expect("Port not well-formed");
    tracing::info!(port, "loaded port");

    // Configure network
    const MAX_MESSAGE_SIZE: u32 = 1024 * 1024; // 1 MB
    let my_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
    let mut p2p_cfg = lookup::Config::local(
        signer.clone(),
        APPLICATION_NAMESPACE,
        my_addr,
        MAX_MESSAGE_SIZE,
    );

    // Behind K8s/NAT source IPs never match registered addresses, so peers are
    // authenticated by the handshake rather than by IP.
    p2p_cfg.bypass_ip_check = true;

    // Start runtime
    runner.start(|context| async move {
        let (mut network, mut oracle) = Network::new(context.child("network"), p2p_cfg);
        let mut recipients: Vec<(PublicKey, Address)>;
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
                                recipients.push((verifier, Address::from(socket_addr)));
                            } else {
                                panic!("No addresses found for socket: {socket}");
                            }
                        }
                        Err(e) => {
                            // If resolution fails, try parsing as direct IP:PORT
                            match SocketAddr::from_str(&socket) {
                                Ok(socket_addr) => {
                                    recipients.push((verifier, Address::from(socket_addr)));
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
            // Authorize ourselves too (nodes dial the router from
            // public_router.json; this entry is never dialed by us).
            let router_verifier = signer.public_key();
            recipients.push((router_verifier, Address::from(my_addr)));
        }
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(std::io::stdout)
            .finish();
        let _ = tracing::subscriber::set_default(subscriber);

        tracing::info!(storage_dir = %storage_dir.display(), "engine journal storage directory");

        // Register the authorized peer set (id 0).
        let authorized = Map::from_iter_dedup(recipients);
        oracle.track(0, authorized);

        // Build the participant set (sorted G2 keys — participant indices derive
        // from this order on every process) and the index-aligned G1 keys.
        // `from_iter_dedup` keeps the first occurrence on a duplicate G2 key —
        // IDENTICAL to the node's construction, so a duplicate key bound to
        // different G1 keys cannot misalign the two sides' participant indices.
        let operators = &quorum_infos[0].operators;
        if operators.is_empty() {
            panic!("Please provide at least one operator");
        }
        let key_map: Map<PublicKey, G1PublicKey> =
            Map::from_iter_dedup(operators.iter().map(|operator| {
                let keys = operator.pub_keys.as_ref().expect("operator has BLS keys");
                tracing::info!(key = ?keys.g2_pub_key, "registered operator");
                (keys.g2_pub_key.clone(), keys.g1_pub_key.clone())
            }));
        let participants: Set<PublicKey> = Set::from_iter_dedup(key_map.iter().cloned());
        let g1_keys: Vec<G1PublicKey> = key_map.iter_pairs().map(|(_, g1)| g1.clone()).collect();

        // Verifier-only scheme: the router validates acks and assembles
        // certificates but never signs (its key is not in the participant set).
        let scheme = Bn254Scheme::verifier(participants, g1_keys);
        tracing::info!(
            participants = scheme.participants().len(),
            "operator set loaded"
        );

        // Register channels (must precede network.start()).
        //
        // The ack channel needs its own, much larger quota: node engines keep
        // rebroadcasting each signed height's ack until it falls activity_timeout
        // below the tip (even after certification), and the p2p limiter silently
        // drops messages beyond the per-peer rate — an undersized quota here
        // starves the router of fresh acks and stalls certification.
        let p2p_backlog = p2p_message_backlog();
        let p2p_quota = Quota::with_period(p2p_quota_period())
            .expect("p2p_quota_period always returns a non-zero duration");
        let ack_rate = ack_messages_per_second();
        let ack_quota = Quota::per_second(ack_rate);
        tracing::info!(
            ack_messages_per_second = ack_rate.get(),
            "engine channel quota"
        );
        let (ack_sender, ack_receiver) = network.register(ACK_CHANNEL, ack_quota, p2p_backlog);
        // The router SENDS directives on channel 1 and receives the nodes'
        // rate-limited TipReport replies (journal-loss recovery) on the same
        // channel.
        let (directive_sender, directive_receiver) =
            network.register(DIRECTIVE_CHANNEL, p2p_quota, p2p_backlog);

        // State shared across sequencer / automaton / submitter.
        let assignments = shared_assignments::<CounterTaskData>();
        let dispatch_time: DispatchTime = Arc::new(Mutex::new(HashMap::new()));
        let (certified_sender, certified_receiver) = certified_channel();
        let (resolution_sender, resolution_receiver) = resolution_channel();

        // Certificate reporter actor (the engine's Reporter).
        let (cert_reporter, reporter_mailbox) = CertReporter::new(
            context.child("cert_reporter"),
            scheme.clone(),
            certified_sender,
        );
        context
            .child("cert_reporter_actor")
            .spawn(move |_| cert_reporter.run());

        // Verifier-only aggregation engine on channel 0.
        let engine = Engine::new(
            context.child("engine"),
            AggregationConfig {
                monitor: StaticEpochMonitor::new(),
                provider: ConstantProvider::<Bn254Scheme, Epoch>::new(scheme.clone()),
                automaton: RouterAutomaton::new(assignments.clone()),
                reporter: reporter_mailbox.clone(),
                blocker: oracle.clone(),
                priority_acks: false,
                rebroadcast_timeout: NonZeroDuration::new_panic(rebroadcast_interval()),
                // Single static epoch: nothing to keep or accept beyond it.
                epoch_bounds: (EpochDelta::new(0), EpochDelta::new(0)),
                window: agg_window(),
                activity_timeout: HeightDelta::new(agg_activity_timeout()),
                journal_partition: JOURNAL_PARTITION.to_string(),
                journal_write_buffer: NZUsize!(4096),
                journal_replay_buffer: NZUsize!(4096),
                journal_heights_per_section: NZU64!(64),
                journal_compression: None,
                journal_page_cache: CacheRef::from_pooler(&context, NZU16!(4096), NZUsize!(128)),
                strategy: Sequential,
            },
        );
        engine.start((ack_sender, ack_receiver));

        // Polling task source: reads Counter.number() between rounds and hands
        // the current round to the sequencer.
        let task_source = crate::factories::create_task_source()
            .await
            .expect("Failed to create task source");

        // On-chain submitter: consumes verified certificates, resolves heights.
        let submitter = crate::factories::create_submitter(
            scheme.clone(),
            assignments.clone(),
            certified_receiver,
            resolution_sender,
            APPLICATION_NAMESPACE.to_vec(),
        )
        .await
        .expect("Failed to create submitter");
        context.child("submitter").spawn(move |_| submitter.run());

        // Node tip reports (channel 1, node → router): if this router lost its
        // journal and assigns heights the nodes are already past, their reports
        // fast-forward the sequencer instead of wedging on a dead height.
        let tip_reports = TipReports::new(scheme.participants().len());
        {
            let participant_keys: HashSet<PublicKey> =
                scheme.participants().iter().cloned().collect();
            let tip_reports = tip_reports.clone();
            context.child("tip_reports").spawn(move |_| async move {
                ingest_tip_reports::<CounterTaskData, _, _>(
                    directive_receiver,
                    participant_keys,
                    tip_reports,
                )
                .await;
            });
        }

        // Sequencer: pulls tasks from the source, assigns heights, broadcasts
        // directives to the operator set (explicit keys — see Sequencer::broadcast).
        let directive_recipients: Vec<PublicKey> = scheme.participants().iter().cloned().collect();
        let sequencer = Sequencer::new(
            task_source,
            dispatch_time,
            assignments,
            reporter_mailbox,
            resolution_receiver,
            directive_sender,
            directive_recipients,
            tip_reports,
            round_timeout(),
            rebroadcast_interval(),
        );
        context.child("sequencer").spawn(move |_| sequencer.run());

        // Run the network; blocks the root future (and thus the process) until
        // shutdown. All tasks spawned above are children of this context and
        // abort when it returns.
        let _ = network.start().await;
    });
}
