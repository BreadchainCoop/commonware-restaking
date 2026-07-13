//! Construction of the router's environment-configured components: alloy
//! providers, the polling task source, and the on-chain submitter.

use crate::executor::CounterHandler;
use crate::provider::CounterProvider;
use crate::source::CounterTaskSource;
use alloy_primitives::Bytes;
use alloy_provider::ProviderBuilder;
use alloy_signer_local::PrivateKeySigner;
use anyhow::Result;
use commonware_avs_bindings::bls_apk_registry::BLSApkRegistry;
use commonware_avs_bindings::bls_sig_check_operator_state_retriever::BLSSigCheckOperatorStateRetriever;
use commonware_avs_core::bn254::Bn254Scheme;
use commonware_avs_router::reporter::CertifiedReceiver;
use commonware_avs_router::sequencer::{ResolutionSender, SharedAssignments};
use commonware_avs_router::submitter::Submitter;
use counter_common::config::CounterDeployment;
use counter_common::types::CounterTaskData;
use std::{env, str::FromStr, time::Duration};

/// Quorum number this example operates on (quorum 0 only).
pub const QUORUM_NUMBERS: &[u8] = &[0x00];

/// Creates the router's polling task source: reads `Counter.number()` every
/// `POLLING_INTERVAL_MS` (default 2000) and hands the current round to the
/// sequencer.
pub async fn create_task_source() -> Result<CounterTaskSource> {
    let http_rpc = env::var("HTTP_RPC").expect("HTTP_RPC must be set");
    let provider = ProviderBuilder::new().connect_http(
        url::Url::parse(&http_rpc)
            .map_err(|e| anyhow::anyhow!("Failed to parse RPC URL '{}': {}", http_rpc, e))?,
    );

    let deployment = CounterDeployment::load()
        .map_err(|e| anyhow::anyhow!("Failed to load deployment: {}", e))?;
    let counter_address = deployment
        .counter_address()
        .map_err(|e| anyhow::anyhow!("Failed to get counter address: {}", e))?;

    let polling_interval_ms: u64 = env::var("POLLING_INTERVAL_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2_000);

    let provider = CounterProvider::new(counter_address, provider);
    Ok(CounterTaskSource::new(
        provider,
        Duration::from_millis(polling_interval_ms),
    ))
}

/// Creates the [`Submitter`] wired to the deployed BLS registry contracts and the
/// `Counter` contract's `increment` entry point.
pub async fn create_submitter(
    scheme: Bn254Scheme,
    assignments: SharedAssignments<CounterTaskData>,
    certified: CertifiedReceiver<Bn254Scheme>,
    resolutions: ResolutionSender,
    namespace: Vec<u8>,
) -> Result<Submitter<CounterTaskData, CounterHandler>> {
    let http_rpc = env::var("HTTP_RPC").expect("HTTP_RPC must be set");
    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");

    let deployment = CounterDeployment::load()
        .map_err(|e| anyhow::anyhow!("Failed to load deployment: {}", e))?;

    let view_only_provider = ProviderBuilder::new().connect_http(
        url::Url::parse(&http_rpc)
            .map_err(|e| anyhow::anyhow!("Failed to parse RPC URL '{}': {}", http_rpc, e))?,
    );

    let bls_apk_registry_address = deployment
        .bls_apk_registry_address()
        .map_err(|e| anyhow::anyhow!("Failed to get BLS APK registry address: {}", e))?;
    let registry_coordinator_address = deployment
        .registry_coordinator_address()
        .map_err(|e| anyhow::anyhow!("Failed to get registry coordinator address: {}", e))?;
    let bls_operator_state_retriever_address = deployment
        .bls_sig_check_operator_state_retriever_address()
        .map_err(|e| {
            anyhow::anyhow!("Failed to get BLS operator state retriever address: {}", e)
        })?;
    let counter_address = deployment
        .counter_address()
        .map_err(|e| anyhow::anyhow!("Failed to get counter address: {}", e))?;

    let ecdsa_signer = PrivateKeySigner::from_str(&private_key)
        .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;
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

    let counter = counter_bindings::Counter::new(counter_address, write_provider);
    let handler = CounterHandler::new(counter);

    Ok(Submitter::new(
        scheme,
        view_only_provider,
        bls_apk_registry,
        bls_operator_state_retriever,
        registry_coordinator_address,
        handler,
        assignments,
        certified,
        resolutions,
        namespace,
        Bytes::from_static(QUORUM_NUMBERS),
    ))
}
