use alloy_primitives::Address;
use commonware_avs_eigenlayer::AvsDeployment;

pub struct CounterDeployment {
    inner: AvsDeployment,
}

impl CounterDeployment {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let inner = AvsDeployment::load()?;
        Ok(Self { inner })
    }

    pub fn counter_address(&self) -> Result<Address, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.custom_address("counter")
    }

    pub fn registry_coordinator_address(
        &self,
    ) -> Result<Address, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.registry_coordinator_address()
    }

    pub fn bls_apk_registry_address(
        &self,
    ) -> Result<Address, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.bls_apk_registry_address()
    }

    pub fn bls_sig_check_operator_state_retriever_address(
        &self,
    ) -> Result<Address, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.bls_sig_check_operator_state_retriever_address()
    }
}
