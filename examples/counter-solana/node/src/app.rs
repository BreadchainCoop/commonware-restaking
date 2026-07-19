//! Solana counter node: aggregation participant for the Jito NCN demo — the
//! Solana peer of `counter-node`.
//!
//! The node runs the commonware-consensus aggregation engine with the Jito
//! BN254 multisig scheme: for every height the router announces a task on p2p
//! channel 1, the node recomputes the expected digest (binding the NCN from
//! its OWN deployment config), signs it in the NCN program's signature domain
//! (or the skip digest when the router abandons the height), and gossips acks
//! on channel 0 until the height certifies.
//!
//! Chain interaction: operator discovery + stake + sockets from
//! `NCNOperatorAccount`/`Snapshot` PDAs at `confirmed` (a live RPC endpoint is
//! REQUIRED — the example takes config, it does not mock the chain).

use clap::{Arg, Command};
use commonware_avs_core::validator::ValidatorTrait;
use commonware_avs_jito::bn254::PublicKey;
use commonware_avs_jito::scheme::JitoBn254Scheme;
use commonware_avs_jito::{JitoQuorum, JitoStakingClient, NcnDeployment};
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
use commonware_utils::ordered::Map;
use commonware_utils::{NZU16, NZU64, NZUsize, NonZeroDuration};
use counter_solana_common::{
    APPLICATION_NAMESPACE, RoundTaskData, RoundValidator, RouterConnection,
    ack_messages_per_second, agg_activity_timeout, agg_window, load_bn254_key, p2p_message_backlog,
    p2p_quota_period, rebroadcast_interval, round_timeout, storage_directory,
};
use governor::Quota;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

/// P2P channel carrying the aggregation engine's ack gossip.
const ENGINE_CHANNEL: u64 = 0;

/// P2P channel carrying the router's `TaskDirective` broadcasts.
const TASK_DIRECTIVE_CHANNEL: u64 = 1;

/// Loads the quorum view from chain (reads at `confirmed`).
async fn load_quorum(deployment: &NcnDeployment) -> JitoQuorum {
    let client = JitoStakingClient::new(deployment.clone()).expect("staking client construction");
    client
        .get_quorum()
        .await
        .expect("failed to load quorum from chain")
}

