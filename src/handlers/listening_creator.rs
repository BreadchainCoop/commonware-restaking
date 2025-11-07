use anyhow::{Result, anyhow};
use std::sync::Arc;
use jito_bls_ncn_clients::jito_clients::JitoClient;
use solana_pubkey::Pubkey;
use tokio::sync::Mutex;

use crate::handlers::{TaskCreator};
use crate::ingress::{TaskRequest, start_http_server};
use crate::solana_helpers::{get_client_and_test_bls_ncn, get_consensus_count, get_raw_message_and_hash};

pub struct ListeningCreator {
    client: JitoClient,
    ncn: Pubkey,
    queue: Arc<Mutex<Vec<TaskRequest>>>,
}

impl ListeningCreator {
    pub fn new(client: JitoClient, ncn: Pubkey) -> Self {
        Self {
            client,
            ncn,
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn get_current_number(&self) -> Result<u64> {
        get_consensus_count(&self.client, &self.ncn).await
    }

    // Pulls the next task from the queue, or returns None if empty
    pub async fn get_next_task(&self) -> Option<TaskRequest> {
        let mut queue = self.queue.lock().await;
        if !queue.is_empty() {
            Some(queue.remove(0))
        } else {
            None
        }
    }

    // Single entry point that can be called by the orchestrator
    // This is where queue requests would be pulled from
    pub async fn get_payload_and_round(&self) -> Result<(Vec<u8>, u64)> {
        // Wait for a task to be available
        let task = loop {
            if let Some(task) = self.get_next_task().await {
                break task;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        };
        let current_number = self.get_current_number().await?;
        let mut payload = self.get_payload_for_round(current_number)?.0;

        // Encode the three variables into the payload
        payload.extend_from_slice(task.body.var1.as_bytes());
        payload.push(0); // null terminator
        payload.extend_from_slice(task.body.var2.as_bytes());
        payload.push(0); // null terminator
        payload.extend_from_slice(task.body.var3.as_bytes());
        payload.push(0); // null terminator

        Ok((payload, current_number))
    }

    // Optional: Method to get payload for a specific round number
    pub fn get_payload_for_round(&self, round_number: u64) -> Result<(Vec<u8>, u64)> {
        let (raw_message, _) = get_raw_message_and_hash(round_number)?;
        Ok((raw_message.to_vec(), round_number))
    }

    // Start the HTTP server in a background task
    pub async fn start_http_server(self: Arc<Self>, addr: String) {
        let queue = self.queue.clone();
        tokio::spawn(async move {
            start_http_server(queue, &addr).await;
        });
    }
}

impl TaskCreator for ListeningCreator {
    async fn get_payload_and_round(&self) -> Result<(Vec<u8>, u64)> {
        self.get_payload_and_round()
            .await
            .map_err(|e| anyhow!("ListeningCreator error: {}", e))
    }
}

// Helper function to create a new ListeningCreator instance and start HTTP server
pub async fn create_listening_creator_with_server(
    addr: String,
) -> Result<Arc<ListeningCreator>> {
    let (client, test_bls_ncn) = get_client_and_test_bls_ncn()?;

    let creator = Arc::new(ListeningCreator::new(client, test_bls_ncn.solana_ncn));
    let server_creator = creator.clone();

    tokio::spawn(async move {
        server_creator.start_http_server(addr).await;
    });

    Ok(creator)
}
