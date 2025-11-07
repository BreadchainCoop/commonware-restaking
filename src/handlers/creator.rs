use crate::handlers::{TaskCreator};
use crate::solana_helpers::{get_client_and_test_bls_ncn, get_consensus_count, get_raw_message_and_hash};
use anyhow::{Result, anyhow};
use jito_bls_ncn_clients::jito_clients::{JitoClient};
use solana_pubkey::Pubkey;

pub struct Creator {
    client: JitoClient,
    ncn: Pubkey,
}

impl Creator {
    pub fn new(client: JitoClient, ncn: Pubkey) -> Self {
        Self { client, ncn }
    }

    pub async fn get_current_number(&self) -> Result<u64> {
        get_consensus_count(&self.client, &self.ncn).await
    }

    async fn get_payload_and_round(&self) -> Result<(Vec<u8>, u64)> {
        let current_number = self.get_current_number().await?;

        // For non-ingress mode, encode default variables into the payload
        let (raw_message, _) = get_raw_message_and_hash(current_number)?;
        let mut payload = raw_message.to_vec();
        let default_vars = ["default_var1", "default_var2", "default_var3"];
        for var in default_vars {
            payload.extend_from_slice(var.as_bytes());
            payload.push(0); // null terminator
        }

        Ok((payload, current_number))
    }
}

impl TaskCreator for Creator {
    async fn get_payload_and_round(&self) -> Result<(Vec<u8>, u64)> {
        self.get_payload_and_round()
            .await
            .map_err(|e| anyhow!("Creator error: {}", e))
    }
}

// Helper function to create a new Creator instance
pub async fn create_creator() -> Result<Creator> {
    let (client, test_bls_ncn) = get_client_and_test_bls_ncn()?;

    Ok(Creator::new(client, test_bls_ncn.solana_ncn))
}
