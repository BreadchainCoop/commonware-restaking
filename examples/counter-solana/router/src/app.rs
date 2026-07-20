//! Solana counter router: verifier-only aggregation engine + task sequencer +
//! on-chain submitter — the Solana peer of `counter-router`.
//!
//! The router is NOT a signing participant. It runs the commonware
//! aggregation engine with a verifier-only [`JitoBn254Scheme`]
//! (`me() == None`): the engine validates the nodes' acks on channel 0,
//! assembles certificates at quorum, journals them, and reports them to the
//! `CertReporter`. Task flow: task source → sequencer (assigns aggregation
//! heights, broadcasts `TaskDirective`s on channel 1) → nodes sign → engine
//! certifies → `JitoSubmitter` resolves the height only at `finalized`.
//!
//! Two modes share the same binary and engine plumbing:
//!
//! - default: the counter demo — locally monotonic rounds, consumed by the
//!   NCN program's stateless `VerifyCertificate`
//!   ([`VerifyCertificateHandler`]);
//! - `LLM_SETTLE=1`: the LLM settlement leg — the producer fixture's
//!   `SettlementPayload` (env `LLM_PAYLOAD_FIXTURE`), sequenced exactly once
//!   and landed via the gaskiller-settlement program's `Settle` instruction
//!   ([`SettleCertificateHandler`], INTERFACES.md §4).
//!
//! Chain interaction is real (operator discovery, snapshot stake, transaction
//! submission); the example takes a deployment config — nothing is mocked.

mod llm_source;
mod source;

use anyhow::Result;
use clap::{Arg, Command};
use commonware_avs_core::wire::TaskData;
use commonware_avs_jito::bn254::{Bn254Signer, PublicKey};
use commonware_avs_jito::scheme::JitoBn254Scheme;
use commonware_avs_jito::{
    JitoStakingClient, JitoSubmitter, NcnDeployment, SettleCertificateHandler,
    SolanaCertificateHandler, VerifyCertificateHandler,
};
use commonware_avs_router::automaton::RouterAutomaton;
use commonware_avs_router::reporter::{CertReporter, certified_channel};
use commonware_avs_router::sequencer::{
    DispatchTime, Sequencer, TaskSource, TipReports, ingest_tip_reports, resolution_channel,
    shared_assignments,
};
use commonware_consensus::aggregation::{Config as AggregationConfig, Engine};
use commonware_consensus::types::{Epoch, EpochDelta, HeightDelta};
use commonware_cryptography::Signer as _;
use commonware_cryptography::certificate::{ConstantProvider, Scheme as _};
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
    APPLICATION_NAMESPACE, LLM_APPLICATION_NAMESPACE, LlmTaskData, RoundTaskData,
    ack_messages_per_second, agg_activity_timeout, agg_window, load_bn254_key, p2p_message_backlog,
    p2p_quota_period, rebroadcast_interval, round_timeout, storage_directory,
};
use governor::Quota;
use std::collections::{HashMap, HashSet};
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// P2p channel carrying the aggregation engine's acks (engine-internal).
const ACK_CHANNEL: u64 = 0;

/// P2p channel on which the router broadcasts `TaskDirective`s to the nodes
/// (and receives their rate-limited TipReport replies).
const DIRECTIVE_CHANNEL: u64 = 1;

/// Journal partition for the router's verifier-only engine.
const JOURNAL_PARTITION: &str = "aggregation-solana-router";

