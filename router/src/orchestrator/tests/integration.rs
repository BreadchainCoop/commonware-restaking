use super::task_data::TestTaskData;
use crate::creator::Creator;
use crate::creator::MockCreator;
use crate::executor::bls::BlsVerificationData;
use crate::executor::{ExecutionResult, MockExecutor, VerificationExecutor};
use crate::orchestrator::builder::OrchestratorBuilder;
use crate::orchestrator::traits::OrchestratorTrait;
use anyhow::Result;
use async_trait::async_trait;
use commonware_avs_core::validator::MockValidator;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::helpers::{contributor, signer};
use super::mocks::clock::MockClock;
use super::mocks::{MockReceiver, MockSender};

/// Wraps a `MockCreator` and separately counts calls to `get_payload_and_round` vs
/// `wait_for_new_round`. Used to assert that the orchestrator does not re-call
/// `get_payload_and_round` after a successful execution (the double-consume bug).
struct TrackingCreator {
    inner: MockCreator<TestTaskData>,
    get_count: Arc<AtomicUsize>,
    wait_count: Arc<AtomicUsize>,
}

#[async_trait]
impl Creator for TrackingCreator {
    type TaskData = TestTaskData;

    async fn get_payload_and_round(&self) -> Result<(Vec<u8>, u64)> {
        self.get_count.fetch_add(1, Ordering::SeqCst);
        self.inner.get_payload_and_round().await
    }

    async fn wait_for_new_round(&self, current: u64) -> Result<(Vec<u8>, u64)> {
        self.wait_count.fetch_add(1, Ordering::SeqCst);
        self.inner.wait_for_new_round(current).await
    }

    fn get_task_metadata(&self) -> TestTaskData {
        self.inner.get_task_metadata()
    }
}

/// Returns `Err` on the first call to `wait_for_new_round`, then delegates to the inner
/// creator. Used to assert that the orchestrator handles the error gracefully (no panic)
/// rather than unwrapping.
struct ErrorOnceCreator {
    inner: MockCreator<TestTaskData>,
    first_wait: Arc<Mutex<bool>>,
}

#[async_trait]
impl Creator for ErrorOnceCreator {
    type TaskData = TestTaskData;

    async fn get_payload_and_round(&self) -> Result<(Vec<u8>, u64)> {
        self.inner.get_payload_and_round().await
    }

    async fn wait_for_new_round(&self, current: u64) -> Result<(Vec<u8>, u64)> {
        let is_first = {
            let mut guard = self.first_wait.lock().unwrap();
            let was = *guard;
            *guard = false;
            was
        };
        if is_first {
            Err(anyhow::anyhow!("simulated queue timeout"))
        } else {
            self.inner.wait_for_new_round(current).await
        }
    }

    fn get_task_metadata(&self) -> TestTaskData {
        self.inner.get_task_metadata()
    }
}

#[tokio::test]
async fn test_orchestrator_builder_integration() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    // Test the full builder workflow
    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors.clone())
        .with_g1_map(g1_map.clone())
        .with_threshold(2)
        .with_round_timeout(Duration::from_millis(100))
        .with_ingress("127.0.0.1:8080".to_string());

    let task_creator = MockCreator::<TestTaskData>::new();
    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = builder
        .build(task_creator, executor, validator)
        .expect("Failed to build orchestrator");

    // Verify the orchestrator was built correctly by testing public methods
    let metadata = orchestrator.task_creator().get_task_metadata();
    assert!(!metadata.var1.is_empty());
    assert!(!metadata.var2.is_empty());
    assert!(!metadata.var3.is_empty());

    let executor_count = orchestrator.executor().get_execution_count();
    assert_eq!(executor_count, 0);

    let validator_count = orchestrator.validator().get_validation_count();
    assert_eq!(validator_count, 0);
}

#[tokio::test]
async fn test_orchestrator_metadata_integration() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let custom_metadata = TestTaskData {
        var1: "integration_test".to_string(),
        var2: "true".to_string(),
        var3: "metadata_verification".to_string(),
    };

    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_threshold(2);

    let task_creator = MockCreator::<TestTaskData>::new().with_metadata(custom_metadata.clone());
    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = builder
        .build(task_creator, executor, validator)
        .expect("Failed to build orchestrator");

    // Verify metadata is accessible through the orchestrator
    let metadata = orchestrator.task_creator().get_task_metadata();
    assert_eq!(metadata, custom_metadata);
    assert_eq!(metadata.var1, "integration_test");
    assert_eq!(metadata.var2, "true");
    assert_eq!(metadata.var3, "metadata_verification");
}