pub fn main() {
    // A stable storage directory is REQUIRED: the engine's journal must
    // survive restarts (the runtime default is a random per-process temp dir).
    let storage_dir = storage_directory();
    let runtime_cfg = tokio::Config::default().with_storage_directory(storage_dir.clone());
    let runner = tokio::Runner::new(runtime_cfg);

    let matches = Command::new("counter-solana-node")
        .about("aggregation node for the Jito NCN counter demo")
        .arg(
            Arg::new("key-file")
                .long("key-file")
                .required(true)
                .help("Path to the JSON file with the BN254 private key"),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .required(true)
                .help("Port to run the p2p listener on"),
        )
        .arg(
            Arg::new("router")
                .long("router")
                .required(true)
                .help("Path to the router's public connection file"),
        )
        .get_matches();

    let key_file = matches
        .get_one::<String>("key-file")
        .expect("key file is required");
    let port: u16 = matches
        .get_one::<String>("port")
        .expect("port is required")
        .parse()
        .expect("port not well-formed");
    let router_file = matches
        .get_one::<String>("router")
        .expect("router file is required");

    let signer = load_bn254_key(key_file).expect("failed to load BN254 key");
    let router_connection = RouterConnection::load(router_file).expect("router connection file");

    runner.start(|context: tokio::Context| async move {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(std::io::stdout)
            .finish();
        let _ = tracing::subscriber::set_default(subscriber);
        dotenv::dotenv().ok();

        let deployment = NcnDeployment::load().expect("NCN_DEPLOYMENT_PATH config");
        let ncn = deployment.ncn().expect("ncn pubkey");
        let quorum = load_quorum(&deployment).await;

        // Startup quorum reconciliation (INTERFACES.md §5): refuse to start if
        // the lightest engine quorum cannot clear the on-chain threshold.
        quorum
            .reconcile_engine_quorum()
            .expect("refusing to start: engine quorum cannot clear the stake threshold");

        // Authorized peers: every operator with a registered socket, plus the
        // router (whose key/address come from its connection file).
        let mut recipients: Vec<(PublicKey, Address)> = Vec::new();
        for operator in &quorum.operators {
            let Some(socket) = operator.socket else {
                tracing::warn!(operator = %operator.operator, "operator has no registered socket");
                continue;
            };
            recipients.push((operator.g2_pub_key.clone(), Address::from(socket)));
        }
        let router_key = router_connection.public_key().expect("router g2 key");
        let router_addr = format!("{}:{}", router_connection.address, router_connection.port)
            .to_socket_addrs()
            .ok()
            .and_then(|mut addrs| addrs.next())
            .expect("router address resolves");
        recipients.push((router_key.clone(), Address::from(router_addr)));

        tracing::info!(storage_dir = %storage_dir.display(), "engine journal storage directory");

        // Configure the p2p network; the BN254 identity key doubles as the
        // handshake signer (namespace-separated from certificate signing).
        const MAX_MESSAGE_SIZE: u32 = 1024 * 1024; // 1 MB
        let my_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let mut p2p_cfg = lookup::Config::local(
            signer.clone(),
            APPLICATION_NAMESPACE,
            my_addr,
            MAX_MESSAGE_SIZE,
        );
        // Behind K8s/NAT source IPs never match registered addresses.
        p2p_cfg.bypass_ip_check = true;

        let (mut network, mut oracle) = Network::new(context.child("network"), p2p_cfg);
        oracle.track(0, Map::from_iter_dedup(recipients));

        // Signing scheme over the participant set discovered on-chain; our own
        // participant index derives from our G2 key's sorted position.
        let participants = quorum.participants();
        let g1_keys = quorum.g1_keys();
        let scheme = JitoBn254Scheme::signer(participants, g1_keys, signer.private_key())
            .unwrap_or_else(|| {
                panic!(
                    "own BN254 G2 key {:?} is not in the operator set; register the operator \
                     on-chain before starting the node",
                    signer.public_key()
                )
            });

        // Register channels (must precede network.start()); the ack channel
        // gets its own larger quota (see counter-common docs).
        let p2p_backlog = p2p_message_backlog();
        let p2p_quota = Quota::with_period(p2p_quota_period())
            .expect("p2p_quota_period always returns a non-zero duration");
        let ack_quota = Quota::per_second(ack_messages_per_second());
        let (engine_sender, engine_receiver) =
            network.register(ENGINE_CHANNEL, ack_quota, p2p_backlog);
        let (directive_sender, directive_receiver) =
            network.register(TASK_DIRECTIVE_CHANNEL, p2p_quota, p2p_backlog);

        // Validator: recomputes the expected digest, binding the NCN from OUR
        // deployment config (never the router's bytes).
        let validator: Arc<dyn ValidatorTrait<RoundTaskData>> = Arc::new(RoundValidator::new(ncn));

        // TaskBook actor: owns the router's per-height directives.
        let (task_book, task_book_mailbox) = TaskBook::new(context.child("task_book"));
        context
            .child("task_book_actor")
            .spawn(move |_| task_book.run());

        // Shared engine-tip mirror for TipReport replies to stale directives.
        let engine_tip = Arc::new(AtomicU64::new(0));

        // Feed the TaskBook from channel 1 (router directives only).
        {
            let task_book_mailbox = task_book_mailbox.clone();
            let router_key = router_key.clone();
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
        let (node_reporter, reporter_mailbox) = NodeReporter::<_, JitoBn254Scheme>::new(
            context.child("reporter"),
            task_book_mailbox.clone(),
            Arc::clone(&engine_tip),
            APPLICATION_NAMESPACE.to_vec(),
        );
        context
            .child("reporter_actor")
            .spawn(move |_| node_reporter.run());

        // Automaton: resolves each proposed height to the validated digest or
        // the skip digest, per the TaskBook.
        let automaton = NodeAutomaton::new(
            context.child("automaton"),
            task_book_mailbox,
            validator,
            APPLICATION_NAMESPACE.to_vec(),
            round_timeout(),
        );

        // Static single-epoch supervision.
        let provider = ConstantProvider::<JitoBn254Scheme, Epoch>::new(scheme);
        let monitor = commonware_avs_core::consensus::StaticEpochMonitor::new();
        let monitor_guard = monitor.clone();

        let engine = Engine::new(
            context.child("engine"),
            AggregationConfig {
                monitor,
                provider,
                automaton,
                reporter: reporter_mailbox,
                blocker: oracle.clone(),
                priority_acks: false,
                rebroadcast_timeout: NonZeroDuration::new_panic(rebroadcast_interval()),
                epoch_bounds: (EpochDelta::new(0), EpochDelta::new(0)),
                window: agg_window(),
                activity_timeout: HeightDelta::new(agg_activity_timeout()),
                journal_partition: format!("aggregation-solana-node-{}", signer.public_key()),
                journal_write_buffer: NZUsize!(4096),
                journal_replay_buffer: NZUsize!(4096),
                journal_heights_per_section: NZU64!(6),
                journal_compression: Some(3),
                journal_page_cache: CacheRef::from_pooler(&context, NZU16!(1024), NZUsize!(10)),
                strategy: Sequential,
            },
        );
        engine.start((engine_sender, engine_receiver));

        // Blocks until the network shuts down; children abort on return.
        let _ = network.start().await;
        drop(monitor_guard);
    });
}
