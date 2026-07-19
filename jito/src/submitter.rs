//! JitoSubmitter: turns certified heights into NCN `VerifyCertificate`
//! submissions — the Solana peer of `commonware_avs_router::submitter`.
//!
//! Consumes certificate observations from the chassis
//! [`CertReporter`](commonware_avs_router::reporter) and resolves each one:
//!
//! - `skip_digest(namespace, height)`: the quorum abandoned the height — log
//!   and notify the sequencer ([`ResolutionKind::Skipped`]).
//! - No assignment / digest mismatch: pre-restart leftovers — notify
//!   ([`ResolutionKind::Foreign`]).
//! - Expected digest: translate the certificate's participant bitmap into the
//!   ON-CHAIN operator-index bitmap, package the §2 wire triple
//!   (`aggregated_signature`, `aggregated_g2`, bitmap) with the
//!   `expected_generation` captured at quorum-assembly time, and hand it to
//!   the [`SolanaCertificateHandler`] with bounded retries.
//!
//! # Finality semantics (INTERFACES.md §5)
//!
//! PDAs are read at `confirmed` minimum when building instructions;
//! `Resolution{Executed}` is reported only once the transaction is
//! `finalized`; blockhash expiry triggers rebuild-and-resend of the
//! TRANSACTION (the certificate itself never expires). The EVM model's
//! "receipt == inclusion" assumption has no processed-level analog under
//! Solana forks.

use crate::config::NcnDeployment;
use crate::instruction::{
    VerifyCertificateAccounts, VerifyCertificateArgs, signer_bitmap, verify_certificate_instruction,
};
use crate::scheme::{JitoBn254Scheme, JitoCertificate};
use anyhow::{Context, Result, anyhow};
use commonware_avs_core::wire::{TaskData, skip_digest};
use commonware_avs_router::reporter::{CertifiedHeight, CertifiedReceiver};
use commonware_avs_router::sequencer::SharedAssignments;
use commonware_avs_router::sequencer::{Resolution, ResolutionKind, ResolutionSender};
use commonware_cryptography::certificate::Scheme as _;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::hash::Hash as Blockhash;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature as TxSignature};
use solana_sdk::signer::Signer as _;
use solana_sdk::transaction::Transaction;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Retries after the first failed submission attempt (handler-level errors).
const MAX_RETRIES: u32 = 2;

/// Delay before the first retry; doubles per attempt.
const INITIAL_RETRY_BACKOFF: Duration = Duration::from_secs(2);

/// How many times an expired-blockhash transaction is rebuilt and resent
/// before the attempt is reported as failed.
const MAX_TX_REBUILDS: u32 = 3;

/// Poll cadence while waiting for finalization.
const FINALITY_POLL_INTERVAL: Duration = Duration::from_millis(500);

/// The §2 wire material for one certified digest, resolved by the submitter
/// and consumed by a [`SolanaCertificateHandler`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateSubmission {
    /// The certified 32-byte digest.
    pub digest: [u8; 32],
    /// Aggregated G2 key of the signers (compressed, 64B).
    pub aggregated_g2: [u8; 64],
    /// Aggregated G1 signature (compressed, 32B).
    pub aggregated_signature: [u8; 32],
    /// LSB-first bitmap over ON-CHAIN operator indices.
    pub operators_signature_bitmap: Vec<u8>,
    /// Snapshot generation captured when the quorum was assembled.
    pub expected_generation: u64,
}

/// Outcome of an on-chain submission, reported back through the resolution
/// channel (`success` only after FINALIZED observation).
#[derive(Debug, Clone)]
pub struct SolanaExecutionResult {
    /// The transaction signature (base58).
    pub signature: String,
    /// Slot the transaction finalized in, when known.
    pub slot: Option<u64>,
    /// Whether the finalized transaction succeeded (no on-chain error).
    pub success: bool,
}

