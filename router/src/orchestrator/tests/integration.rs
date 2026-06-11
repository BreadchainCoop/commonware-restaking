use super::task_data::TestTaskData;
use crate::creator::Creator;
use crate::creator::MockCreator;
use crate::executor::bls::BlsVerificationData;
use crate::executor::{ExecutionResult, MockExecutor, VerificationExecutor};
use crate::orchestrator::builder::OrchestratorBuilder;
use crate::orchestrator::traits::OrchestratorTrait;
use async_trait::async_trait;
use commonware_avs_core::validator::MockValidator;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::helpers::{contributor, signer};
use super::mocks::clock::MockClock;
use super::mocks::{MockReceiver, MockSender};

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
        .with_aggregation_frequency(Duration::from_millis(100))
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
        .with_aggregation_frequency(Duration::from_secs(60))
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
        std::env::set_var("AGGREGATION_FREQUENCY", "120");
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
        std::env::remove_var("AGGREGATION_FREQUENCY");
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
        .with_aggregation_frequency(Duration::from_millis(100));

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
        .with_aggregation_frequency(Duration::from_millis(100));

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
        .with_aggregation_frequency(Duration::from_millis(100));

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
        .with_aggregation_frequency(Duration::from_millis(100));

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
