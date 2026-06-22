use commonware_avs_core::bn254::{Bn254, G1PublicKey, PublicKey};
use commonware_avs_core::validator::ValidatorTrait;
use commonware_runtime::{Clock, Metrics};
use std::collections::HashMap;
use std::time::Duration;
use tracing::info;

use crate::executor::{FromBlsAggregation, VerificationData, VerificationExecutor};

/// Fallible orchestrator construction, parameterized by the verification-data container
/// `VD` the executor consumes.
type OrchestratorBuildResult<TC, E, V, C, VD> =
    Result<crate::orchestrator::types::Orchestrator<TC, E, V, C, VD>, Box<dyn std::error::Error>>;

/// Configuration bundle returned by the orchestrator builder
#[allow(dead_code)]
pub struct OrchestratorBuilderConfig<C: Clock + Metrics> {
    pub runtime: C,
    pub signer: Bn254,
    pub contributors: Vec<PublicKey>,
    pub g1_map: HashMap<PublicKey, G1PublicKey>,
    pub config: OrchestratorConfig,
}

/// Configuration for orchestrator construction.
///
/// This struct holds all the configuration parameters needed
/// to construct an orchestrator instance.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Max duration to wait for operator signatures before abandoning a round
    pub aggregation_timeout: Duration,
    /// The threshold number of signatures required for aggregation
    pub threshold: usize,
    /// Whether to use ingress mode (HTTP server for external requests)
    pub use_ingress: bool,
    /// The HTTP server address (only used if use_ingress is true)
    pub ingress_address: String,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            aggregation_timeout: Duration::from_secs(30),
            threshold: 3,
            use_ingress: false,
            ingress_address: "0.0.0.0:8080".to_string(),
        }
    }
}

/// Builder for constructing orchestrator instances.
///
/// This builder provides a fluent interface for constructing
/// orchestrator instances with proper dependency injection.
/// It handles the complex construction logic including environment
/// variable configuration and dependency creation.
pub struct OrchestratorBuilder<C: Clock + Metrics> {
    runtime: C,
    signer: Bn254,
    contributors: Vec<PublicKey>,
    g1_map: HashMap<PublicKey, G1PublicKey>,
    config: OrchestratorConfig,
}

impl<C: Clock + Metrics> OrchestratorBuilder<C> {
    /// Creates a new OrchestratorBuilder with the given runtime and signer.
    ///
    /// # Arguments
    /// * `runtime` - The clock implementation for timing operations
    /// * `signer` - The BLS signer for cryptographic operations
    ///
    /// # Returns
    /// * `Self` - The new OrchestratorBuilder instance
    pub fn new(runtime: C, signer: Bn254) -> Self {
        Self {
            runtime,
            signer,
            contributors: Vec::new(),
            g1_map: HashMap::new(),
            config: OrchestratorConfig::default(),
        }
    }

    /// Sets the contributors for the orchestrator.
    ///
    /// # Arguments
    /// * `contributors` - List of contributor public keys
    ///
    /// # Returns
    /// * `Self` - The builder for method chaining
    pub fn with_contributors(mut self, contributors: Vec<PublicKey>) -> Self {
        self.contributors = contributors;
        self
    }

    /// Sets the G1 public key mapping for the orchestrator.
    ///
    /// # Arguments
    /// * `g1_map` - Mapping from G2 public keys to G1 public keys
    ///
    /// # Returns
    /// * `Self` - The builder for method chaining
    pub fn with_g1_map(mut self, g1_map: HashMap<PublicKey, G1PublicKey>) -> Self {
        self.g1_map = g1_map;
        self
    }

    /// Sets the aggregation timeout.
    ///
    /// # Arguments
    /// * `timeout` - Max duration to wait for operator signatures before abandoning a round
    ///
    /// # Returns
    /// * `Self` - The builder for method chaining
    #[allow(dead_code)]
    pub fn with_aggregation_timeout(mut self, timeout: Duration) -> Self {
        self.config.aggregation_timeout = timeout;
        self
    }

    /// Sets the threshold for signature aggregation.
    ///
    /// # Arguments
    /// * `threshold` - The number of signatures required for aggregation
    ///
    /// # Returns
    /// * `Self` - The builder for method chaining
    pub fn with_threshold(mut self, threshold: usize) -> Self {
        self.config.threshold = threshold;
        self
    }

    /// Enables ingress mode with the specified address.
    ///
    /// # Arguments
    /// * `address` - The HTTP server address (e.g., "0.0.0.0:8080")
    ///
    /// # Returns
    /// * `Self` - The builder for method chaining
    #[allow(dead_code)]
    pub fn with_ingress(mut self, address: String) -> Self {
        self.config.use_ingress = true;
        self.config.ingress_address = address;
        self
    }