/// Program-specific handler for a certified digest's on-chain verification —
/// the Solana-typed peer of the EVM `BlsSignatureVerificationHandler` seam.
///
/// The submitter has already translated the certificate into the §2 wire
/// triple; implementations decide WHICH instruction consumes it: the NCN
/// program's standalone `VerifyCertificate` ([`VerifyCertificateHandler`]) or
/// a settlement program's `settle` instruction (INTERFACES.md §4). Track C's
/// `gaskiller-settlement` program (jito-ncn-program branch
/// `settlement-program`, crate `settlement_core`) exposes the settle wire
/// format for that handler: `SETTLE_DISCRIMINATOR ‖ borsh(payload,
/// aggregated_g2, aggregated_signature, bitmap, expected_generation)` with
/// `digest = sha256(borsh(SettlementPayload))` — a settle handler feeds this
/// seam's [`CertificateSubmission`] fields into that instruction unchanged
/// once the branch merges to `main`.
#[async_trait::async_trait]
pub trait SolanaCertificateHandler: Send + Sync {
    /// The application's task payload type.
    type TaskData: TaskData;

    /// Submits (or otherwise executes) the verification for `height`.
    ///
    /// Implementations MUST only report `success == true` for transactions
    /// observed at `finalized` commitment.
    async fn handle_verification(
        &mut self,
        height: u64,
        submission: CertificateSubmission,
        task_data: Option<&Self::TaskData>,
    ) -> Result<SolanaExecutionResult>;
}

/// Consumes certified heights and drives the on-chain submission.
pub struct JitoSubmitter<T, H>
where
    T: TaskData,
    H: SolanaCertificateHandler<TaskData = T>,
{
    /// Participant index → on-chain `ncn_operator_index`, aligned with the
    /// scheme's participant set (from `JitoQuorum::operator_indices`).
    operator_indices: Vec<u64>,
    /// Bit-length domain of the on-chain bitmap (snapshot
    /// `operators_registered`).
    operators_registered: u64,
    /// Snapshot generation captured at quorum-assembly time; stale
    /// generations are rejected on-chain, which is exactly the protection the
    /// generation check exists for.
    expected_generation: u64,
    /// Executes the final on-chain call.
    handler: H,
    /// Height → expected digest + task, written by the sequencer.
    assignments: SharedAssignments<T>,
    /// Verified certificates from the reporter.
    certified: CertifiedReceiver<JitoBn254Scheme>,
    /// Final dispositions back to the sequencer.
    resolutions: ResolutionSender,
    /// Application namespace bound into `skip_digest`.
    namespace: Vec<u8>,
}

