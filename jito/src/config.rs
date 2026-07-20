//! NCN deployment configuration — the Jito analog of the EigenLayer backend's
//! `avs_deploy.json` (`commonware_avs_eigenlayer::AvsDeployment`).
//!
//! A deployment JSON pins everything the router/node need to talk to one NCN:
//!
//! ```json
//! {
//!   "rpcHttpUrl": "http://127.0.0.1:8899",
//!   "rpcWsUrl": "ws://127.0.0.1:8900",
//!   "commitment": "confirmed",
//!   "ncnProgramId": "...",
//!   "ncn": "...",
//!   "restakingProgramId": "...",
//!   "consensusThresholdBps": 6667,
//!   "computeUnitLimit": 1000000,
//!   "computeUnitPrice": 0
//! }
//! ```
//!
//! Loaded from the path in `NCN_DEPLOYMENT_PATH` (peer of
//! `AVS_DEPLOYMENT_PATH`).

use serde::Deserialize;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::{env, fs};

/// Default consensus threshold: 6667 bps (Config default per INTERFACES.md §2).
pub const DEFAULT_CONSENSUS_THRESHOLD_BPS: u64 = 6667;

/// Default compute-unit limit for a `VerifyCertificate` transaction: one
/// challenge-combined pairing (~250k CU) plus hash-to-curve retries and
/// account loading, with generous headroom.
pub const DEFAULT_COMPUTE_UNIT_LIMIT: u32 = 1_000_000;

/// Seed of the jito-restaking global config PDA (`jito_restaking_core`).
const RESTAKING_CONFIG_SEED: &[u8] = b"config";

/// Errors surfaced while loading or interpreting a deployment config.
pub type ConfigError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NcnDeployment {
    /// HTTP RPC endpoint for all reads and transaction submission.
    pub rpc_http_url: String,
    /// Optional websocket endpoint (subscriptions; unused by the base flow).
    #[serde(default)]
    pub rpc_ws_url: Option<String>,
    /// Read commitment level. `confirmed` minimum (INTERFACES.md §5);
    /// `processed` is rejected at load time — it is racy under forks.
    #[serde(default = "default_commitment")]
    pub commitment: String,
    /// The deployed NCN program id.
    pub ncn_program_id: String,
    /// The NCN account (jito-restaking `Ncn`), keying every PDA.
    pub ncn: String,
    /// The jito-restaking program id (for the `restaking_config` account of
    /// `VerifyCertificate`).
    pub restaking_program_id: String,
    /// Stake-weighted consensus threshold in bps.
    ///
    /// Phase 1 put `consensus_threshold_bps` on the NCN `Config` PDA
    /// (admin-settable, default 6667) and `get_quorum` reads THAT value — the
    /// one `VerifyCertificate` enforces. This field is only the fallback for
    /// the window where the Config PDA does not exist yet; a mismatch is
    /// logged and resolved in favor of the chain.
    #[serde(default = "default_threshold_bps")]
    pub consensus_threshold_bps: u64,
    /// Compute-unit limit for submitted transactions.
    #[serde(default = "default_compute_unit_limit")]
    pub compute_unit_limit: u32,
    /// Optional priority fee (micro-lamports per CU).
    #[serde(default)]
    pub compute_unit_price: Option<u64>,
}

fn default_commitment() -> String {
    "confirmed".to_string()
}

fn default_threshold_bps() -> u64 {
    DEFAULT_CONSENSUS_THRESHOLD_BPS
}

fn default_compute_unit_limit() -> u32 {
    DEFAULT_COMPUTE_UNIT_LIMIT
}

impl NcnDeployment {
    /// Loads the deployment JSON from the path in `NCN_DEPLOYMENT_PATH`.
    pub fn load() -> Result<Self, ConfigError> {
        let path =
            env::var("NCN_DEPLOYMENT_PATH").map_err(|_| "NCN_DEPLOYMENT_PATH must be set")?;
        Self::load_from(&path)
    }

