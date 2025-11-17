use crate::creator::MockCreator;
use crate::creator::core::Creator;
use crate::executor::MockExecutor;
use crate::orchestrator::generic::{Orchestrator, OrchestratorConfig};
use commonware_avs_core::validator::MockValidator;
use commonware_usecase_counter::creator::CounterTaskData;
use std::time::Duration;

use super::helpers::{contributor, signer};
use super::mocks::clock::MockClock;

#[tokio::test]
async fn test_orchestrator_new() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let config = OrchestratorConfig {
        aggregation_frequency: Duration::from_secs(30),
        contributors: contributors.clone(),
        g1_map: g1_map.clone(),
        threshold: 2,
    };

    let task_creator = MockCreator::<CounterTaskData>::new();
    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = Orchestrator::new(
        clock.clone(),
        signer,
        config,
        task_creator,
        executor,
        validator,
    );

    // Test that we can access the components through public methods
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
async fn test_orchestrator_task_creator_metadata() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let metadata = CounterTaskData {
        var1: "custom_key".to_string(),
        var2: "custom_value".to_string(),
        var3: "42".to_string(),
    };

    let config = OrchestratorConfig {
        aggregation_frequency: Duration::from_secs(30),
        contributors,
        g1_map,
        threshold: 2,
    };

    let task_creator = MockCreator::<CounterTaskData>::new().with_metadata(metadata.clone());
    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = Orchestrator::new(
        clock.clone(),
        signer,
        config,
        task_creator,
        executor,
        validator,
    );

    // Test that we can access task creator metadata
    let creator_metadata = orchestrator.task_creator().get_task_metadata();
    assert_eq!(creator_metadata, metadata);
}

#[tokio::test]
async fn test_orchestrator_executor_access() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let config = OrchestratorConfig {
        aggregation_frequency: Duration::from_secs(30),
        contributors,
        g1_map,
        threshold: 2,
    };

    let task_creator = MockCreator::<CounterTaskData>::new();
    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = Orchestrator::new(
        clock.clone(),
        signer,
        config,
        task_creator,
        executor,
        validator,
    );

    // Test that we can access the executor
    let executor_ref = orchestrator.executor();
    assert_eq!(executor_ref.get_execution_count(), 0);
}

#[tokio::test]
async fn test_orchestrator_validator_access() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let config = OrchestratorConfig {
        aggregation_frequency: Duration::from_secs(30),
        contributors,
        g1_map,
        threshold: 2,
    };

    let task_creator = MockCreator::<CounterTaskData>::new();
    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = Orchestrator::new(
        clock.clone(),
        signer,
        config,
        task_creator,
        executor,
        validator,
    );

    // Test that we can access the validator
    let validator_ref = orchestrator.validator();
    assert_eq!(validator_ref.get_validation_count(), 0);
}

#[tokio::test]
async fn test_orchestrator_config_creation() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let config = OrchestratorConfig {
        aggregation_frequency: Duration::from_secs(45),
        contributors: contributors.clone(),
        g1_map: g1_map.clone(),
        threshold: 3,
    };

    let task_creator = MockCreator::<CounterTaskData>::new();
    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = Orchestrator::new(
        clock.clone(),
        signer,
        config,
        task_creator,
        executor,
        validator,
    );

    // Verify we can access components and they work correctly
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
async fn test_orchestrator_threshold_validation() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    // Test with threshold equal to number of contributors
    let config = OrchestratorConfig {
        aggregation_frequency: Duration::from_secs(30),
        contributors: contributors.clone(),
        g1_map: g1_map.clone(),
        threshold: 3, // Equal to number of contributors
    };

    let task_creator = MockCreator::<CounterTaskData>::new();
    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = Orchestrator::new(
        clock.clone(),
        signer,
        config,
        task_creator,
        executor,
        validator,
    );

    // Test that components are accessible and working
    let metadata = orchestrator.task_creator().get_task_metadata();
    assert!(!metadata.var1.is_empty());
    assert!(!metadata.var2.is_empty());
    assert!(!metadata.var3.is_empty());

    let executor_count = orchestrator.executor().get_execution_count();
    assert_eq!(executor_count, 0);
}

#[tokio::test]
async fn test_orchestrator_component_interaction() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let config = OrchestratorConfig {
        aggregation_frequency: Duration::from_secs(30),
        contributors,
        g1_map,
        threshold: 2,
    };

    let task_creator = MockCreator::<CounterTaskData>::new();
    let executor = MockExecutor::new();
    let validator = MockValidator::new_success(1);

    let orchestrator = Orchestrator::new(
        clock.clone(),
        signer,
        config,
        task_creator,
        executor,
        validator,
    );

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