impl<T, H> JitoSubmitter<T, H>
where
    T: TaskData,
    H: SolanaCertificateHandler<TaskData = T>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        scheme: JitoBn254Scheme,
        operator_indices: Vec<u64>,
        operators_registered: u64,
        expected_generation: u64,
        handler: H,
        assignments: SharedAssignments<T>,
        certified: CertifiedReceiver<JitoBn254Scheme>,
        resolutions: ResolutionSender,
        namespace: Vec<u8>,
    ) -> Self {
        assert_eq!(
            scheme.participants().len(),
            operator_indices.len(),
            "operator_indices must be index-aligned with the scheme participants"
        );
        // The scheme itself is only needed for the alignment assertion: the
        // certificate already carries the aggregate points, and the reporter
        // re-verified it against the same verifier scheme instance.
        let _ = scheme;
        Self {
            operator_indices,
            operators_registered,
            expected_generation,
            handler,
            assignments,
            certified,
            resolutions,
            namespace,
        }
    }

    /// Runs until the reporter side of the certified channel closes.
    pub async fn run(mut self) {
        while let Some(certified) = self.certified.recv().await {
            self.handle_certified(certified).await;
        }
        info!("certified channel closed; jito submitter exiting");
    }

    /// Notifies the sequencer of a height's final disposition.
    fn notify(&self, height: u64, kind: ResolutionKind) {
        if self.resolutions.send(Resolution { height, kind }).is_err() {
            // The sequencer is gone; the process is shutting down.
            warn!(height, "resolution channel closed");
        }
    }

    async fn handle_certified(&mut self, certified: CertifiedHeight<JitoBn254Scheme>) {
        let CertifiedHeight {
            height,
            digest,
            certificate,
        } = certified;

        // Skip certificate: the quorum abandoned the height.
        if digest == skip_digest(&self.namespace, height) {
            info!(height, "skip certificate observed; nothing to submit");
            self.notify(height, ResolutionKind::Skipped);
            return;
        }

        // Look up the sequencer's assignment for this height.
        let assignment = match self.assignments.read() {
            Ok(assignments) => assignments.get(&height).cloned(),
            Err(_) => {
                error!(height, "assignments lock poisoned; treating as unassigned");
                None
            }
        };
        let Some(assignment) = assignment else {
            info!(
                height,
                digest = %digest,
                "certificate for unassigned height; nothing to submit"
            );
            self.notify(height, ResolutionKind::Foreign);
            return;
        };
        if assignment.digest != digest {
            warn!(
                height,
                expected = %assignment.digest,
                certified = %digest,
                "certificate digest does not match assignment; height consumed"
            );
            self.notify(height, ResolutionKind::Foreign);
            return;
        }

        // Expected digest: resolve the wire material once, then submit with
        // bounded retries (transient RPC errors retry; deterministic
        // rejections burn two extra round-trips then release the height).
        let submission = match self.resolve_submission(&digest, &certificate) {
            Ok(submission) => submission,
            Err(error) => {
                error!(height, %error, "certificate could not be resolved for submission");
                self.notify(height, ResolutionKind::Executed { success: false });
                return;
            }
        };

        let mut backoff = INITIAL_RETRY_BACKOFF;
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            match self
                .handler
                .handle_verification(height, submission.clone(), Some(&assignment.task))
                .await
            {
                Ok(result) => {
                    info!(
                        height,
                        tx = %result.signature,
                        slot = ?result.slot,
                        success = result.success,
                        "verification finalized"
                    );
                    self.notify(
                        height,
                        ResolutionKind::Executed {
                            success: result.success,
                        },
                    );
                    return;
                }
                Err(error) if attempt <= MAX_RETRIES => {
                    warn!(
                        height,
                        attempt,
                        max_retries = MAX_RETRIES,
                        %error,
                        backoff_secs = backoff.as_secs_f64(),
                        "submission failed; retrying"
                    );
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                }
                Err(error) => {
                    error!(
                        height,
                        attempts = attempt,
                        %error,
                        "submission failed after retries; releasing height"
                    );
                    self.notify(height, ResolutionKind::Executed { success: false });
                    return;
                }
            }
        }
    }

    /// Translates a verified certificate into the §2 wire material: the
    /// participant bitmap becomes the LSB-first bitmap over on-chain operator
    /// indices, and the aggregate points ride in compressed form.
    fn resolve_submission(
        &self,
        digest: &commonware_cryptography::sha256::Digest,
        certificate: &JitoCertificate,
    ) -> Result<CertificateSubmission> {
        let digest: [u8; 32] = digest
            .as_ref()
            .try_into()
            .map_err(|_| anyhow!("certified digest is not 32 bytes"))?;

        // The certificate was verified by the reporter, so the bitmap length
        // matches the participant set; translate each set bit.
        let mut onchain_indices = Vec::with_capacity(certificate.signers.count());
        for participant in certificate.signers.iter() {
            let index = self
                .operator_indices
                .get(usize::from(participant))
                .copied()
                .ok_or_else(|| {
                    anyhow!("participant {participant} has no on-chain operator index")
                })?;
            onchain_indices.push(index);
        }
        let bitmap = signer_bitmap(&onchain_indices, self.operators_registered)?;

        Ok(CertificateSubmission {
            digest,
            aggregated_g2: certificate.agg_g2.compressed(),
            aggregated_signature: certificate.signature.compressed(),
            operators_signature_bitmap: bitmap,
            expected_generation: self.expected_generation,
        })
    }
}

/// The standalone-demo handler: submits the NCN program's `VerifyCertificate`
/// instruction (INTERFACES.md §2) with a compute budget, resending on
/// blockhash expiry, and reporting success only at `finalized`.
pub struct VerifyCertificateHandler<T: TaskData> {
    rpc: RpcClient,
    program_id: Pubkey,
    accounts: VerifyCertificateAccounts,
    payer: Keypair,
    compute_unit_limit: u32,
    compute_unit_price: Option<u64>,
    read_commitment: CommitmentConfig,
    _task: std::marker::PhantomData<fn() -> T>,
}

