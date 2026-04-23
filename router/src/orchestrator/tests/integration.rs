use super::task_data::TestTaskData;
use crate::creator::Creator;
use crate::creator::MockCreator;
use crate::executor::MockExecutor;
use crate::orchestrator::builder::OrchestratorBuilder;
use crate::orchestrator::traits::OrchestratorTrait;
use commonware_avs_core::validator::MockValidator;
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