#[tokio::test]
async fn test_orchestrator_component_access_integration() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_threshold(2);

    let task_creator = MockCreator::<TestTaskData>::new();
    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = builder
        .build(task_creator, executor, validator)
        .expect("Failed to build orchestrator");

    // Test access to all components
    let creator_metadata = orchestrator.task_creator().get_task_metadata();
    assert!(!creator_metadata.var1.is_empty());
    assert!(!creator_metadata.var2.is_empty());
    assert!(!creator_metadata.var3.is_empty());

    let executor_count = orchestrator.executor().get_execution_count();
    assert_eq!(executor_count, 0);

    let validator_count = orchestrator.validator().get_validation_count();
    assert_eq!(validator_count, 0);
}

#[tokio::test]
async fn test_orchestrator_config_integration() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    // Test with various configuration combinations
    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors.clone())
        .with_g1_map(g1_map.clone())
        .with_threshold(3)
        .with_round_timeout(Duration::from_secs(60))
        .with_ingress("0.0.0.0:9090".to_string());

    let task_creator = MockCreator::<TestTaskData>::new();
    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = builder
        .build(task_creator, executor, validator)
        .expect("Failed to build orchestrator");

    // Verify all configuration is properly applied by testing component behavior
    let metadata = orchestrator.task_creator().get_task_metadata();
    assert!(!metadata.var1.is_empty());
    assert!(!metadata.var2.is_empty());
    assert!(!metadata.var3.is_empty());

    let executor_count = orchestrator.executor().get_execution_count();
    assert_eq!(executor_count, 0);

    let validator_count = orchestrator.validator().get_validation_count();
    assert_eq!(validator_count, 0);
}

#[tokio::test]
async fn test_orchestrator_validation_integration() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    // Test validation with different thresholds
    for threshold in 1..=3 {
        let builder = OrchestratorBuilder::new(clock.clone(), signer.clone())
            .with_contributors(contributors.clone())
            .with_g1_map(g1_map.clone())
            .with_threshold(threshold);

        let task_creator = MockCreator::<TestTaskData>::new();
        let executor = MockExecutor::new();
        let validator = MockValidator::new_success(1);

        let orchestrator = builder
            .build(task_creator, executor, validator)
            .expect("Failed to build orchestrator");

        // Verify the orchestrator was built successfully
        let metadata = orchestrator.task_creator().get_task_metadata();
        assert!(!metadata.var1.is_empty());
        assert!(!metadata.var2.is_empty());
        assert!(!metadata.var3.is_empty());

        let executor_count = orchestrator.executor().get_execution_count();
        assert_eq!(executor_count, 0);
    }
}

#[tokio::test]
async fn test_orchestrator_environment_integration() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    // Set environment variables
    unsafe {
        std::env::set_var("INGRESS", "true");
        std::env::set_var("INGRESS_ADDRESS", "127.0.0.1:7070");
        std::env::set_var("ROUND_TIMEOUT", "120");
        std::env::set_var("THRESHOLD", "2");
    }

    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .load_from_env();

    let task_creator = MockCreator::<TestTaskData>::new();
    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = builder
        .build(task_creator, executor, validator)
        .expect("Failed to build orchestrator");

    // Verify environment variables were applied by testing component behavior
    let metadata = orchestrator.task_creator().get_task_metadata();
    assert!(!metadata.var1.is_empty());
    assert!(!metadata.var2.is_empty());
    assert!(!metadata.var3.is_empty());

    let executor_count = orchestrator.executor().get_execution_count();
    assert_eq!(executor_count, 0);

    // Clean up environment variables
    unsafe {
        std::env::remove_var("INGRESS");
        std::env::remove_var("INGRESS_ADDRESS");
        std::env::remove_var("ROUND_TIMEOUT");
        std::env::remove_var("THRESHOLD");
    }
}

#[tokio::test]
async fn test_orchestrator_component_interaction() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_threshold(2);

    let task_creator = MockCreator::<TestTaskData>::new();
    let executor = MockExecutor::new().with_success(true);
    let validator = MockValidator::new_success(1);

    let orchestrator = builder
        .build(task_creator, executor, validator)
        .expect("Failed to build orchestrator");

    // Test that components can interact properly
    let (payload, round) = orchestrator
        .task_creator()
        .get_payload_and_round()
        .await
        .expect("Failed to get payload and round");

    assert_eq!(round, 1);
    assert_eq!(payload, round.to_le_bytes().to_vec());

    let metadata = orchestrator.task_creator().get_task_metadata();
    assert!(!metadata.var1.is_empty());
    assert!(!metadata.var2.is_empty());
    assert!(!metadata.var3.is_empty());

    // Test executor interaction
    let executor_ref = orchestrator.executor();
    assert_eq!(executor_ref.get_execution_count(), 0);

    // Test validator interaction
    let validator_ref = orchestrator.validator();
    assert_eq!(validator_ref.get_validation_count(), 0);
}

