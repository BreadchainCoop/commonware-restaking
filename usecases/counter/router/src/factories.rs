use crate::{
    creator::{CounterCreator, CounterCreatorType, ListeningCounterCreator},
    executor::CounterHandler,
    provider::CounterProvider,
};
use alloy_primitives::Address;
use alloy_provider::ProviderBuilder;
use alloy_signer_local::PrivateKeySigner;
use anyhow::Result;
use commonware_avs_bindings::WalletProvider;
use commonware_avs_bindings::counter::Counter;
use commonware_avs_core::eigenlayer::config::AvsDeployment;
use commonware_avs_router::creator::{CreatorConfig, SimpleTaskQueue};
use commonware_avs_router::ingress::http_server::start_http_server;
use std::{env, str::FromStr};

pub async fn create_creator() -> Result<CounterCreatorType> {
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

pub async fn create_listening_creator_with_server(addr: String) -> Result<CounterCreatorType> {
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
    let timeout_ms: u64 = env::var("INGRESS_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30_000);
    let config = CreatorConfig {
        polling_interval_ms: 100,
        timeout_ms,
    };
    let queue = SimpleTaskQueue::new();
    let creator = ListeningCounterCreator::new(provider, queue.clone(), config);
    let queue_for_server = std::sync::Arc::new(queue);
    tokio::spawn(async move {
        start_http_server(queue_for_server, &addr).await;
    });
    Ok(CounterCreatorType::Listening(creator))
}

pub fn create_counter_handler(
    write_provider: WalletProvider,
    counter_address: Address,
) -> CounterHandler {
    let counter = Counter::new(counter_address, write_provider);
    CounterHandler::new(counter)
}
