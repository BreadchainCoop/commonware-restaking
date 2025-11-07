#![allow(dead_code)]
use crate::solana_helpers::{get_client_and_test_bls_ncn, get_consensus_count, get_raw_message_and_hash};
use crate::wire;
use anyhow::Result;
use commonware_codec::{DecodeExt, ReadExt};
use jito_bls_ncn_clients::jito_clients::JitoClient;
use solana_pubkey::Pubkey;
use std::io::Cursor;

pub struct Validator {
    client: JitoClient,
    ncn: Pubkey,
}

impl Validator {
    pub async fn new() -> Result<Self> {
        let (client, test_ncn_config) = get_client_and_test_bls_ncn()?;

        Ok(Self { client, ncn: test_ncn_config.solana_ncn })
    }

    pub async fn validate_and_return_expected_hash(&self, msg: &[u8]) -> Result<Vec<u8>> {
        // First verify the message round
        self.verify_message_round(msg).await?;

        // Then get the payload hash
        self.get_payload_from_message(msg).await
    }

    pub async fn get_payload_from_message(&self, msg: &[u8]) -> Result<Vec<u8>> {
        // Decode the wire message
        let aggregation = wire::Aggregation::decode(msg)?;

        let (raw_message, _) = get_raw_message_and_hash(aggregation.round)?;

        Ok(raw_message.to_vec())
    }

    async fn verify_message_round(&self, msg: &[u8]) -> Result<()> {
        let aggregation = wire::Aggregation::read(&mut Cursor::new(msg))?;

        let consensus_count = get_consensus_count(&self.client, &self.ncn).await?;

        if aggregation.round != consensus_count {
            return Err(anyhow::anyhow!(
                "Invalid round number in message. Expected {}, got {}",
                consensus_count,
                aggregation.round
            ));
        }

        Ok(())
    }
}
