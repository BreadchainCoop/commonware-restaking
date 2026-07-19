//! Jito NCN network client — the peer of
//! `commonware_avs_eigenlayer::EigenStakingClient`.
//!
//! Produces the quorum view the chassis wiring needs from ON-CHAIN state only:
//!
//! - operators via `getProgramAccounts` with memcmp filters on
//!   `NCNOperatorAccount` (discriminator at offset 0, the `ncn` field at
//!   offset 8), sockets from the accounts' `ip_address`/`port` fields;
//! - stake, APK material and snapshot freshness from the `Snapshot` PDA;
//! - every read at `confirmed` commitment minimum (INTERFACES.md §5).
//!
//! Account LAYOUTS come from `ncn-program-core` (`try_from_slice_unchecked`)
//! — imported, never copied.

use crate::bn254::{G1PublicKey, PublicKey};
use crate::config::NcnDeployment;
use crate::quorum::{QuorumReconciliation, reconcile};
use anyhow::{Context, Result, anyhow};
use commonware_utils::ordered::Set;
use jito_bytemuck::{AccountDeserialize, Discriminator};
use ncn_program_core::ncn_operator_account::NCNOperatorAccount;
use ncn_program_core::snapshot::Snapshot;
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::mem::size_of;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tracing::{info, warn};

/// One registered operator, joined across its `NCNOperatorAccount` (keys +
/// socket) and its `OperatorSnapshot` inside the `Snapshot` PDA (stake).
#[derive(Debug, Clone)]
pub struct JitoOperatorInfo {
    /// The operator's jito-restaking `Operator` account.
    pub operator: Pubkey,
    /// The operator's index in the NCN — the BIT position in the on-chain
    /// signer bitmap (NOT the chassis participant index).
    pub ncn_operator_index: u64,
    /// BLS G2 identity key (participant key of the chassis scheme).
    pub g2_pub_key: PublicKey,
    /// BLS G1 key (APK arithmetic).
    pub g1_pub_key: G1PublicKey,
    /// p2p socket from the on-chain ip/port registration; `None` when unset.
    pub socket: Option<SocketAddr>,
    /// Snapshotted stake weight (zero when not snapshotted yet).
    pub stake_weight: u128,
    /// Whether the snapshot marks the operator active with minimum stake.
    pub can_vote: bool,
}

/// The assembled quorum view: operators in PARTICIPANT ORDER (sorted by
/// compressed G2 bytes, first occurrence kept on duplicates) plus the
/// snapshot-level facts the submitter needs. Built by
/// [`JitoQuorum::from_parts`] so every consumer derives identical,
/// index-aligned vectors.
#[derive(Debug, Clone)]
pub struct JitoQuorum {
    /// Operators sorted into participant order.
    pub operators: Vec<JitoOperatorInfo>,
    /// Registered-operator count from the snapshot — the on-chain bitmap's
    /// bit-length domain.
    pub operators_registered: u64,
    /// Snapshot generation captured at assembly time.
    ///
    /// TODO-FREEZE(phase1-dmsg): `main`'s `Snapshot` has no `generation`
    /// field yet (Phase 1 adds it, bumped on register/remove/rotation). Until
    /// the git dep picks that up this is ALWAYS 0, matching what the Phase 1
    /// program will report for a never-mutated operator set; bump the
    /// `ncn-program-core` dependency and read `snapshot.generation()` here.
    pub generation: u64,
    /// Slot the snapshot was last refreshed at (staleness input).
    pub last_snapshot_slot: u64,
    /// Stake-weighted consensus threshold (bps) certificates must clear.
    pub threshold_bps: u64,
    /// Total snapshotted stake across all registered operators.
    pub total_stake: u128,
}

impl JitoQuorum {
    /// Sorts raw operator rows into participant order and attaches the
    /// snapshot facts. Deduplicates by G2 key (first occurrence wins — the
    /// same rule `Map::from_iter_dedup` applies in the wiring), so a
    /// duplicate key registered under two operator accounts cannot misalign
    /// participant indices across processes.
    pub fn from_parts(
        mut operators: Vec<JitoOperatorInfo>,
        operators_registered: u64,
        generation: u64,
        last_snapshot_slot: u64,
        threshold_bps: u64,
    ) -> Self {
        // Stable sort by the participant ordering key; stability preserves
        // on-chain discovery order among duplicates so "first occurrence"
        // is deterministic.
        operators.sort_by(|a, b| a.g2_pub_key.cmp(&b.g2_pub_key));
        operators.dedup_by(|next, kept| {
            if next.g2_pub_key == kept.g2_pub_key {
                warn!(
                    operator = %next.operator,
                    "duplicate G2 key registered by multiple operators; keeping first"
                );
                true
            } else {
                false
            }
        });
        let total_stake = operators
            .iter()
            .fold(0u128, |acc, op| acc.saturating_add(op.stake_weight));
        Self {
            operators,
            operators_registered,
            generation,
            last_snapshot_slot,
            threshold_bps,
            total_stake,
        }
    }