impl<T: TaskData> VerifyCertificateHandler<T> {
    /// Builds the handler from a deployment config and the fee-payer keypair.
    pub fn new(deployment: &NcnDeployment, payer: Keypair) -> Result<Self> {
        let read_commitment = deployment
            .read_commitment()
            .map_err(|e| anyhow!("invalid read commitment: {e}"))?;
        Ok(Self {
            rpc: RpcClient::new_with_commitment(deployment.rpc_http_url.clone(), read_commitment),
            program_id: deployment
                .ncn_program_id()
                .map_err(|e| anyhow!("invalid ncn program id: {e}"))?,
            accounts: VerifyCertificateAccounts {
                ncn_config: deployment
                    .ncn_config_pda()
                    .map_err(|e| anyhow!("config pda: {e}"))?,
                ncn: deployment.ncn().map_err(|e| anyhow!("ncn pubkey: {e}"))?,
                snapshot: deployment
                    .snapshot_pda()
                    .map_err(|e| anyhow!("snapshot pda: {e}"))?,
                restaking_config: deployment
                    .restaking_config_pda()
                    .map_err(|e| anyhow!("restaking config pda: {e}"))?,
            },
            payer,
            compute_unit_limit: deployment.compute_unit_limit,
            compute_unit_price: deployment.compute_unit_price,
            read_commitment,
            _task: std::marker::PhantomData,
        })
    }

    /// The instruction list for one submission: compute budget first, then
    /// `VerifyCertificate`.
    fn instructions(&self, submission: &CertificateSubmission) -> Vec<Instruction> {
        let mut instructions = vec![ComputeBudgetInstruction::set_compute_unit_limit(
            self.compute_unit_limit,
        )];
        if let Some(price) = self.compute_unit_price {
            instructions.push(ComputeBudgetInstruction::set_compute_unit_price(price));
        }
        instructions.push(verify_certificate_instruction(
            &self.program_id,
            &self.accounts,
            &VerifyCertificateArgs {
                digest: submission.digest,
                aggregated_g2: submission.aggregated_g2,
                aggregated_signature: submission.aggregated_signature,
                operators_signature_bitmap: submission.operators_signature_bitmap.clone(),
                expected_generation: submission.expected_generation,
            },
        ));
        instructions
    }

    /// Sends one signed transaction and waits for finalization or blockhash
    /// expiry. Returns `Ok(Some(result))` on a finalized observation,
    /// `Ok(None)` when the blockhash expired unobserved (rebuild and resend).
    async fn send_and_finalize(
        &self,
        instructions: &[Instruction],
        blockhash: Blockhash,
    ) -> Result<Option<SolanaExecutionResult>> {
        let transaction = Transaction::new_signed_with_payer(
            instructions,
            Some(&self.payer.pubkey()),
            &[&self.payer],
            blockhash,
        );
        let signature: TxSignature = self
            .rpc
            .send_transaction_with_config(
                &transaction,
                RpcSendTransactionConfig {
                    preflight_commitment: Some(self.read_commitment.commitment),
                    ..RpcSendTransactionConfig::default()
                },
            )
            .await
            .context("send_transaction failed")?;
        debug!(tx = %signature, "verify-certificate transaction sent");

        loop {
            let statuses = self
                .rpc
                .get_signature_statuses(&[signature])
                .await
                .context("get_signature_statuses failed")?;
            if let Some(Some(status)) = statuses.value.first() {
                if matches!(
                    status.confirmation_status(),
                    solana_transaction_status_client_types::TransactionConfirmationStatus::Finalized
                ) {
                    return Ok(Some(SolanaExecutionResult {
                        signature: signature.to_string(),
                        slot: Some(status.slot),
                        success: status.err.is_none(),
                    }));
                }
            } else {
                // Unobserved: if the blockhash died, the tx can never land —
                // signal a rebuild.
                let still_valid = self
                    .rpc
                    .is_blockhash_valid(&blockhash, self.read_commitment)
                    .await
                    .context("is_blockhash_valid failed")?;
                if !still_valid {
                    warn!(tx = %signature, "blockhash expired before observation; rebuilding");
                    return Ok(None);
                }
            }
            tokio::time::sleep(FINALITY_POLL_INTERVAL).await;
        }
    }
}

