use crate::orchestrator::OrchestratorBuilder;
use crate::usecases::counter::factories::{
    create_counter_executor, create_creator, create_listening_creator_with_server,
};
use crate::usecases::counter::{CounterCreatorType, CounterOrchestrator, CounterValidator};

use commonware_runtime::Clock;
use tracing::info;

/// Builder extension for creating counter orchestrators.
///
/// This provides counter-specific building functionality while
/// maintaining the separation between generic and concrete implementations.
pub struct CounterOrchestratorBuilder;

impl CounterOrchestratorBuilder {
    /// Builds a CounterOrchestrator using the configured builder.
    ///
    /// This method takes a configured OrchestratorBuilder and uses it
    /// to create a CounterOrchestrator with all the counter-specific
    /// dependencies.
    ///
    /// # Arguments
    /// * `builder` - The configured orchestrator builder
    ///
    /// # Returns
    /// * `Result<CounterOrchestrator<C>>` - The constructed counter orchestrator
    pub async fn build<C: Clock>(
        builder: OrchestratorBuilder<C>,
    ) -> Result<CounterOrchestrator<C>, Box<dyn std::error::Error>> {
        // Create counter-specific dependencies
        let use_ingress = std::env::var("INGRESS").unwrap_or_default().to_lowercase() == "true";
        let task_creator: CounterCreatorType = if use_ingress {
            info!("Using creator with HTTP server on port 8080");
            create_listening_creator_with_server("0.0.0.0:8080".to_string()).await?
        } else {
            info!("Using Creator without ingress");
            create_creator().await?
        };

        let executor = create_counter_executor().await?;
        let validator = CounterValidator::new().await?;

        builder.build(task_creator, executor, validator)
    }
}
