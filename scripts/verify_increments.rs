use alloy::providers::ProviderBuilder;
use commonware_avs_router::bindings::counter::Counter;
use commonware_eigenlayer::config::AvsDeployment;
use std::{env, time::Duration};
use tokio::time::sleep;

const DEFAULT_HTTP_RPC: &str = "http://localhost:8545";
const DEFAULT_AVS_DEPLOYMENT_PATH: &str = "../eigenlayer-bls-local/.nodes/avs_deploy.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Load configuration - try different possible paths
    let deployment_path =
        env::var("AVS_DEPLOYMENT_PATH").unwrap_or_else(|_| DEFAULT_AVS_DEPLOYMENT_PATH.to_string());

    println!("Trying to load deployment from: {}", deployment_path);

    // Check if file exists
    if !std::path::Path::new(&deployment_path).exists() {
        return Err(format!("Deployment file not found at: {}", deployment_path).into());
    }

    // Try different loading methods based on what's available
    let deployment = if let Ok(deployment) = AvsDeployment::load() {
        deployment
    } else {
        // If load() doesn't work, we might need to set the environment variable
        std::env::set_var("AVS_DEPLOYMENT_PATH", &deployment_path);
        AvsDeployment::load().map_err(|e| format!("Failed to load deployment: {}", e))?
    };
    let counter_address = deployment
        .counter_address()
        .map_err(|e| format!("Failed to get counter address: {}", e))?;
    let http_rpc = env::var("HTTP_RPC").unwrap_or_else(|_| DEFAULT_HTTP_RPC.to_string());

    println!("Connecting to RPC: {}", http_rpc);
    println!("Counter address: {}", counter_address);

    // Create provider and counter instance
    let url = url::Url::parse(&http_rpc).map_err(|e| format!("Invalid RPC URL: {}", e))?;
    let provider = ProviderBuilder::new().connect_http(url);
    let counter = Counter::new(counter_address, provider);

    // Get initial counter value
    let initial_count = counter
        .number()
        .call()
        .await
        .map_err(|e| format!("Failed to get initial counter: {}", e))?
        .to::<u64>();
    println!("Initial counter value: {}", initial_count);

    let target_increments = 2;
    let max_wait_time = Duration::from_secs(150); // 2.5 minutes max wait
    let check_interval = Duration::from_secs(10); // Check every 10 seconds

    let start_time = std::time::Instant::now();

    loop {
        // Check current counter value
        let current_count = counter
            .number()
            .call()
            .await
            .map_err(|e| format!("Failed to get current counter: {}", e))?
            .to::<u64>();
        let increments = current_count.saturating_sub(initial_count);

        println!(
            "Current counter: {}, Increments since start: {}, Elapsed: {:.1}s",
            current_count,
            increments,
            start_time.elapsed().as_secs_f64()
        );

        if increments >= target_increments {
            println!(
                "✅ SUCCESS: Counter was incremented {} times (target: {})",
                increments, target_increments
            );
            println!(
                "Total time elapsed: {:.1} seconds",
                start_time.elapsed().as_secs_f64()
            );
            return Ok(());
        }

        if start_time.elapsed() >= max_wait_time {
            println!(
                "❌ TIMEOUT: Only {} increments after {:.1} seconds (target: {})",
                increments,
                max_wait_time.as_secs_f64(),
                target_increments
            );
            return Err("Timeout waiting for required increments".into());
        }

        sleep(check_interval).await;
    }
}
