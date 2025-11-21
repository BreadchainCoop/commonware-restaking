use alloy_primitives::Address;
use serde::Deserialize;
use std::{env, fs, str::FromStr};

#[derive(Debug, Deserialize)]
pub struct AvsDeployment {
    pub addresses: ContractAddresses,
}

#[derive(Debug, Deserialize)]
pub struct ContractAddresses {
    #[serde(rename = "registryCoordinator")]
    pub registry_coordinator: String,
    #[serde(rename = "blsapkRegistry")]
    pub bls_apk_registry: String,
    #[serde(rename = "blsSigCheck")]
    pub bls_sig_check_operator_state_retriever: String,
    pub counter: String,
}

impl AvsDeployment {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let deployment_path =
            env::var("AVS_DEPLOYMENT_PATH").expect("AVS_DEPLOYMENT_PATH must be set");
        let content = fs::read_to_string(deployment_path)?;
        let deployment: AvsDeployment = serde_json::from_str(&content)?;
        Ok(deployment)
    }

    pub fn registry_coordinator_address(
        &self,
    ) -> Result<Address, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Address::from_str(&self.addresses.registry_coordinator)?)
    }

    pub fn bls_apk_registry_address(
        &self,
    ) -> Result<Address, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Address::from_str(&self.addresses.bls_apk_registry)?)
    }

    pub fn bls_sig_check_operator_state_retriever_address(
        &self,
    ) -> Result<Address, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Address::from_str(
            &self.addresses.bls_sig_check_operator_state_retriever,
        )?)
    }

    pub fn counter_address(&self) -> Result<Address, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Address::from_str(&self.addresses.counter)?)
    }
}