    /// The chassis participant set (sorted G2 keys). Participant index `i`
    /// corresponds to `self.operators[i]`.
    pub fn participants(&self) -> Set<PublicKey> {
        Set::from_iter_dedup(self.operators.iter().map(|op| op.g2_pub_key.clone()))
    }

    /// G1 keys index-aligned with [`JitoQuorum::participants`].
    pub fn g1_keys(&self) -> Vec<G1PublicKey> {
        self.operators
            .iter()
            .map(|op| op.g1_pub_key.clone())
            .collect()
    }

    /// On-chain operator indices aligned with [`JitoQuorum::participants`] —
    /// the participant→bitmap-bit translation table the submitter uses.
    pub fn operator_indices(&self) -> Vec<u64> {
        self.operators
            .iter()
            .map(|op| op.ncn_operator_index)
            .collect()
    }

    /// Stake weights aligned with [`JitoQuorum::participants`].
    pub fn stakes(&self) -> Vec<u128> {
        self.operators.iter().map(|op| op.stake_weight).collect()
    }

    /// Startup quorum reconciliation (INTERFACES.md §5): asserts the lightest
    /// possible engine quorum still clears the on-chain stake threshold.
    /// Returns an error when the router/node must refuse to start.
    pub fn reconcile_engine_quorum(&self) -> Result<QuorumReconciliation> {
        let summary = reconcile(&self.stakes(), self.threshold_bps)
            .map_err(|e| anyhow!("quorum reconciliation failed: {e}"))?;
        info!(
            participants = summary.participants,
            engine_quorum = summary.engine_quorum,
            lightest_quorum_bps = summary.lightest_quorum_bps,
            threshold_bps = summary.threshold_bps,
            "engine quorum clears the on-chain stake threshold"
        );
        Ok(summary)
    }
}

/// RPC client over one NCN deployment.
pub struct JitoStakingClient {
    rpc: RpcClient,
    deployment: NcnDeployment,
    commitment: CommitmentConfig,
}

impl JitoStakingClient {
    /// Builds a client from a validated deployment config. All reads use the
    /// deployment's commitment (`confirmed` minimum, enforced at load).
    pub fn new(deployment: NcnDeployment) -> Result<Self> {
        let commitment = deployment
            .read_commitment()
            .map_err(|e| anyhow!("invalid read commitment: {e}"))?;
        let rpc = RpcClient::new_with_commitment(deployment.rpc_http_url.clone(), commitment);
        Ok(Self {
            rpc,
            deployment,
            commitment,
        })
    }

    /// The deployment this client reads.
    pub fn deployment(&self) -> &NcnDeployment {
        &self.deployment
    }