#[async_trait::async_trait]
impl<T: TaskData> SolanaCertificateHandler for VerifyCertificateHandler<T> {
    type TaskData = T;

    async fn handle_verification(
        &mut self,
        height: u64,
        submission: CertificateSubmission,
        _task_data: Option<&T>,
    ) -> Result<SolanaExecutionResult> {
        let instructions = self.instructions(&submission);
        for rebuild in 0..MAX_TX_REBUILDS {
            // A fresh blockhash per attempt: the TRANSACTION expires, the
            // certificate does not.
            let (blockhash, _) = self
                .rpc
                .get_latest_blockhash_with_commitment(self.read_commitment)
                .await
                .context("get_latest_blockhash failed")?;
            match self.send_and_finalize(&instructions, blockhash).await? {
                Some(result) => return Ok(result),
                None => {
                    warn!(
                        height,
                        rebuild = rebuild + 1,
                        max = MAX_TX_REBUILDS,
                        "resending verify-certificate transaction with a fresh blockhash"
                    );
                }
            }
        }
        Err(anyhow!(
            "transaction expired {MAX_TX_REBUILDS} times without finalized observation"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bn254::{Bn254Signer, G1PublicKey, PublicKey};
    use commonware_consensus::aggregation::types::Item;
    use commonware_consensus::types::Height;
    use commonware_cryptography::{Hasher as _, Sha256, Signer as _};
    use commonware_math::algebra::Random as _;
    use commonware_parallel::Sequential;
    use commonware_utils::ordered::Set;
    use commonware_utils::{N3f1, TryCollect, test_rng};

    /// Builds a real 4-signer scheme plus shuffled on-chain operator indices
    /// and returns everything a submission resolution needs.
    fn setup() -> (JitoBn254Scheme, Vec<JitoBn254Scheme>, Vec<u64>) {
        let mut rng = test_rng();
        let keys: Vec<Bn254Signer> = (0..4).map(|_| Bn254Signer::random(&mut rng)).collect();
        let participants: Set<PublicKey> = keys
            .iter()
            .map(|k| k.public_key())
            .try_collect()
            .expect("no duplicate keys");
        let g1_keys: Vec<G1PublicKey> = participants
            .iter()
            .map(|pk| {
                keys.iter()
                    .find(|k| &k.public_key() == pk)
                    .unwrap()
                    .public_g1()
            })
            .collect();
        let mut schemes: Vec<JitoBn254Scheme> = keys
            .iter()
            .map(|k| {
                JitoBn254Scheme::signer(participants.clone(), g1_keys.clone(), k.private_key())
                    .unwrap()
            })
            .collect();
        schemes.sort_by_key(|s| s.me().unwrap());
        let verifier = JitoBn254Scheme::verifier(participants, g1_keys);
        // On-chain indices deliberately differ from participant indices.
        let operator_indices = vec![9, 2, 5, 0];
        (verifier, schemes, operator_indices)
    }

    fn certified(
        schemes: &[JitoBn254Scheme],
        verifier: &JitoBn254Scheme,
        count: usize,
    ) -> (commonware_cryptography::sha256::Digest, JitoCertificate) {
        let mut hasher = Sha256::new();
        hasher.update(b"submission test digest");
        let digest = hasher.finalize();
        let item = Item {
            height: Height::new(3),
            digest,
        };
        let attestations: Vec<_> = schemes
            .iter()
            .take(count)
            .map(|s| {
                s.sign::<commonware_cryptography::sha256::Digest>(&item)
                    .unwrap()
            })
            .collect();
        let certificate = verifier
            .assemble::<_, N3f1>(attestations, &Sequential)
            .unwrap();
        (digest, certificate)
    }

    /// A handler that records what it received (no chain interaction; the
    /// crypto feeding it is real).
    struct RecordingHandler {
        received: std::sync::Arc<std::sync::Mutex<Vec<CertificateSubmission>>>,
    }

    #[async_trait::async_trait]
    impl SolanaCertificateHandler for RecordingHandler {
        type TaskData = u64;

        async fn handle_verification(
            &mut self,
            _height: u64,
            submission: CertificateSubmission,
            _task_data: Option<&u64>,
        ) -> Result<SolanaExecutionResult> {
            self.received.lock().unwrap().push(submission);
            Ok(SolanaExecutionResult {
                signature: "test".into(),
                slot: Some(1),
                success: true,
            })
        }
    }

    fn submitter_with(
        verifier: JitoBn254Scheme,
        operator_indices: Vec<u64>,
        operators_registered: u64,
    ) -> (
        JitoSubmitter<u64, RecordingHandler>,
        std::sync::Arc<std::sync::Mutex<Vec<CertificateSubmission>>>,
    ) {
        let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let handler = RecordingHandler {
            received: received.clone(),
        };
        let assignments = commonware_avs_router::sequencer::shared_assignments::<u64>();
        let (_, certified_rx) =
            commonware_avs_router::reporter::certified_channel::<JitoBn254Scheme>();
        let (resolutions_tx, _resolutions_rx) =
            commonware_avs_router::sequencer::resolution_channel();
        (
            JitoSubmitter::new(
                verifier,
                operator_indices,
                operators_registered,
                7,
                handler,
                assignments,
                certified_rx,
                resolutions_tx,
                b"_TEST".to_vec(),
            ),
            received,
        )
    }

    #[test]
    fn resolve_submission_translates_participant_bits_to_onchain_bitmap() {
        let (verifier, schemes, operator_indices) = setup();
        // Signers = participants 0..3 (first three by sorted order); their
        // on-chain indices are 9, 2, 5.
        let (digest, certificate) = certified(&schemes, &verifier, 3);
        let (submitter, _) = submitter_with(verifier, operator_indices, 10);

        let submission = submitter
            .resolve_submission(&digest, &certificate)
            .expect("resolvable");
        assert_eq!(submission.expected_generation, 7);
        assert_eq!(
            submission.digest,
            <[u8; 32]>::try_from(digest.as_ref()).unwrap()
        );
        assert_eq!(
            submission.aggregated_signature,
            certificate.signature.compressed()
        );
        assert_eq!(submission.aggregated_g2, certificate.agg_g2.compressed());
        // operators_registered = 10 → 2 bytes; bits 9, 2 and 5 set, and the
        // padding bits 10..15 SET per the program builder's convention.
        assert_eq!(
            submission.operators_signature_bitmap,
            vec![0b0010_0100, 0b1111_1110]
        );
    }

    #[test]
    fn resolve_submission_rejects_out_of_range_onchain_index() {
        let (verifier, schemes, operator_indices) = setup();
        let (digest, certificate) = certified(&schemes, &verifier, 3);
        // operators_registered = 6 but participant 0 maps to on-chain index 9.
        let (submitter, _) = submitter_with(verifier, operator_indices, 6);
        assert!(submitter.resolve_submission(&digest, &certificate).is_err());
    }

    #[test]
    fn instructions_carry_compute_budget_and_frozen_account_order() {
        // Handler construction is pure (no RPC I/O until used).
        let deployment = NcnDeployment::parse(
            r#"{
                "rpcHttpUrl": "http://127.0.0.1:8899",
                "ncnProgramId": "Vote111111111111111111111111111111111111111",
                "ncn": "BPFLoaderUpgradeab1e11111111111111111111111",
                "restakingProgramId": "RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q",
                "computeUnitPrice": 1000
            }"#,
        )
        .unwrap();
        let handler: VerifyCertificateHandler<u64> =
            VerifyCertificateHandler::new(&deployment, Keypair::new()).unwrap();
        let submission = CertificateSubmission {
            digest: [1; 32],
            aggregated_g2: [2; 64],
            aggregated_signature: [3; 32],
            operators_signature_bitmap: vec![0b111],
            expected_generation: 0,
        };
        let instructions = handler.instructions(&submission);
        assert_eq!(instructions.len(), 3, "cu limit + cu price + verify");
        // The verify instruction is last, addressed to the NCN program with
        // the §2 read-only account order.
        let verify = instructions.last().unwrap();
        assert_eq!(verify.program_id, deployment.ncn_program_id().unwrap());
        assert_eq!(verify.accounts.len(), 4);
        assert!(
            verify
                .accounts
                .iter()
                .all(|m| !m.is_writable && !m.is_signer)
        );
    }
}
