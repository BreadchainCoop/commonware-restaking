use anyhow::{Result, anyhow};
use jito_bls_ncn_clients::{jito_clients::{JitoClient}, program_clients::bls_ncn_client::{get_bls_operator, get_rolling_snapshot, vote}};
use jito_bls_ncn_core::bls::{solana_bls::offchain_prepare_vote_data, solana_bls_interface::{SolanaBN254G1, SolanaBN254G2, SolanaBN254PublicKey, SolanaBN254Signature}};
use solana_pubkey::Pubkey;
use std::{collections::HashMap};

use crate::solana_helpers::{get_client_and_test_bls_ncn, get_consensus_count};

pub struct Executor {
    client: JitoClient,
    ncn: Pubkey,
    operator_hash_map: HashMap<Pubkey, SolanaBN254PublicKey>,
}

impl Executor {
    pub async fn populate_all_operators(&mut self) -> Result<()> {
        let snapshot = get_rolling_snapshot(&self.client, &self.ncn).await?;

        for operator_entry in snapshot.operators.iter() {
            let operator_entry = match operator_entry.as_ref() {
                Some(entry) => *entry,
                None => continue,
            };

            let should_update = match self.operator_hash_map.get(&operator_entry.operator) {
                Some(cached) => cached.g1.raw != operator_entry.g1,
                None => true,
            };

            if should_update {
                let bls_operator_account = get_bls_operator(&self.client, &operator_entry.operator).await?;
                let g1 = SolanaBN254G1::new(&bls_operator_account.g1).map_err(|e| anyhow!("Could not derive G1: {}", e))?;
                let g2 = SolanaBN254G2::new(&bls_operator_account.g2).map_err(|e| anyhow!("Could not derive G2: {}", e))?;
                let bls_pubkey = SolanaBN254PublicKey::new(g1, g2);

                self.operator_hash_map.insert(operator_entry.operator, bls_pubkey);
            }
        }
        Ok(())
    }

    pub async fn get_signing_indices(&self, signed_operators: &[SolanaBN254PublicKey]) -> Result<Vec<usize>> {
        let snapshot = get_rolling_snapshot(&self.client, &self.ncn).await?;

        let mut signing_indices = Vec::new();
        for (index, operator_entry) in snapshot.operators.iter().enumerate() {
            let operator_entry = match operator_entry.as_ref() {
                Some(entry) => *entry,
                None => continue,
            };

            if signed_operators.iter().any(|operator| operator.g1.raw.eq(&operator_entry.g1)) {
                signing_indices.push(index);
            }
        }

        Ok(signing_indices)
    }

    pub async fn get_current_number(&self) -> Result<u64> {
        get_consensus_count(&self.client, &self.ncn).await
    }

    pub async fn execute_verification(
        &mut self,
        raw_message: &[u8; 32],
        signed_operators: &[SolanaBN254PublicKey],
        signatures: &[SolanaBN254Signature],
    ) -> Result<()> {
        self.populate_all_operators().await?;
        let current_number = self.get_current_number().await?;

        let g1_signatures = signatures.iter().map(|sig| sig.raw).collect::<Vec<_>>();
        let g2_signed_pubkeys = signed_operators.iter().map(|op| op.g2.raw).collect::<Vec<_>>();
        let total_operators = self.operator_hash_map.len();

        let signing_indices = self.get_signing_indices(signed_operators).await?;

        let (aggregated_g1_signature, aggregated_g2_signed, operators_bitmap_signed) = offchain_prepare_vote_data(
            &g1_signatures,
            &g2_signed_pubkeys,
            &signing_indices,
            total_operators
        ).map_err(|e| anyhow!("Could not prepare vote data: {}", e))?;

        let aggregated_g1_signature = SolanaBN254G1::new(&aggregated_g1_signature).map_err(|e| anyhow!("Could not derive G1: {}", e))?;
        let aggregated_g2_signed = SolanaBN254G2::new(&aggregated_g2_signed).map_err(|e| anyhow!("Could not derive G2: {}", e))?;

        vote(
            &mut self.client,
            &self.ncn,
            &aggregated_g1_signature,
            &aggregated_g2_signed,
            &operators_bitmap_signed,
            raw_message,
            current_number,
        ).await?;

        Ok(())
    }
}

pub async fn create_executor() -> Result<Executor> {
    let (client, test_bls_ncn) = get_client_and_test_bls_ncn()?;

    Ok(Executor {
        client,
        ncn: test_bls_ncn.solana_ncn,
        operator_hash_map: HashMap::new(),
    })
}