    /// Fetches the full quorum view: `NCNOperatorAccount`s joined with the
    /// `Snapshot` PDA, sorted into participant order.
    pub async fn get_quorum(&self) -> Result<JitoQuorum> {
        let ncn = self
            .deployment
            .ncn()
            .map_err(|e| anyhow!("invalid ncn pubkey: {e}"))?;
        let program_id = self
            .deployment
            .ncn_program_id()
            .map_err(|e| anyhow!("invalid ncn program id: {e}"))?;

        let accounts = self.fetch_operator_accounts(&program_id, &ncn).await?;
        let snapshot = self.fetch_snapshot().await?;

        let mut operators = Vec::with_capacity(accounts.len());
        for account in &accounts {
            if account.is_empty() {
                continue;
            }
            let Ok(g2_pub_key) = PublicKey::try_from(account.g2_pubkey().as_slice()) else {
                // Deterministic rule (same on-chain data => same participant
                // set on every process): operators with invalid keys are
                // excluded everywhere.
                warn!(
                    operator = %account.operator_pubkey(),
                    "operator registered an invalid/identity G2 key; excluding"
                );
                continue;
            };
            let Ok(g1_pub_key) = G1PublicKey::try_from(account.g1_pubkey().as_slice()) else {
                warn!(
                    operator = %account.operator_pubkey(),
                    "operator registered an invalid/identity G1 key; excluding"
                );
                continue;
            };

            let (stake_weight, can_vote) =
                match snapshot.find_operator_snapshot(account.operator_pubkey()) {
                    Some(op_snapshot) => (
                        op_snapshot.stake_weight().stake_weight(),
                        op_snapshot.is_active() && op_snapshot.has_minimum_stake(),
                    ),
                    None => (0, false),
                };

            operators.push(JitoOperatorInfo {
                operator: *account.operator_pubkey(),
                ncn_operator_index: account.ncn_operator_index(),
                g2_pub_key,
                g1_pub_key,
                socket: socket_from_registration(account.ip_address(), account.port()),
                stake_weight,
                can_vote,
            });
        }

        let quorum = JitoQuorum::from_parts(
            operators,
            snapshot.operators_registered(),
            // TODO-FREEZE(phase1-dmsg): read snapshot.generation() once the
            // Phase 1 snapshot layout lands on main (see JitoQuorum docs).
            0,
            snapshot.last_snapshot_slot(),
            self.deployment.consensus_threshold_bps,
        );
        info!(
            operators = quorum.operators.len(),
            operators_registered = quorum.operators_registered,
            total_stake = quorum.total_stake,
            last_snapshot_slot = quorum.last_snapshot_slot,
            "quorum loaded from chain"
        );
        Ok(quorum)
    }

    /// All `NCNOperatorAccount`s of this NCN via `getProgramAccounts`:
    /// discriminator memcmp at offset 0, `ncn` field memcmp at offset 8
    /// (jito-bytemuck layout: 1-byte discriminator + 7 padding, struct at 8).
    async fn fetch_operator_accounts(
        &self,
        program_id: &Pubkey,
        ncn: &Pubkey,
    ) -> Result<Vec<NCNOperatorAccount>> {
        let account_size = 8 + size_of::<NCNOperatorAccount>();
        let filters = vec![
            RpcFilterType::DataSize(account_size as u64),
            RpcFilterType::Memcmp(Memcmp::new(
                0,
                MemcmpEncodedBytes::Bytes(vec![NCNOperatorAccount::DISCRIMINATOR]),
            )),
            RpcFilterType::Memcmp(Memcmp::new(
                8,
                MemcmpEncodedBytes::Bytes(ncn.to_bytes().to_vec()),
            )),
        ];
        let config = RpcProgramAccountsConfig {
            filters: Some(filters),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: None,
                commitment: Some(self.commitment),
                min_context_slot: None,
            },
            with_context: Some(false),
            sort_results: Some(false),
        };
        let results = self
            .rpc
            .get_program_accounts_with_config(program_id, config)
            .await
            .context("getProgramAccounts(NCNOperatorAccount) failed")?;

        let mut accounts = Vec::with_capacity(results.len());
        for (address, account) in &results {
            match NCNOperatorAccount::try_from_slice_unchecked(account.data.as_slice()) {
                Ok(parsed) => accounts.push(*parsed),
                Err(error) => warn!(
                    account = %address,
                    ?error,
                    "undecodable NCNOperatorAccount; skipping"
                ),
            }
        }
        Ok(accounts)
    }

    /// Fetches and parses the `Snapshot` PDA.
    async fn fetch_snapshot(&self) -> Result<Snapshot> {
        let snapshot_pda = self
            .deployment
            .snapshot_pda()
            .map_err(|e| anyhow!("snapshot pda derivation failed: {e}"))?;
        let account = self
            .rpc
            .get_account_with_commitment(&snapshot_pda, self.commitment)
            .await
            .context("get_account(Snapshot) failed")?
            .value
            .ok_or_else(|| {
                anyhow!("snapshot PDA {snapshot_pda} does not exist; initialize the NCN snapshot")
            })?;
        let snapshot = Snapshot::try_from_slice_unchecked(account.data.as_slice())
            .map_err(|error| anyhow!("undecodable Snapshot account: {error:?}"))?;
        Ok(*snapshot)
    }
}