pub fn main() {
    let matches = Command::new("counter-solana-router")
        .about("router for the Jito NCN counter demo")
        .arg(
            Arg::new("key-file")
                .long("key-file")
                .required(true)
                .help("Path to the JSON file with the router's BN254 private key"),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .required(true)
                .help("Port to run the p2p listener on"),
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
    let signer = load_bn254_key(key_file).expect("failed to load BN254 key");

    // Mode switch: the LLM settlement leg sequences the producer fixture's
    // payload once and lands it via `Settle`.
    let llm_mode = env::var("LLM_SETTLE").is_ok_and(|v| v == "1");
    if llm_mode {
        let fixture: PathBuf = env::var("LLM_PAYLOAD_FIXTURE")
            .expect("LLM_PAYLOAD_FIXTURE must be set in LLM_SETTLE mode")
            .into();
        run::<LlmTaskData, _, _, _, _>(
            signer,
            port,
            LLM_APPLICATION_NAMESPACE,
            move |deployment| {
                let settlement_program_id = deployment
                    .settlement_program_id()
                    .map_err(|e| anyhow::anyhow!("settlement program id: {e}"))?;
                let state_pda = deployment
                    .gk_state_pda()
                    .map_err(|e| anyhow::anyhow!("gk state pda: {e}"))?;
                let source = llm_source::LlmSource::from_fixture(
                    &fixture,
                    &settlement_program_id,
                    &state_pda,
                )?;
                if let Some(digest) = source.digest() {
                    tracing::info!(
                        digest = %digest,
                        state = %state_pda,
                        "LLM settle payload loaded; sequencing one transition"
                    );
                }
                Ok(source)
            },
            SettleCertificateHandler::new,
        );
    } else {
        run::<RoundTaskData, _, _, _, _>(
            signer,
            port,
            APPLICATION_NAMESPACE,
            |deployment| {
                let ncn = deployment
                    .ncn()
                    .map_err(|e| anyhow::anyhow!("ncn pubkey: {e}"))?;
                let start_round: u64 = env::var("TASK_START_ROUND")
                    .ok()
                    .and_then(|v| v.trim().parse().ok())
                    .unwrap_or(0);
                let polling_interval_ms: u64 = env::var("POLLING_INTERVAL_MS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(2_000);
                Ok(source::RoundSource::new(
                    ncn,
                    start_round,
                    Duration::from_millis(polling_interval_ms),
                ))
            },
            VerifyCertificateHandler::new,
        );
    }
}

/// Runs the verifier-only router for one task flavor. `source_for` builds the
/// task source and `handler_for` the on-chain certificate handler, both from
/// the loaded deployment config.
fn run<T, S, SF, H, HF>(
    signer: Bn254Signer,
    port: u16,
    namespace: &'static [u8],
    source_for: SF,
    handler_for: HF,
) where
    T: TaskData + PartialEq,
    S: TaskSource<T> + Send + 'static,
    SF: FnOnce(&NcnDeployment) -> Result<S> + Send + 'static,
    H: SolanaCertificateHandler<TaskData = T> + 'static,
    HF: FnOnce(&NcnDeployment, solana_sdk::signature::Keypair) -> Result<H> + Send + 'static,
{
    // A stable storage directory is REQUIRED for certificate journal replay.
    let storage_dir = storage_directory().join("solana-router");
    let runtime_cfg = tokio::Config::default().with_storage_directory(storage_dir.clone());
    let runner = tokio::Runner::new(runtime_cfg);

    runner.start(|context| async move {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(std::io::stdout)
            .finish();
        // Hold the guard for the life of the task — `let _ =` would drop it
        // immediately and silently disable all tracing output.
        let _tracing_guard = tracing::subscriber::set_default(subscriber);
        dotenv::dotenv().ok();

        let deployment = NcnDeployment::load().expect("NCN_DEPLOYMENT_PATH config");
        let client =
            JitoStakingClient::new(deployment.clone()).expect("staking client construction");
        let quorum = client
            .get_quorum()
            .await
            .expect("failed to load quorum from chain");

        // Startup quorum reconciliation (INTERFACES.md §5): refuse to start if
        // the lightest engine quorum cannot clear the on-chain threshold.
        quorum
            .reconcile_engine_quorum()
            .expect("refusing to start: engine quorum cannot clear the stake threshold");

        // Task source + on-chain handler for this mode.
        let task_source = source_for(&deployment).expect("task source construction");
        let payer_path =
            env::var("SOLANA_PAYER_KEYPAIR").expect("SOLANA_PAYER_KEYPAIR must be set");
        let payer = solana_sdk::signature::read_keypair_file(&payer_path)
            .expect("failed to read payer keypair file");
        let handler = handler_for(&deployment, payer).expect("handler construction");

        // Authorized peers: operators with registered sockets + ourselves
        // (nodes dial the router from its connection file; our entry is never
        // dialed by us).
        const MAX_MESSAGE_SIZE: u32 = 1024 * 1024; // 1 MB
        let my_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        let mut recipients: Vec<(PublicKey, Address)> = Vec::new();
        for operator in &quorum.operators {
            let Some(socket) = operator.socket else {
                tracing::warn!(operator = %operator.operator, "operator has no registered socket");
                continue;
            };
            recipients.push((operator.g2_pub_key.clone(), Address::from(socket)));
        }
        recipients.push((signer.public_key(), Address::from(my_addr)));

        tracing::info!(storage_dir = %storage_dir.display(), "engine journal storage directory");

        let mut p2p_cfg =
            lookup::Config::local(signer.clone(), namespace, my_addr, MAX_MESSAGE_SIZE);
        p2p_cfg.bypass_ip_check = true;
        let (mut network, mut oracle) = Network::new(context.child("network"), p2p_cfg);
        oracle.track(0, Map::from_iter_dedup(recipients));

        // Verifier-only scheme: the router validates acks and assembles
        // certificates but never signs (its key is not in the operator set).
        let scheme = JitoBn254Scheme::verifier(quorum.participants(), quorum.g1_keys());
        tracing::info!(
            participants = scheme.participants().len(),
            "operator set loaded"
        );

        // Register channels (must precede network.start()).
        let p2p_backlog = p2p_message_backlog();
        let p2p_quota = Quota::with_period(p2p_quota_period())
            .expect("p2p_quota_period always returns a non-zero duration");
        let ack_quota = Quota::per_second(ack_messages_per_second());
        let (ack_sender, ack_receiver) = network.register(ACK_CHANNEL, ack_quota, p2p_backlog);
        let (directive_sender, directive_receiver) =
            network.register(DIRECTIVE_CHANNEL, p2p_quota, p2p_backlog);

        // State shared across sequencer / automaton / submitter.
        let assignments = shared_assignments::<T>();
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
                monitor: commonware_avs_core::consensus::StaticEpochMonitor::new(),
                provider: ConstantProvider::<JitoBn254Scheme, Epoch>::new(scheme.clone()),
                automaton: RouterAutomaton::new(assignments.clone()),
                reporter: reporter_mailbox.clone(),
                blocker: oracle.clone(),
                priority_acks: false,
                rebroadcast_timeout: NonZeroDuration::new_panic(rebroadcast_interval()),
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

        // On-chain submitter: resolved only at `finalized`, blockhash expiry
        // -> rebuild and resend. The payer keypair funds transaction fees.
        let submitter = JitoSubmitter::new(
            scheme.clone(),
            quorum.operator_indices(),
            quorum.operators_registered,
            // Captured at quorum-assembly time from the on-chain
            // Snapshot.generation (bumped on register/remove/rotation).
            quorum.generation,
            handler,
            assignments.clone(),
            certified_receiver,
            resolution_sender,
            namespace.to_vec(),
        );
        context.child("submitter").spawn(move |_| submitter.run());

        // Node tip reports (channel 1, node → router): journal-loss recovery.
        let tip_reports = TipReports::new(scheme.participants().len());
        {
            let participant_keys: HashSet<PublicKey> =
                scheme.participants().iter().cloned().collect();
            let tip_reports = tip_reports.clone();
            context.child("tip_reports").spawn(move |_| async move {
                ingest_tip_reports::<T, _, _>(directive_receiver, participant_keys, tip_reports)
                    .await;
            });
        }

        // Sequencer: pulls tasks, assigns heights, broadcasts directives to
        // the operator set (explicit keys — see Sequencer::broadcast).
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

        // Blocks until shutdown; children abort when this returns.
        let _ = network.start().await;
    });
}