    /// Loads and validates a deployment JSON from an explicit path.
    pub fn load_from(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parses and validates a deployment JSON document.
    pub fn parse(content: &str) -> Result<Self, ConfigError> {
        let deployment: NcnDeployment = serde_json::from_str(content)?;
        // Fail fast on malformed keys and unsafe settings.
        deployment.ncn_program_id()?;
        deployment.ncn()?;
        deployment.restaking_program_id()?;
        deployment.read_commitment()?;
        if deployment.consensus_threshold_bps == 0 || deployment.consensus_threshold_bps > 10_000 {
            return Err(format!(
                "consensusThresholdBps must be in (0, 10000], got {}",
                deployment.consensus_threshold_bps
            )
            .into());
        }
        Ok(deployment)
    }

    pub fn ncn_program_id(&self) -> Result<Pubkey, ConfigError> {
        Ok(Pubkey::from_str(&self.ncn_program_id)?)
    }

    pub fn ncn(&self) -> Result<Pubkey, ConfigError> {
        Ok(Pubkey::from_str(&self.ncn)?)
    }

    pub fn restaking_program_id(&self) -> Result<Pubkey, ConfigError> {
        Ok(Pubkey::from_str(&self.restaking_program_id)?)
    }

    /// The commitment used for every chain read while building instructions.
    /// `confirmed` minimum; `Resolution::Executed` is still only declared at
    /// `finalized` (submitter policy, independent of this knob).
    pub fn read_commitment(&self) -> Result<CommitmentConfig, ConfigError> {
        match self.commitment.as_str() {
            "confirmed" => Ok(CommitmentConfig::confirmed()),
            "finalized" => Ok(CommitmentConfig::finalized()),
            "processed" => Err(
                "commitment 'processed' is below the 'confirmed' minimum (racy under forks)".into(),
            ),
            other => Err(format!("unknown commitment level '{other}'").into()),
        }
    }

    /// The NCN program's `Config` PDA (`["config", ncn]`).
    pub fn ncn_config_pda(&self) -> Result<Pubkey, ConfigError> {
        let (pda, _, _) = ncn_program_core::config::Config::find_program_address(
            &self.ncn_program_id()?,
            &self.ncn()?,
        );
        Ok(pda)
    }

    /// The NCN program's `Snapshot` PDA (`["snapshot", ncn]`).
    pub fn snapshot_pda(&self) -> Result<Pubkey, ConfigError> {
        let (pda, _, _) = ncn_program_core::snapshot::Snapshot::find_program_address(
            &self.ncn_program_id()?,
            &self.ncn()?,
        );
        Ok(pda)
    }

    /// The jito-restaking global config PDA (`["config"]` under the restaking
    /// program) — the `restaking_config` account of `VerifyCertificate`.
    pub fn restaking_config_pda(&self) -> Result<Pubkey, ConfigError> {
        let (pda, _) =
            Pubkey::find_program_address(&[RESTAKING_CONFIG_SEED], &self.restaking_program_id()?);
        Ok(pda)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "rpcHttpUrl": "http://127.0.0.1:8899",
        "rpcWsUrl": "ws://127.0.0.1:8900",
        "ncnProgramId": "7Vbxgo1BpTpqJpjLVQqLPvUZm6EWMFc3nO111111111",
        "ncn": "BPFLoaderUpgradeab1e11111111111111111111111",
        "restakingProgramId": "RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q"
    }"#;

    #[test]
    fn parses_with_defaults() {
        // Pubkeys must be valid base58 of 32 bytes; use well-known program ids.
        let sample = SAMPLE.replace(
            "7Vbxgo1BpTpqJpjLVQqLPvUZm6EWMFc3nO111111111",
            "Vote111111111111111111111111111111111111111",
        );
        let deployment = NcnDeployment::parse(&sample).expect("sample parses");
        assert_eq!(deployment.commitment, "confirmed");
        assert_eq!(
            deployment.consensus_threshold_bps,
            DEFAULT_CONSENSUS_THRESHOLD_BPS
        );
        assert_eq!(deployment.compute_unit_limit, DEFAULT_COMPUTE_UNIT_LIMIT);
        assert!(deployment.compute_unit_price.is_none());
        deployment.ncn_config_pda().expect("config pda derives");
        deployment.snapshot_pda().expect("snapshot pda derives");
        deployment
            .restaking_config_pda()
            .expect("restaking config pda derives");
    }

    #[test]
    fn rejects_processed_commitment() {
        let sample = SAMPLE
            .replace(
                "7Vbxgo1BpTpqJpjLVQqLPvUZm6EWMFc3nO111111111",
                "Vote111111111111111111111111111111111111111",
            )
            .replace(
                "\"rpcHttpUrl\"",
                "\"commitment\": \"processed\", \"rpcHttpUrl\"",
            );
        assert!(NcnDeployment::parse(&sample).is_err());
    }

    #[test]
    fn rejects_out_of_range_threshold() {
        let sample = SAMPLE
            .replace(
                "7Vbxgo1BpTpqJpjLVQqLPvUZm6EWMFc3nO111111111",
                "Vote111111111111111111111111111111111111111",
            )
            .replace(
                "\"rpcHttpUrl\"",
                "\"consensusThresholdBps\": 10001, \"rpcHttpUrl\"",
            );
        assert!(NcnDeployment::parse(&sample).is_err());
    }

    #[test]
    fn snapshot_pda_matches_program_core_seeds() {
        let sample = SAMPLE.replace(
            "7Vbxgo1BpTpqJpjLVQqLPvUZm6EWMFc3nO111111111",
            "Vote111111111111111111111111111111111111111",
        );
        let deployment = NcnDeployment::parse(&sample).unwrap();
        let program_id = deployment.ncn_program_id().unwrap();
        let ncn = deployment.ncn().unwrap();
        let expected =
            Pubkey::find_program_address(&[b"snapshot", ncn.to_bytes().as_slice()], &program_id).0;
        assert_eq!(deployment.snapshot_pda().unwrap(), expected);
    }
}