/// Converts the on-chain ip/port registration into a socket address. All-zero
/// ip or zero port means "not registered yet".
fn socket_from_registration(ip: &[u8; 4], port: u16) -> Option<SocketAddr> {
    if *ip == [0, 0, 0, 0] || port == 0 {
        return None;
    }
    Some(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])),
        port,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bn254::Bn254Signer;
    use commonware_cryptography::Signer as _;
    use commonware_math::algebra::Random as _;
    use commonware_utils::ordered::Quorum as _;
    use commonware_utils::test_rng;

    fn operator(signer: &Bn254Signer, index: u64, stake: u128) -> JitoOperatorInfo {
        JitoOperatorInfo {
            operator: Pubkey::new_unique(),
            ncn_operator_index: index,
            g2_pub_key: signer.public_key(),
            g1_pub_key: signer.public_g1(),
            socket: socket_from_registration(&[10, 0, 0, index as u8 + 1], 9000 + index as u16),
            stake_weight: stake,
            can_vote: stake > 0,
        }
    }

    #[test]
    fn quorum_vectors_are_index_aligned_with_the_participant_set() {
        let mut rng = test_rng();
        let signers: Vec<Bn254Signer> = (0..5).map(|_| Bn254Signer::random(&mut rng)).collect();
        let operators: Vec<JitoOperatorInfo> = signers
            .iter()
            .enumerate()
            .map(|(i, s)| operator(s, i as u64, (i as u128 + 1) * 100))
            .collect();

        let quorum = JitoQuorum::from_parts(operators.clone(), 5, 0, 1234, 6667);
        let participants = quorum.participants();
        assert_eq!(participants.len(), 5);

        // Every aligned vector must agree with the Set's index for the same
        // operator — this is the invariant the submitter's bitmap translation
        // depends on.
        let g1_keys = quorum.g1_keys();
        let indices = quorum.operator_indices();
        let stakes = quorum.stakes();
        for (position, op) in quorum.operators.iter().enumerate() {
            let set_index = participants
                .index(&op.g2_pub_key)
                .expect("operator key is in the set");
            assert_eq!(usize::from(set_index), position);
            assert_eq!(g1_keys[position], op.g1_pub_key);
            assert_eq!(indices[position], op.ncn_operator_index);
            assert_eq!(stakes[position], op.stake_weight);
        }
        assert_eq!(quorum.total_stake, 100 + 200 + 300 + 400 + 500);
    }

    #[test]
    fn duplicate_g2_keys_keep_first_occurrence() {
        let mut rng = test_rng();
        let a = Bn254Signer::random(&mut rng);
        let b = Bn254Signer::random(&mut rng);
        let dup = operator(&a, 7, 999);
        let quorum = JitoQuorum::from_parts(
            vec![operator(&a, 0, 100), operator(&b, 1, 200), dup],
            3,
            0,
            0,
            6667,
        );
        assert_eq!(quorum.operators.len(), 2);
        let kept: Vec<u64> = quorum.operator_indices();
        // The duplicate (index 7) is dropped, whichever sort position the key
        // lands in.
        assert!(kept.contains(&0));
        assert!(kept.contains(&1));
        assert!(!kept.contains(&7));
    }

    #[test]
    fn reconciliation_uses_participant_stakes() {
        let mut rng = test_rng();
        let signers: Vec<Bn254Signer> = (0..4).map(|_| Bn254Signer::random(&mut rng)).collect();
        let equal: Vec<JitoOperatorInfo> = signers
            .iter()
            .enumerate()
            .map(|(i, s)| operator(s, i as u64, 100))
            .collect();
        let quorum = JitoQuorum::from_parts(equal, 4, 0, 0, 6667);
        quorum
            .reconcile_engine_quorum()
            .expect("equal weights clear 6667 bps");

        let skewed: Vec<JitoOperatorInfo> = signers
            .iter()
            .enumerate()
            .map(|(i, s)| operator(s, i as u64, if i == 0 { 10_000 } else { 1 }))
            .collect();
        let quorum = JitoQuorum::from_parts(skewed, 4, 0, 0, 6667);
        assert!(quorum.reconcile_engine_quorum().is_err());
    }

    #[test]
    fn socket_registration_zero_values_mean_unset() {
        assert_eq!(socket_from_registration(&[0, 0, 0, 0], 8000), None);
        assert_eq!(socket_from_registration(&[10, 0, 0, 1], 0), None);
        assert_eq!(
            socket_from_registration(&[10, 0, 0, 1], 8000),
            Some(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
                8000
            ))
        );
    }
}
