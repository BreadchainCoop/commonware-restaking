//! Counter node: aggregation participant for the counter AVS.
//!
//! The node runs the commonware-consensus aggregation engine with the BN254
//! multisig scheme: for every height the router announces a task on p2p channel 1,
//! the node validates it against the deployed `Counter` contract, signs the
//! expected task digest (or the skip digest when the router abandons the height),
//! and gossips acks on channel 0 until the height certifies.

use ark_bn254::Fr;
use clap::{Arg, Command};
use commonware_avs_core::bn254::{Bn254, Bn254Scheme, G1PublicKey, PrivateKey, PublicKey};
use commonware_avs_core::consensus::StaticEpochMonitor;
use commonware_avs_core::validator::ValidatorTrait;
use commonware_avs_eigenlayer::{EigenStakingClient, QuorumInfo};
use commonware_avs_node::automaton::NodeAutomaton;
use commonware_avs_node::reporter::NodeReporter;
use commonware_avs_node::task_book::{self, TaskBook};
use commonware_consensus::aggregation::{Config as AggregationConfig, Engine};
use commonware_consensus::types::{Epoch, EpochDelta, HeightDelta};
use commonware_cryptography::Signer as _;
use commonware_cryptography::certificate::ConstantProvider;
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
    CounterValidator, ack_messages_per_second, agg_activity_timeout, agg_window,
    p2p_message_backlog, p2p_quota_period, rebroadcast_interval, round_timeout, storage_directory,
};
use eigen_logging::log_level::LogLevel;
use governor::Quota;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct KeyConfig {
    privateKey: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct RouterConfig {
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

fn load_router_config(path: &str) -> RouterConfig {
    let contents = fs::read_to_string(path).expect("Could not read key file");
    let config: RouterConfig = serde_json::from_str(&contents).expect("Could not parse key file");
    config
}

// Unique namespace to avoid message replay attacks.
const APPLICATION_NAMESPACE: &[u8] = b"_COMMONWARE_AGGREGATION_";

/// P2P channel carrying the aggregation engine's ack gossip.
const ENGINE_CHANNEL: u64 = 0;

/// P2P channel carrying the router's `TaskDirective` broadcasts (nodes receive;
/// the sender half is only used for rate-limited TipReport replies to stale
/// directives).
const TASK_DIRECTIVE_CHANNEL: u64 = 1;

fn configure_identity(matches: &clap::ArgMatches) -> (Bn254, u16) {
    let key_file = matches
        .get_one::<String>("key-file")
        .expect("Please provide key file");
    let port = matches
        .get_one::<String>("port")
        .expect("Please provide port");
    let key = load_key_from_file(key_file);
    let signer = get_signer(&key);
    let port = port.parse::<u16>().expect("Port not well-formed");

    (signer, port)
}

fn configure_router(matches: &clap::ArgMatches) -> RouterConfig {
    let router_file = matches
        .get_one::<String>("router")
        .expect("No router connection file provided");
    load_router_config(router_file)
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
    // Initialize runtime. The storage directory anchors the engine's journal:
    // without a stable path the runtime defaults to a random per-process temp
    // dir and journal replay after restart silently loses history.
    let storage_dir = storage_directory();
    let runtime_cfg = tokio::Config::default().with_storage_directory(storage_dir.clone());
    let runner = tokio::Runner::new(runtime_cfg);

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
            Arg::new("router")
                .long("router")
                .required(false)
                .help("Path to the router's public connection file"),
        )
        .get_matches();

    // Configure my identity
    let (signer, port) = configure_identity(&matches);
    let router_config = configure_router(&matches);

    // Start runtime
    runner.start(|context: tokio::Context| async move {
        let mut recipients: Vec<(PublicKey, Address)> = Vec::new();
        // Scoped to avoid configuring two loggers
        let router_pub_key;
        let quorum_infos;
        {
            eigen_logging::init_logger(LogLevel::Debug);
            quorum_infos = get_operator_states()
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
                                recipients.push((verifier, Address::from(socket_addr)));
                            } else {
                                panic!("No addresses found for socket: {socket}");
                            }
                        }
                        Err(_) => {
                            // If resolution fails, try parsing as direct IP:PORT
                            match SocketAddr::from_str(socket) {
                                Ok(socket_addr) => {
                                    recipients.push((verifier, Address::from(socket_addr)));
                                }
                                Err(_) => {
                                    panic!("Operator address not well-formed: {socket}");
                                }
                            }
                        }
                    }
                }
            }
            router_pub_key = PublicKey::create_from_g2_coordinates(
                &router_config.g2_x1,
                &router_config.g2_x2,
                &router_config.g2_y1,
                &router_config.g2_y2,
            )
            .unwrap();

            let router_addr = router_config
                .address
                .parse::<IpAddr>()
                .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

            let local_addr = SocketAddr::new(
                router_addr,
                router_config
                    .port
                    .parse::<u16>()
                    .expect("Port not well-formed"),
            );

            recipients.push((router_pub_key.clone(), Address::from(local_addr)));
        }

        tracing::info!(storage_dir = %storage_dir.display(), "engine journal storage directory");

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

        let (mut network, mut oracle) = Network::new(context.child("network"), p2p_cfg);

        // Register the authorized peer set (id 0).
        let authorized = Map::from_iter_dedup(recipients);
        oracle.track(0, authorized);

        // Build the ordered participant set (G2 keys, sorted by compressed bytes)
        // with the index-aligned G1 key vector. Participant indices are positions
        // in this sorted order — the router builds the identical set from the same
        // operator state, so attestations attribute to the same indices everywhere.
        // `from_iter_dedup` keeps the first occurrence on a duplicate G2 key, so
        // both sides resolve a duplicate to the same G1 key.
        let operators = &quorum_infos[0].operators;
        if operators.is_empty() {
            panic!("Please provide at least one operator");
        }
        let key_map: Map<PublicKey, G1PublicKey> =
            Map::from_iter_dedup(operators.iter().map(|operator| {
                let keys = operator.pub_keys.as_ref().expect("operator has BLS keys");
                (keys.g2_pub_key.clone(), keys.g1_pub_key.clone())
            }));
        let participants: Set<PublicKey> = Set::from_iter_dedup(key_map.iter().cloned());
        let g1_keys: Vec<G1PublicKey> = key_map.iter_pairs().map(|(_, g1)| g1.clone()).collect();
        for key in participants.iter() {
            tracing::info!(key = ?key, "registered participant");
        }

        // Signing scheme over the participant set; our own participant index is
        // derived from our G2 key's position in the sorted set.
        let scheme = Bn254Scheme::signer(participants, g1_keys, signer.private_key())
            .unwrap_or_else(|| {
                panic!(
                    "own BN254 G2 key {:?} is not in the operator set; register the \
                     operator on-chain before starting the node",
                    signer.public_key()
                )
            });

        // Create network channels (must all be registered BEFORE network.start()):
        // channel 0 carries the engine's ack gossip, channel 1 the router's task
        // directives.
        //
        // The engine channel needs its own, much larger quota: the engine keeps
        // rebroadcasting each signed height's ack until it falls activity_timeout
        // below the tip (even after certification), and the p2p send-side limiter
        // SILENTLY DROPS messages to rate-limited peers — an undersized quota would
        // starve fresh acks and stall certification.
        let p2p_backlog = p2p_message_backlog();
        let p2p_quota = Quota::with_period(p2p_quota_period())
            .expect("p2p_quota_period always returns a non-zero duration");
        let ack_rate = ack_messages_per_second();
        let ack_quota = Quota::per_second(ack_rate);
        tracing::info!(
            ack_messages_per_second = ack_rate.get(),
            "engine channel quota"
        );
        let (engine_sender, engine_receiver) =
            network.register(ENGINE_CHANNEL, ack_quota, p2p_backlog);
        let (directive_sender, directive_receiver) =
            network.register(TASK_DIRECTIVE_CHANNEL, p2p_quota, p2p_backlog);

        // Validator: recomputes the expected digest for each announced task from
        // the deployed Counter contract's state.
        let counter_validator = CounterValidator::new()
            .await
            .expect("Failed to construct CounterValidator");
        let validator: Arc<dyn ValidatorTrait<CounterTaskData>> = Arc::new(counter_validator);

        // TaskBook actor: owns the router's per-height directives and parks the
        // automaton's subscriptions until the skip rules resolve them.
        let (task_book, task_book_mailbox) = TaskBook::new(context.child("task_book"));
        context
            .child("task_book_actor")
            .spawn(move |_| task_book.run());

        // Shared engine-tip mirror: written by the reporter, read by the directive
        // ingest loop to answer stale directives with a TipReport.
        let engine_tip = Arc::new(AtomicU64::new(0));

        // Feed the TaskBook from channel 1 (router directives only; other peers
        // are authorized on the channel but must not assign heights).
        {
            let task_book_mailbox = task_book_mailbox.clone();
            let router_key = router_pub_key.clone();
            let engine_tip = Arc::clone(&engine_tip);
            let min_report_interval = rebroadcast_interval();
            context.child("directives").spawn(move |_| async move {
                task_book::ingest(
                    directive_receiver,
                    directive_sender,
                    router_key,
                    task_book_mailbox,
                    engine_tip,
                    agg_window().get(),
                    min_report_interval,
                )
                .await;
            });
        }

        // Reporter actor: certificate/tip accounting + TaskBook pruning.
        let (node_reporter, reporter_mailbox) = NodeReporter::<_, Bn254Scheme>::new(
            context.child("reporter"),
            task_book_mailbox.clone(),
            Arc::clone(&engine_tip),
            APPLICATION_NAMESPACE.to_vec(),
        );
        context
            .child("reporter_actor")
            .spawn(move |_| node_reporter.run());

        // Automaton: resolves each proposed height to the expected task digest
        // (validated against the Counter contract) or the skip digest, per the
        // TaskBook.
        let automaton = NodeAutomaton::new(
            context.child("automaton"),
            task_book_mailbox,
            validator,
            APPLICATION_NAMESPACE.to_vec(),
            // Retry transient validation errors up to the router's round timeout:
            // past that the router is broadcasting Skip{h} anyway.
            round_timeout(),
        );

        // Static single-epoch supervision: ConstantProvider serves the same scheme
        // for every epoch and the monitor never fires. Keep a clone of the monitor
        // in the root future — it must outlive the engine or the engine exits.
        let provider = ConstantProvider::<Bn254Scheme, Epoch>::new(scheme);
        let monitor = StaticEpochMonitor::new();
        let monitor_guard = monitor.clone();

        // Aggregation engine (journal knobs follow the upstream test defaults).
        tracing::info!(
            window = agg_window().get(),
            activity_timeout = agg_activity_timeout(),
            rebroadcast_secs = rebroadcast_interval().as_secs_f64(),
            round_timeout_secs = round_timeout().as_secs_f64(),
            "aggregation engine tuning"
        );
        let engine = Engine::new(
            context.child("engine"),
            AggregationConfig {
                monitor,
                provider,
                automaton,
                reporter: reporter_mailbox,
                // The oracle disconnects peers on blockable offenses (bad ack
                // signatures / signer mismatches).
                blocker: oracle.clone(),
                priority_acks: false,
                // Re-send our own ack until quorum; reuse the router's directive
                // rebroadcast cadence.
                rebroadcast_timeout: NonZeroDuration::new_panic(rebroadcast_interval()),
                // Single static epoch: only epoch 0 acks are valid.
                epoch_bounds: (EpochDelta::new(0), EpochDelta::new(0)),
                window: agg_window(),
                activity_timeout: HeightDelta::new(agg_activity_timeout()),
                // Per-identity partition: two nodes sharing a storage directory
                // (e.g. the local $TMPDIR fallback) must not share a journal.
                journal_partition: format!("aggregation-node-{}", signer.public_key()),
                journal_write_buffer: NZUsize!(4096),
                journal_replay_buffer: NZUsize!(4096),
                journal_heights_per_section: NZU64!(6),
                journal_compression: Some(3),
                journal_page_cache: CacheRef::from_pooler(&context, NZU16!(1024), NZUsize!(10)),
                strategy: Sequential,
            },
        );
        engine.start((engine_sender, engine_receiver));

        // Start network; blocks the root future (and thus keeps every spawned
        // child alive) until the network shuts down.
        let _ = network.start().await;
        drop(monitor_guard);
    });
}