/// Verify that execution fires exactly once for a round even when more signatures arrive
/// after the threshold has been reached.
///
/// Uses `start_paused = true` so `tokio::time::sleep` in MockClock::sleep_until is controlled
/// by `tokio::time::advance` rather than real wall time.
#[tokio::test(start_paused = true)]
async fn test_executor_called_exactly_once_after_threshold() {
    use alloy::primitives::U256;
    use alloy::sol_types::SolValue;
    use bytes::Bytes;
    use commonware_avs_core::wire::{Aggregation, aggregation::Payload};
    use commonware_codec::{EncodeSize, Write};
    use commonware_cryptography::{Hasher, Sha256, Signer};
    use tokio::sync::mpsc::unbounded_channel;

    let clock = MockClock::new();
    let orchestrator_signer = signer::create_test_signer();
    let (contributors, g1_map, contributor_signers) =
        contributor::create_test_contributors_with_signers();

    // threshold=2, 3 contributors → we will send 3 signatures so execution fires at #2 and #3
    // is ignored.
    let builder = OrchestratorBuilder::new(clock, orchestrator_signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_threshold(2)
        .with_round_timeout(Duration::from_millis(100));

    let executor = MockExecutor::new();
    let exec_count = executor.execution_count_handle();
    let validator = MockValidator::new_success(1);

    let orchestrator = builder
        .build(MockCreator::<TestTaskData>::new(), executor, validator)
        .expect("failed to build orchestrator");

    // MockValidator::new_success(1) ignores the message bytes and always returns
    // Sha256(U256::from(1).abi_encode()). Reproduce the same digest so we can sign it.
    let expected_digest = {
        let payload = U256::from(1u64).abi_encode();
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        hasher.finalize()
    };

    // Enqueue threshold+1 = 3 signed messages for round 1 before the orchestrator starts.
    let (msg_tx, msg_rx) = unbounded_channel::<(commonware_avs_core::bn254::PublicKey, Bytes)>();
    for contributor_signer in &contributor_signers {
        let sig = contributor_signer.sign(None, expected_digest.as_ref());
        let msg = Aggregation::<TestTaskData>::new(
            1,
            TestTaskData::default(),
            Some(Payload::Signature(sig.to_vec())),
        );
        let mut buf = Vec::with_capacity(msg.encode_size());
        msg.write(&mut buf);
        msg_tx
            .send((contributor_signer.public_key(), Bytes::from(buf)))
            .unwrap();
    }

    let handle = tokio::spawn(async move {
        orchestrator
            .run(MockSender::new(), MockReceiver::new(msg_rx))
            .await;
    });

    // Keep msg_tx alive until after advancing time so the receiver stays open (Pending rather
    // than Err) once the channel drains, letting the inner select! block on it rather than
    // spinning on errors until the timer fires.
    tokio::time::advance(Duration::from_millis(200)).await;
    tokio::task::yield_now().await;

    assert_eq!(
        *exec_count.lock().unwrap(),
        1,
        "executor should fire exactly once at threshold"
    );

    drop(msg_tx);
    handle.abort();
}

/// Records the signer counts from each `BlsVerificationData` it receives.
struct RecordingBlsExecutor {
    /// (signatures, public_keys, g1_public_keys) counts from the last call.
    received: Arc<Mutex<Option<(usize, usize, usize)>>>,
    /// Round number from the last call.
    received_round: Arc<Mutex<Option<u64>>>,
}

#[async_trait]
impl VerificationExecutor<TestTaskData, BlsVerificationData> for RecordingBlsExecutor {
    async fn execute_verification(
        &mut self,
        round: u64,
        _payload_hash: &[u8],
        verification_data: BlsVerificationData,
        _task_data: Option<&TestTaskData>,
    ) -> anyhow::Result<ExecutionResult> {
        *self.received_round.lock().unwrap() = Some(round);
        *self.received.lock().unwrap() = Some((
            verification_data.signatures.len(),
            verification_data.public_keys.len(),
            verification_data.g1_public_keys.len(),
        ));
        Ok(ExecutionResult {
            transaction_hash: "typed".to_string(),
            block_number: None,
            gas_used: None,
            status: Some(true),
            contract_address: None,
        })
    }
}

/// Built with `VD = BlsVerificationData`, the orchestrator delivers one entry per
/// participating signer to the executor.
#[tokio::test(start_paused = true)]
async fn test_orchestrator_passes_typed_bls_data() {
    use alloy::primitives::U256;
    use alloy::sol_types::SolValue;
    use bytes::Bytes;
    use commonware_avs_core::wire::{Aggregation, aggregation::Payload};
    use commonware_codec::{EncodeSize, Write};
    use commonware_cryptography::{Hasher, Sha256, Signer};
    use tokio::sync::mpsc::unbounded_channel;

    let clock = MockClock::new();
    let orchestrator_signer = signer::create_test_signer();
    let (contributors, g1_map, contributor_signers) =
        contributor::create_test_contributors_with_signers();

    let builder = OrchestratorBuilder::new(clock, orchestrator_signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_threshold(2)
        .with_round_timeout(Duration::from_millis(100));

    let received = Arc::new(Mutex::new(None));
    let received_round = Arc::new(Mutex::new(None));
    let executor = RecordingBlsExecutor {
        received: received.clone(),
        received_round: received_round.clone(),
    };
    let validator = MockValidator::new_success(1);

    let orchestrator = builder
        .build_with::<_, _, _, BlsVerificationData>(
            MockCreator::<TestTaskData>::new(),
            executor,
            validator,
        )
        .expect("failed to build orchestrator");

    // MockValidator::new_success(1) returns Sha256(U256::from(1).abi_encode()) regardless of
    // the message bytes; reproduce it so the contributor signatures verify and aggregate.
    let expected_digest = {
        let payload = U256::from(1u64).abi_encode();
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        hasher.finalize()
    };

    // Enqueue exactly the threshold (2) signed messages for round 1.
    let (msg_tx, msg_rx) = unbounded_channel::<(commonware_avs_core::bn254::PublicKey, Bytes)>();
    for contributor_signer in contributor_signers.iter().take(2) {
        let sig = contributor_signer.sign(None, expected_digest.as_ref());
        let msg = Aggregation::<TestTaskData>::new(
            1,
            TestTaskData::default(),
            Some(Payload::Signature(sig.to_vec())),
        );
        let mut buf = Vec::with_capacity(msg.encode_size());
        msg.write(&mut buf);
        msg_tx
            .send((contributor_signer.public_key(), Bytes::from(buf)))
            .unwrap();
    }

    let handle = tokio::spawn(async move {
        orchestrator
            .run(MockSender::new(), MockReceiver::new(msg_rx))
            .await;
    });

    tokio::time::advance(Duration::from_millis(200)).await;
    tokio::task::yield_now().await;

    assert_eq!(
        *received.lock().unwrap(),
        Some((2, 2, 2)),
        "orchestrator should deliver typed BlsVerificationData with one entry per signer"
    );
    assert_eq!(
        *received_round.lock().unwrap(),
        Some(1),
        "orchestrator should pass the round number through to the executor"
    );

    drop(msg_tx);
    handle.abort();
}

/// Metrics registered by the orchestrator are observable through the runtime context's
/// `Metrics::encode` on the quorum path: accepted signatures, time-to-quorum, and a
/// successful execution are all recorded, and a late signature for the executed round
/// is counted as dropped.
#[tokio::test(start_paused = true)]
async fn test_metrics_observed_on_quorum_path() {
    use alloy::primitives::U256;
    use alloy::sol_types::SolValue;
    use bytes::Bytes;
    use commonware_avs_core::wire::{Aggregation, aggregation::Payload};
    use commonware_codec::{EncodeSize, Write};
    use commonware_cryptography::{Hasher, Sha256, Signer};
    use commonware_runtime::Metrics;
    use tokio::sync::mpsc::unbounded_channel;

    let clock = MockClock::new();
    let orchestrator_signer = signer::create_test_signer();
    let (contributors, g1_map, contributor_signers) =
        contributor::create_test_contributors_with_signers();

    // threshold=2, 3 signatures for round 1: the first two are accepted and trigger
    // execution; the third arrives after the round executed and is dropped.
    let builder = OrchestratorBuilder::new(clock.clone(), orchestrator_signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_threshold(2)
        .with_round_timeout(Duration::from_millis(100));

    let orchestrator = builder
        .build(
            MockCreator::<TestTaskData>::new(),
            MockExecutor::new(),
            MockValidator::new_success(1),
        )
        .expect("failed to build orchestrator");

    let expected_digest = {
        let payload = U256::from(1u64).abi_encode();
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        hasher.finalize()
    };

    let (msg_tx, msg_rx) = unbounded_channel::<(commonware_avs_core::bn254::PublicKey, Bytes)>();
    for contributor_signer in &contributor_signers {
        let sig = contributor_signer.sign(None, expected_digest.as_ref());
        let msg = Aggregation::<TestTaskData>::new(
            1,
            TestTaskData::default(),
            Some(Payload::Signature(sig.to_vec())),
        );
        let mut buf = Vec::with_capacity(msg.encode_size());
        msg.write(&mut buf);
        msg_tx
            .send((contributor_signer.public_key(), Bytes::from(buf)))
            .unwrap();
    }

    let handle = tokio::spawn(async move {
        orchestrator
            .run(MockSender::new(), MockReceiver::new(msg_rx))
            .await;
    });

    tokio::time::advance(Duration::from_millis(200)).await;
    tokio::task::yield_now().await;

    let encoded = clock.encode();
    for expected in [
        "orchestrator_signatures_total{status=\"Success\"} 2",
        "orchestrator_signatures_total{status=\"Dropped\"} 1",
        "orchestrator_round_executions_total{status=\"Success\"} 1",
        "orchestrator_time_to_quorum_seconds_count 1",
        "orchestrator_signature_arrival_seconds_count 2",
    ] {
        assert!(
            encoded.contains(expected),
            "expected `{expected}` in encoded metrics:\n{encoded}"
        );
    }

    drop(msg_tx);
    handle.abort();
}

/// An aggregation window that expires without reaching the signature threshold
/// increments the round-timeout counter, and the next round is started and broadcast.
#[tokio::test(start_paused = true)]
async fn test_metrics_round_timeout_counted() {
    use bytes::Bytes;
    use commonware_runtime::Metrics;
    use tokio::sync::mpsc::unbounded_channel;

    let clock = MockClock::new();
    let orchestrator_signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let builder = OrchestratorBuilder::new(clock.clone(), orchestrator_signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_threshold(2)
        .with_round_timeout(Duration::from_millis(100));

    let orchestrator = builder
        .build(
            MockCreator::<TestTaskData>::new(),
            MockExecutor::new(),
            MockValidator::new_success(1),
        )
        .expect("failed to build orchestrator");

    // Keep the sender alive so the receiver stays open; no signatures are ever sent.
    let (msg_tx, msg_rx) = unbounded_channel::<(commonware_avs_core::bn254::PublicKey, Bytes)>();

    let handle = tokio::spawn(async move {
        orchestrator
            .run(MockSender::new(), MockReceiver::new(msg_rx))
            .await;
    });

    // Let the orchestrator run up to its first aggregation window before advancing,
    // so the window timer is registered against the pre-advance clock. Then advance
    // exactly one window: round 1 expires without any signatures, and round 2 is
    // broadcast with its window still pending.
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;

    let encoded = clock.encode();
    for expected in [
        "orchestrator_round_timeouts_total 1",
        "orchestrator_rounds_started_total 2",
        "orchestrator_round_broadcasts_total 2",
        // Histograms and the executions family are registered (and thus visible to
        // consumers) even before their first observation.
        "orchestrator_time_to_quorum_seconds",
        "orchestrator_signature_arrival_seconds",
        "orchestrator_round_executions",
    ] {
        assert!(
            encoded.contains(expected),
            "expected `{expected}` in encoded metrics:\n{encoded}"
        );
    }

    drop(msg_tx);
    handle.abort();
}

/// With a short `round_timeout` and a long `rebroadcast_interval`, the orchestrator
/// abandons rounds quickly without amplifying `Start` broadcasts.  Each round produces
/// exactly one broadcast (the initial one); the rebroadcast timer never fires within
/// any round because `rebroadcast_interval >> round_timeout`.
///
/// This is the core safety property of the decoupled knobs: operators can shrink
/// `round_timeout` for fast recovery without flooding the P2P channel with `Start`
/// messages.
#[tokio::test(start_paused = true)]
async fn test_short_round_timeout_no_rebroadcast_storm() {
    use bytes::Bytes;
    use commonware_runtime::Metrics;
    use tokio::sync::mpsc::unbounded_channel;

    let clock = MockClock::new();
    let orchestrator_signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    // round_timeout=100ms, rebroadcast_interval=10s: the rebroadcast timer never
    // fires during any round because the round times out first.
    let builder = OrchestratorBuilder::new(clock.clone(), orchestrator_signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_threshold(2)
        .with_round_timeout(Duration::from_millis(100))
        .with_rebroadcast_interval(Duration::from_secs(10));

    let orchestrator = builder
        .build(
            MockCreator::<TestTaskData>::new(),
            MockExecutor::new(),
            MockValidator::new_success(1),
        )
        .expect("failed to build orchestrator");

    let (msg_tx, msg_rx) = unbounded_channel::<(commonware_avs_core::bn254::PublicKey, Bytes)>();

    let handle = tokio::spawn(async move {
        orchestrator
            .run(MockSender::new(), MockReceiver::new(msg_rx))
            .await;
    });

    // Each round's timer is registered as tokio::time::sleep(100ms) from the moment
    // MockClock::sleep_until is called, so it fires 100ms of tokio-time after that
    // call — not at a fixed absolute offset. To drive N timeouts we must interleave
    // yield (let the task register the next timer) and advance (fire it).
    tokio::task::yield_now().await; // round 1 starts, registers sleep(100ms)
    for _ in 0..3 {
        tokio::time::advance(Duration::from_millis(100)).await; // fire current round's timer
        tokio::task::yield_now().await; // process timeout, start next round
    }

    let encoded = clock.encode();

    // 3 timeouts, 4 rounds started (round 4 open), 4 broadcasts.
    // broadcasts_total == rounds_started proves exactly one Start per round: no storm.
    for expected in [
        "orchestrator_round_timeouts_total 3",
        "orchestrator_rounds_started_total 4",
        "orchestrator_round_broadcasts_total 4",
    ] {
        assert!(
            encoded.contains(expected),
            "expected `{expected}` in encoded metrics:\n{encoded}"
        );
    }

    drop(msg_tx);
    handle.abort();
}

/// After a successful execution the orchestrator must use the `(payload, round)` returned by
/// `wait_for_new_round` directly for the next broadcast rather than discarding that result and
/// calling `get_payload_and_round` a second time. Calling `get_payload_and_round` an extra
/// time per executed round causes double-consumption of queue-backed creators:
/// `ListeningCounterCreator` pops a task from the shared queue on every call, so the extra
/// call silently discards the task that should drive the next round.
#[tokio::test(start_paused = true)]
async fn test_get_payload_not_recalled_after_execution() {
    use alloy::primitives::U256;
    use alloy::sol_types::SolValue;
    use bytes::Bytes;
    use commonware_avs_core::wire::{Aggregation, aggregation::Payload};
    use commonware_codec::{EncodeSize, Write};
    use commonware_cryptography::{Hasher, Sha256, Signer};
    use tokio::sync::mpsc::unbounded_channel;

    let clock = MockClock::new();
    let orchestrator_signer = signer::create_test_signer();
    let (contributors, g1_map, contributor_signers) =
        contributor::create_test_contributors_with_signers();

    let get_count = Arc::new(AtomicUsize::new(0));
    let wait_count = Arc::new(AtomicUsize::new(0));

    let creator = TrackingCreator {
        inner: MockCreator::<TestTaskData>::new(),
        get_count: get_count.clone(),
        wait_count: wait_count.clone(),
    };

    // Long aggregation timeout so round 2 does not time out before we snapshot counts.
    let builder = OrchestratorBuilder::new(clock, orchestrator_signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_threshold(2)
        .with_round_timeout(Duration::from_secs(10));

    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = builder
        .build(creator, executor, validator)
        .expect("failed to build orchestrator");

    // MockValidator::new_success(1) always returns Sha256(U256::from(1).abi_encode()).
    let expected_digest = {
        let payload = U256::from(1u64).abi_encode();
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        hasher.finalize()
    };

    let (msg_tx, msg_rx) = unbounded_channel::<(commonware_avs_core::bn254::PublicKey, Bytes)>();
    for contributor_signer in contributor_signers.iter().take(2) {
        let sig = contributor_signer.sign(None, expected_digest.as_ref());
        let msg = Aggregation::<TestTaskData>::new(
            1,
            TestTaskData::default(),
            Some(Payload::Signature(sig.to_vec())),
        );
        let mut buf = Vec::with_capacity(msg.encode_size());
        msg.write(&mut buf);
        msg_tx
            .send((contributor_signer.public_key(), Bytes::from(buf)))
            .unwrap();
    }

    let handle = tokio::spawn(async move {
        orchestrator
            .run(MockSender::new(), MockReceiver::new(msg_rx))
            .await;
    });

    // Advance 1 ms — enough to flush the pre-queued channel messages and let execution
    // fire, but far less than the 10 s aggregation timeout so round 2 has not yet timed
    // out and triggered another get_payload_and_round call.
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(1)).await;
    tokio::task::yield_now().await;

    // get_payload_and_round must have been called exactly once (for round 1).
    // wait_for_new_round must have been called exactly once (for the round 1 → 2 transition).
    // If the orchestrator discards the wait_for_new_round result and re-fetches, get_count == 2.
    assert_eq!(
        get_count.load(Ordering::SeqCst),
        1,
        "get_payload_and_round must not be re-called after execution; \
         the wait_for_new_round return value must drive the next round"
    );
    assert_eq!(
        wait_count.load(Ordering::SeqCst),
        1,
        "wait_for_new_round must be called exactly once after execution"
    );

    drop(msg_tx);
    handle.abort();
}

/// After `wait_for_new_round` returns `Err` the orchestrator must log the error and
/// retry rather than calling `.unwrap()` and panicking. A panicking orchestrator process
/// takes the whole service down; for a queue-backed creator an idle queue produces a
/// timeout error on every inter-round wait, so the panic is not a rare edge case.
#[tokio::test(start_paused = true)]
async fn test_orchestrator_survives_wait_for_new_round_error() {
    use alloy::primitives::U256;
    use alloy::sol_types::SolValue;
    use bytes::Bytes;
    use commonware_avs_core::wire::{Aggregation, aggregation::Payload};
    use commonware_codec::{EncodeSize, Write};
    use commonware_cryptography::{Hasher, Sha256, Signer};
    use tokio::sync::mpsc::unbounded_channel;

    let clock = MockClock::new();
    let orchestrator_signer = signer::create_test_signer();
    let (contributors, g1_map, contributor_signers) =
        contributor::create_test_contributors_with_signers();

    let creator = ErrorOnceCreator {
        inner: MockCreator::<TestTaskData>::new(),
        first_wait: Arc::new(Mutex::new(true)),
    };

    let builder = OrchestratorBuilder::new(clock, orchestrator_signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_threshold(2)
        .with_round_timeout(Duration::from_secs(10));

    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = builder
        .build(creator, executor, validator)
        .expect("failed to build orchestrator");

    let expected_digest = {
        let payload = U256::from(1u64).abi_encode();
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        hasher.finalize()
    };

    let (msg_tx, msg_rx) = unbounded_channel::<(commonware_avs_core::bn254::PublicKey, Bytes)>();
    for contributor_signer in contributor_signers.iter().take(2) {
        let sig = contributor_signer.sign(None, expected_digest.as_ref());
        let msg = Aggregation::<TestTaskData>::new(
            1,
            TestTaskData::default(),
            Some(Payload::Signature(sig.to_vec())),
        );
        let mut buf = Vec::with_capacity(msg.encode_size());
        msg.write(&mut buf);
        msg_tx
            .send((contributor_signer.public_key(), Bytes::from(buf)))
            .unwrap();
    }

    let handle = tokio::spawn(async move {
        orchestrator
            .run(MockSender::new(), MockReceiver::new(msg_rx))
            .await;
    });

    // Let execution fire and the first wait_for_new_round (which returns Err) run.
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(1)).await;
    tokio::task::yield_now().await;

    // The orchestrator must still be alive — not finished due to an unwrap panic.
    // With the current .unwrap() the spawned task finishes immediately after the panic.
    assert!(
        !handle.is_finished(),
        "orchestrator must not panic on wait_for_new_round error; \
         it should log and retry instead of calling .unwrap()"
    );

    drop(msg_tx);
    handle.abort();
}

/// A `rebroadcast_interval` shorter than `round_timeout` causes `Start` to be re-sent
/// within the same round.  The rebroadcast count increments independently of timeouts;
/// the round does not start over (rounds_started stays 1) and no timeout fires.
#[tokio::test(start_paused = true)]
async fn test_rebroadcast_fires_before_round_timeout() {
    use bytes::Bytes;
    use commonware_runtime::Metrics;
    use tokio::sync::mpsc::unbounded_channel;

    let clock = MockClock::new();
    let orchestrator_signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let builder = OrchestratorBuilder::new(clock.clone(), orchestrator_signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_threshold(2)
        .with_round_timeout(Duration::from_millis(300))
        .with_rebroadcast_interval(Duration::from_millis(100));

    let orchestrator = builder
        .build(
            MockCreator::<TestTaskData>::new(),
            MockExecutor::new(),
            MockValidator::new_success(1),
        )
        .expect("failed to build orchestrator");

    let (msg_tx, msg_rx) = unbounded_channel::<(commonware_avs_core::bn254::PublicKey, Bytes)>();

    let handle = tokio::spawn(async move {
        orchestrator
            .run(MockSender::new(), MockReceiver::new(msg_rx))
            .await;
    });

    // Let the orchestrator register its timers, then advance past the first
    // rebroadcast_interval (100ms) but well short of round_timeout (300ms).
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;

    let encoded = clock.encode();

    // The initial broadcast plus one rebroadcast within the same round.
    assert!(
        encoded.contains("orchestrator_round_broadcasts_total 2"),
        "expected two broadcasts (initial + rebroadcast) but got:\n{encoded}"
    );
    // The round has not timed out — still on round 1.
    assert!(
        encoded.contains("orchestrator_rounds_started_total 1"),
        "expected round 1 to still be open but got:\n{encoded}"
    );
    // No timeout has fired.
    assert!(
        !encoded.contains("orchestrator_round_timeouts_total 1"),
        "expected no timeout yet but got:\n{encoded}"
    );

    drop(msg_tx);
    handle.abort();
}

/// Verifies that a `DurationProvider` that changes its return value between rounds causes
/// the orchestrator to apply the new timeout on the next round.
///
/// Setup: the provider starts at 300 ms. After the first round times out we bump it to
/// 100 ms. We then confirm that the second round fires a timeout within 100 ms (not 300 ms),
/// which would only be possible if `round_timeout` was sampled at the start of each round.
#[tokio::test(start_paused = true)]
async fn test_dynamic_round_timeout_provider() {
    use commonware_runtime::Metrics;
    use std::sync::atomic::{AtomicU64, Ordering};

    let clock = MockClock::new();
    let orchestrator_signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    // Shared atomic millisecond value — starts at 300 ms, will be lowered to 100 ms.
    let timeout_ms = Arc::new(AtomicU64::new(300));
    let timeout_ms_clone = timeout_ms.clone();
    let provider: crate::orchestrator::types::DurationProvider =
        Arc::new(move || Duration::from_millis(timeout_ms_clone.load(Ordering::Relaxed)));

    let builder = OrchestratorBuilder::new(clock.clone(), orchestrator_signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_threshold(2)
        .with_round_timeout_provider(provider)
        .with_rebroadcast_interval(Duration::from_secs(3600)); // effectively disabled

    let orchestrator = builder
        .build(
            MockCreator::<TestTaskData>::new(),
            MockExecutor::new(),
            MockValidator::new_success(1),
        )
        .expect("failed to build orchestrator");

    let (msg_tx, msg_rx) = tokio::sync::mpsc::unbounded_channel::<(
        commonware_avs_core::bn254::PublicKey,
        bytes::Bytes,
    )>();

    let handle = tokio::spawn(async move {
        orchestrator
            .run(MockSender::new(), MockReceiver::new(msg_rx))
            .await;
    });

    // Yield so the orchestrator starts and registers the first round timer at 300 ms.
    tokio::task::yield_now().await;

    // Change the provider to 100 ms while round 1 is still running. Round 2 will pick
    // this up when it calls the provider at the start of the new round.
    timeout_ms.store(100, Ordering::Relaxed);

    // Advance past the first 300 ms timeout — round 1 times out, round 2 starts and
    // registers a new timer using the updated provider value (100 ms from now).
    tokio::time::advance(Duration::from_millis(300)).await;
    tokio::task::yield_now().await;

    let encoded = clock.encode();
    assert!(
        encoded.contains("orchestrator_round_timeouts_total 1"),
        "expected one timeout after 300 ms but got:\n{encoded}"
    );
    assert!(
        encoded.contains("orchestrator_rounds_started_total 2"),
        "expected round 2 to have started but got:\n{encoded}"
    );

    // Advance only 100 ms — only sufficient if round 2 used the updated provider (100 ms),
    // not the stale value (300 ms). Proves the provider is sampled fresh each round.
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;

    let encoded = clock.encode();
    assert!(
        encoded.contains("orchestrator_round_timeouts_total 2"),
        "expected two timeouts — proves provider is sampled each round, but got:\n{encoded}"
    );
    assert!(
        encoded.contains("orchestrator_rounds_started_total 3"),
        "expected round 3 to have started but got:\n{encoded}"
    );

    drop(msg_tx);
    handle.abort();
}
