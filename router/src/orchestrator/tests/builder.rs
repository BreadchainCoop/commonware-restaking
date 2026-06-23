use super::task_data::TestTaskData;
use crate::creator::Creator;
use crate::creator::MockCreator;
use crate::executor::MockExecutor;
use crate::orchestrator::builder::{OrchestratorBuilder, OrchestratorBuilderConfig};
use commonware_avs_core::validator::MockValidator;
use std::time::Duration;

use super::helpers::{contributor, signer};
use super::mocks::clock::MockClock;

#[tokio::test]
async fn test_builder_new() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();

    let builder = OrchestratorBuilder::new(clock.clone(), signer);

    // Test that we can build with default configuration
    let (contributors, g1_map) = contributor::create_test_contributors();
    let builder = builder
        .with_contributors(contributors.clone())
        .with_g1_map(g1_map.clone());

    let config = builder.get_config().expect("Failed to get config");
    assert_eq!(config.contributors.len(), 3);
    assert_eq!(config.g1_map.len(), 3);
    assert_eq!(config.config.round_timeout, Duration::from_secs(30));
    assert_eq!(config.config.rebroadcast_interval, Duration::from_secs(30));
    assert_eq!(config.config.threshold, 3);
}

#[tokio::test]
async fn test_builder_with_contributors() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors.clone())
        .with_g1_map(g1_map);

    let config = builder.get_config().expect("Failed to get config");
    assert_eq!(config.contributors.len(), 3);
    assert_eq!(config.contributors, contributors);
}

#[tokio::test]
async fn test_builder_with_g1_map() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map.clone());

    let config = builder.get_config().expect("Failed to get config");
    assert_eq!(config.g1_map.len(), 3);
    assert_eq!(config.g1_map, g1_map);
}

#[tokio::test]
async fn test_builder_with_threshold() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_threshold(5);

    let config = builder.get_config().expect("Failed to get config");
    assert_eq!(config.config.threshold, 5);
}

#[tokio::test]
async fn test_builder_with_round_timeout_and_rebroadcast_interval() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();
    let round_timeout = Duration::from_secs(60);
    let rebroadcast_interval = Duration::from_secs(10);

    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_round_timeout(round_timeout)
        .with_rebroadcast_interval(rebroadcast_interval);

    let config = builder.get_config().expect("Failed to get config");
    assert_eq!(config.config.round_timeout, round_timeout);
    assert_eq!(config.config.rebroadcast_interval, rebroadcast_interval);
}

#[tokio::test]
async fn test_builder_with_ingress() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();
    let address = "127.0.0.1:8080".to_string();

    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .with_ingress(address.clone());

    let config = builder.get_config().expect("Failed to get config");
    assert!(config.config.use_ingress);
    assert_eq!(config.config.ingress_address, address);
}

#[tokio::test]
async fn test_builder_load_from_env() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    // Set environment variables for testing
    unsafe {
        std::env::set_var("INGRESS", "true");
        std::env::set_var("INGRESS_ADDRESS", "0.0.0.0:9090");
        std::env::set_var("ROUND_TIMEOUT", "45");
        std::env::set_var("REBROADCAST_INTERVAL", "5");
        std::env::set_var("THRESHOLD", "7");
    }

    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map)
        .load_from_env();

    let config = builder.get_config().expect("Failed to get config");
    assert!(config.config.use_ingress);
    assert_eq!(config.config.ingress_address, "0.0.0.0:9090");
    assert_eq!(config.config.round_timeout, Duration::from_secs(45));
    assert_eq!(config.config.rebroadcast_interval, Duration::from_secs(5));
    assert_eq!(config.config.threshold, 7);

    // Clean up environment variables
    unsafe {
        std::env::remove_var("INGRESS");
        std::env::remove_var("INGRESS_ADDRESS");
        std::env::remove_var("ROUND_TIMEOUT");
        std::env::remove_var("REBROADCAST_INTERVAL");
        std::env::remove_var("THRESHOLD");
    }
}

#[tokio::test]
async fn test_builder_validate_success() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors)
        .with_g1_map(g1_map);

    let result = builder.validate();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_builder_validate_no_contributors() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();

    let builder = OrchestratorBuilder::new(clock.clone(), signer);

    let result = builder.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "No contributors provided");
}

#[tokio::test]
async fn test_builder_validate_no_g1_map() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, _) = contributor::create_test_contributors();

    let builder = OrchestratorBuilder::new(clock.clone(), signer).with_contributors(contributors);

    let result = builder.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "No G1 public key mapping provided"
    );
}

#[tokio::test]
async fn test_builder_get_config() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors.clone())
        .with_g1_map(g1_map.clone())
        .with_threshold(5)
        .with_round_timeout(Duration::from_secs(60))
        .with_rebroadcast_interval(Duration::from_secs(15));

    let config = builder.get_config().expect("Failed to get config");

    assert_eq!(config.contributors.len(), 3);
    assert_eq!(config.g1_map.len(), 3);
    assert_eq!(config.config.threshold, 5);
    assert_eq!(config.config.round_timeout, Duration::from_secs(60));
    assert_eq!(config.config.rebroadcast_interval, Duration::from_secs(15));
}

#[tokio::test]
async fn test_builder_build() {
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

    // Verify the orchestrator was created successfully
    // We can't access private fields, but we can verify it was built
    let metadata = orchestrator.task_creator().get_task_metadata();
    assert!(!metadata.var1.is_empty());
    assert!(!metadata.var2.is_empty());
    assert!(!metadata.var3.is_empty());
}

#[tokio::test]
async fn test_builder_config_struct() {
    let clock = MockClock::new();
    let signer = signer::create_test_signer();
    let (contributors, g1_map) = contributor::create_test_contributors();

    let builder = OrchestratorBuilder::new(clock.clone(), signer)
        .with_contributors(contributors.clone())
        .with_g1_map(g1_map.clone())
        .with_threshold(4);

    let config: OrchestratorBuilderConfig<_> = builder.get_config().expect("Failed to get config");

    // Test that we can access all fields
    assert_eq!(config.contributors.len(), 3);
    assert_eq!(config.g1_map.len(), 3);
    assert_eq!(config.config.threshold, 4);
    assert_eq!(config.contributors, contributors);
    assert_eq!(config.g1_map, g1_map);
}