    /// Configures the builder from environment variables.
    ///
    /// This method reads configuration from environment variables:
    /// - `INGRESS`: If set to "true", enables ingress mode
    /// - `INGRESS_ADDRESS`: The HTTP server address (defaults to "0.0.0.0:8080")
    /// - `AGGREGATION_TIMEOUT`: The aggregation timeout in seconds (defaults to 30)
    /// - `THRESHOLD`: The signature threshold (defaults to 3)
    ///
    /// # Returns
    /// * `Self` - The builder for method chaining
    pub fn load_from_env(mut self) -> Self {
        // Check for ingress configuration
        if let Ok(ingress) = std::env::var("INGRESS")
            && ingress.to_lowercase() == "true"
        {
            self.config.use_ingress = true;
            info!("Ingress mode enabled from environment");
        }

        // Check for ingress address
        if let Ok(address) = std::env::var("INGRESS_ADDRESS") {
            self.config.ingress_address = address;
        }

        // Check for aggregation timeout (supports fractional seconds)
        if let Ok(timeout) = std::env::var("AGGREGATION_TIMEOUT")
            && let Ok(seconds) = timeout.parse::<f64>()
        {
            self.config.aggregation_timeout = Duration::from_secs_f64(seconds);
            info!(
                "Aggregation timeout set to {} seconds from environment",
                seconds
            );
        }

        // Check for threshold
        if let Ok(threshold) = std::env::var("THRESHOLD")
            && let Ok(t) = threshold.parse::<usize>()
        {
            self.config.threshold = t;
            info!("Threshold set to {} from environment", t);
        }

        self
    }

    /// Validates the builder configuration.
    ///
    /// This method checks that all required fields are provided.
    ///
    /// # Returns
    /// * `Result<()>` - Ok if valid, Err with description if invalid
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.contributors.is_empty() {
            return Err("No contributors provided".into());
        }

        if self.g1_map.is_empty() {
            return Err("No G1 public key mapping provided".into());
        }

        Ok(())
    }

    /// Gets the configuration for building orchestrators.
    ///
    /// This method validates and returns the builder configuration so that concrete
    /// implementations can use it to build their specific orchestrator types.
    ///
    /// # Returns
    /// * `Result<OrchestratorBuilderConfig<C>>` - The configuration bundle if valid
    #[allow(dead_code)]
    pub fn get_config(self) -> Result<OrchestratorBuilderConfig<C>, Box<dyn std::error::Error>> {
        self.validate()?;

        info!(
            contributors = self.contributors.len(),
            threshold = self.config.threshold,
            aggregation_timeout = ?self.config.aggregation_timeout,
            use_ingress = self.config.use_ingress,
            "Validated orchestrator configuration"
        );

        Ok(OrchestratorBuilderConfig {
            runtime: self.runtime,
            signer: self.signer,
            contributors: self.contributors,
            g1_map: self.g1_map,
            config: self.config,
        })
    }

    /// Builds a generic orchestrator instance.
    ///
    /// This method creates a generic orchestrator with the specified
    /// trait implementations. This is useful for testing or when
    /// you need custom implementations.
    ///
    /// # Type Parameters
    /// * `TC` - Task creator implementation
    /// * `E` - Executor implementation
    /// * `V` - Validator implementation
    ///
    /// # Arguments
    /// * `task_creator` - The task creator implementation
    /// * `executor` - The executor implementation
    /// * `validator` - The validator implementation
    ///
    /// # Returns
    /// * `Result<Orchestrator<TC, E, V, C>>` - The constructed orchestrator instance
    pub fn build<TC, E, V>(
        self,
        task_creator: TC,
        executor: E,
        validator: V,
    ) -> OrchestratorBuildResult<TC, E, V, C, VerificationData>
    where
        TC: crate::creator::Creator + Send + Sync,
        E: VerificationExecutor<TC::TaskData, VerificationData> + Send + Sync,
        V: ValidatorTrait + Send + Sync,
    {
        self.build_with::<TC, E, V, VerificationData>(task_creator, executor, validator)
    }

    /// Builds an orchestrator whose executor consumes the verification-data container `VD`.
    pub fn build_with<TC, E, V, VD>(
        self,
        task_creator: TC,
        executor: E,
        validator: V,
    ) -> OrchestratorBuildResult<TC, E, V, C, VD>
    where
        TC: crate::creator::Creator + Send + Sync,
        E: VerificationExecutor<TC::TaskData, VD> + Send + Sync,
        V: ValidatorTrait + Send + Sync,
        VD: FromBlsAggregation + Send + Sync,
    {
        self.validate()?;

        info!(
            contributors = self.contributors.len(),
            threshold = self.config.threshold,
            aggregation_timeout = ?self.config.aggregation_timeout,
            "Building generic orchestrator"
        );

        let config = crate::orchestrator::types::OrchestratorConfig {
            aggregation_timeout: self.config.aggregation_timeout,
            contributors: self.contributors,
            g1_map: self.g1_map,
            threshold: self.config.threshold,
        };

        Ok(crate::orchestrator::types::Orchestrator::new(
            self.runtime,
            self.signer,
            config,
            task_creator,
            executor,
            validator,
        ))
    }
}
