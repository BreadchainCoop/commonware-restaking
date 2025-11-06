use crate::executor::bls::BlsEigenlayerExecutor;
use crate::ingress::start_http_server;
use crate::usecases::counter::{
    CounterCreator, CounterCreatorType, CounterHandler, CounterProvider, CreatorConfig,
    ListeningCounterCreator, SimpleTaskQueue,
};
use alloy_provider::ProviderBuilder;
use alloy_signer_local::PrivateKeySigner;
use anyhow::Result;
use commonware_avs_shared::bindings::blsapkregistry::BLSApkRegistry;
use commonware_avs_shared::bindings::blssigcheckoperatorstateretriever::BLSSigCheckOperatorStateRetriever;
use commonware_avs_shared::bindings::counter::Counter;
use commonware_eigenlayer::config::AvsDeployment;
use std::{env, str::FromStr, sync::Arc};

/// Factory function to create a default creator
pub async fn create_creator() -> anyhow::Result<CounterCreatorType> {
    let http_rpc = env::var("HTTP_RPC").expect("HTTP_RPC must be set");
    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
    let signer = PrivateKeySigner::from_str(&private_key)
        .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;
    let provider = ProviderBuilder::new()
        .wallet(signer)
        .connect(&http_rpc)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect provider: {}", e))?;

    let deployment =
        AvsDeployment::load().map_err(|e| anyhow::anyhow!("Failed to load deployment: {}", e))?;
    let counter_address = deployment
        .counter_address()
        .map_err(|e| anyhow::anyhow!("Failed to get counter address: {}", e))?;

    let provider = CounterProvider::new(counter_address, provider.clone());
    let creator = CounterCreator::new(provider);
    Ok(CounterCreatorType::Basic(creator))
}

/// Factory function to create a listening creator with HTTP server
pub async fn create_listening_creator_with_server(
    addr: String,
) -> anyhow::Result<CounterCreatorType> {
    let http_rpc = env::var("HTTP_RPC").expect("HTTP_RPC must be set");
    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
    let signer = PrivateKeySigner::from_str(&private_key)?;
    let provider = ProviderBuilder::new()
        .wallet(signer)
        .connect(&http_rpc)
        .await?;
    let deployment =
        AvsDeployment::load().map_err(|e| anyhow::anyhow!("Failed to load deployment: {}", e))?;
    let counter_address = deployment
        .counter_address()
        .map_err(|e| anyhow::anyhow!("Failed to get counter address: {}", e))?;
    let provider = CounterProvider::new(counter_address, provider.clone());
    // Use configurable timeout for ingress mode, defaulting to 30 seconds
    let timeout_ms: u64 = env::var("INGRESS_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30_000);
    let config = CreatorConfig {
        polling_interval_ms: 100,
        timeout_ms,
    };
    let queue = SimpleTaskQueue::new();
    // Clone the queue for the creator - both share the same underlying storage
    let creator = ListeningCounterCreator::new(provider, queue.clone(), config);
    // Wrap the queue in Arc for the HTTP server
    let queue_for_server = Arc::new(queue);
    tokio::spawn(async move {
        start_http_server(queue_for_server, &addr).await;
    });
    Ok(CounterCreatorType::Listening(creator))
}

/// Creates a new BlsEigenlayerExecutor configured for Counter operations
pub async fn create_counter_executor() -> Result<BlsEigenlayerExecutor<CounterHandler>> {
    let http_rpc = env::var("HTTP_RPC").expect("HTTP_RPC must be set");
    let view_only_provider = ProviderBuilder::new().on_http(url::Url::parse(&http_rpc).unwrap());

    let deployment =
        AvsDeployment::load().map_err(|e| anyhow::anyhow!("Failed to load deployment: {}", e))?;
    let bls_apk_registry_address = deployment
        .bls_apk_registry_address()
        .map_err(|e| anyhow::anyhow!("Failed to get BLS APK registry address: {}", e))?;
    let registry_coordinator_address = deployment
        .registry_coordinator_address()
        .map_err(|e| anyhow::anyhow!("Failed to get registry coordinator address: {}", e))?;
    let counter_address = deployment
        .counter_address()
        .map_err(|e| anyhow::anyhow!("Failed to get counter address: {}", e))?;

    let ecdsa_signer =
        PrivateKeySigner::from_str(&env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set"))
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;
    let bls_operator_state_retriever_address = deployment
        .bls_sig_check_operator_state_retriever_address()
        .map_err(|e| {
            anyhow::anyhow!("Failed to get BLS operator state retriever address: {}", e)
        })?;

    let write_provider = ProviderBuilder::new()
        .wallet(ecdsa_signer)
        .connect(&http_rpc)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect write provider: {}", e))?;

    let bls_apk_registry =
        BLSApkRegistry::new(bls_apk_registry_address, view_only_provider.clone());
    let bls_operator_state_retriever = BLSSigCheckOperatorStateRetriever::new(
        bls_operator_state_retriever_address,
        view_only_provider.clone(),
    );
    let counter = Counter::new(counter_address, write_provider.clone());

    // Create the counter handler
    let counter_handler = CounterHandler::new(counter);

    // Create and return the contract executor
    Ok(BlsEigenlayerExecutor::new(
        view_only_provider,
        bls_apk_registry,
        bls_operator_state_retriever,
        registry_coordinator_address,
        counter_handler,
    ))
}
