#![allow(dead_code)]
use std::{env, net::{IpAddr, Ipv4Addr, SocketAddr}};
use anyhow::{Result, anyhow};
use jito_bls_ncn_clients::{jito_clients::{rpc::JitoRpcClient, JitoClient}, program_clients::bls_ncn_client::{get_bls_operator, get_consensus, get_rolling_snapshot}};
use jito_bls_ncn_core::bls::{solana_bls::solana_hash_to_curve, solana_bls_interface::{SolanaBN254G1, SolanaBN254G2, SolanaBN254PublicKey, TestBlsNcn}};
use solana_pubkey::Pubkey;

pub fn get_client_and_test_bls_ncn() -> Result<(JitoClient, TestBlsNcn)> {
    let test_config_filepath = env::var("TEST_BLS_NCN_CONFIG_FILE_PATH").expect("TEST_BLS_NCN_CONFIG_FILE_PATH must be set");
    let test_ncn_config = TestBlsNcn::from_json_file(&test_config_filepath).map_err(|e| anyhow!("Could not parse test config file: {}", e))?;

    let client = JitoRpcClient::new_with_keypair(test_ncn_config.rpc_url.clone(), test_ncn_config.solana_test_keypair.insecure_clone());

    Ok((client, test_ncn_config))
}

pub async fn get_consensus_count(client: &JitoClient, ncn: &Pubkey) -> Result<u64> {
    let consensus = get_consensus(client, ncn).await?;

    Ok(consensus.consensus_count())
}

pub fn get_raw_message_and_hash(consensus_count: u64) -> Result<([u8; 32], [u8; 64])> {
    let mut raw_message: [u8; 32] = [0; 32];
    raw_message[0..8].copy_from_slice(&consensus_count.to_le_bytes());

    let hash = solana_hash_to_curve(&raw_message, consensus_count).map_err(|e| anyhow!("Could not hash to curve: {}", e))?;

    Ok((raw_message, hash))
}

#[derive(Clone, Debug)]
pub struct CommonwareOperator {
    pub solana_operator: Pubkey,
    pub bls_operator: SolanaBN254PublicKey,
    pub socket_address: Option<SocketAddr>,
}

pub async fn get_operator_states(
    client: &JitoClient,
    test_bls_ncn: &TestBlsNcn,
) -> Result<Vec<CommonwareOperator>> {
    let snapshot = get_rolling_snapshot(&client, &test_bls_ncn.solana_ncn).await?;

    let mut operators = Vec::new();
    for operator_entry in snapshot.operators.iter() {
        let operator_entry = match operator_entry.as_ref() {
            Some(entry) => *entry,
            None => continue,
        };

        let bls_operator_account = get_bls_operator(&client, &operator_entry.operator).await?;
        let g1 = SolanaBN254G1::new(&bls_operator_account.g1).map_err(|e| anyhow!("Could not derive G1: {}", e))?;
        let g2 = SolanaBN254G2::new(&bls_operator_account.g2).map_err(|e| anyhow!("Could not derive G2: {}", e))?;
        let bls_pubkey = SolanaBN254PublicKey::new(g1, g2);

        let index = test_bls_ncn.test_operators.iter().position(|operator| operator.bls_public_key.g2.eq(&g2)).ok_or(anyhow!("Could not find operator"))?;

        //TODO - You can read the socket from the Operator - but for testing purposes, we'll use a hardcoded port
        let port: u16 = 3001 + index as u16;
        let socket_address = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            port,);

        operators.push(CommonwareOperator {
            solana_operator: operator_entry.operator,
            bls_operator: bls_pubkey,
            socket_address: Some(socket_address),
        });
    }


    Ok(operators)
}
