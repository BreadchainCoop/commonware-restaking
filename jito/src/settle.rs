//! Settlement handler for the [`SolanaCertificateHandler`] seam: lands a
//! certified `SettlementPayload` via the gaskiller-settlement program's
//! `Settle` instruction (INTERFACES.md §4) instead of the standalone
//! `VerifyCertificate` demo.
//!
//! The task data IS the borsh-encoded payload (`AsRef<[u8]>`): the digest the
//! quorum certified is `sha256(borsh(SettlementPayload))`, so the handler
//! recomputes it from the task bytes and refuses to submit on any mismatch —
//! the certificate and the payload travel together or not at all. The §2
//! certificate wire triple from [`CertificateSubmission`] feeds `SettleArgs`
//! unchanged; when the payload carries a `story_meta` event the digest-verified
//! buffer PDA for `(state, transition_index)` is appended as the optional
//! trailing account.

use crate::config::NcnDeployment;
use crate::submitter::{
    CertificateSubmission, FinalizedSender, SolanaCertificateHandler, SolanaExecutionResult,
};
use anyhow::{Context, Result, anyhow};
use borsh10::BorshDeserialize as _;
use commonware_avs_core::wire::TaskData;
use settlement_core::buffer::find_buffer_program_address;
use settlement_core::instruction::{SettleArgs, settle_ix};
use settlement_core::payload::{STORY_META_DISCRIMINANT, SettlementPayload, StateUpdate};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use tracing::info;

/// Submits certified settlement payloads as `Settle` transactions.
pub struct SettleCertificateHandler<T> {
    sender: FinalizedSender,
    /// The gaskiller-settlement program id.
    program_id: Pubkey,
    /// The consumer's `GkState` PDA, derived from the deployment config.
    state: Pubkey,
    ncn_config: Pubkey,
    ncn: Pubkey,
    snapshot: Pubkey,
    restaking_config: Pubkey,
    compute_unit_limit: u32,
    compute_unit_price: Option<u64>,
    _task: std::marker::PhantomData<fn() -> T>,
}

impl<T> SettleCertificateHandler<T> {
    /// Builds the handler from a deployment config (which must carry the
    /// settlement binding: `settlementProgramId` + `appId`) and the fee-payer
    /// keypair.
    pub fn new(deployment: &NcnDeployment, payer: Keypair) -> Result<Self> {
        Ok(Self {
            sender: FinalizedSender::new(deployment, payer)?,
            program_id: deployment
                .settlement_program_id()
                .map_err(|e| anyhow!("settlement program id: {e}"))?,
            state: deployment
                .gk_state_pda()
                .map_err(|e| anyhow!("gk state pda: {e}"))?,
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
            compute_unit_limit: deployment.compute_unit_limit,
            compute_unit_price: deployment.compute_unit_price,
            _task: std::marker::PhantomData,
        })
    }

    /// Builds the settle transaction's instruction list for one certified
    /// payload: compute budget first, then `Settle`.
    fn instructions(
        &self,
        payload_bytes: &[u8],
        submission: &CertificateSubmission,
    ) -> Result<Vec<Instruction>> {
        let payload = SettlementPayload::try_from_slice(payload_bytes)
            .context("task data is not a borsh SettlementPayload")?;

        // The certified digest must be the payload digest — a mismatch means
        // the sequencer assigned bytes the quorum did not certify.
        let digest = payload
            .digest()
            .map_err(|e| anyhow!("payload digest: {e:?}"))?;
        if digest.0 != submission.digest {
            return Err(anyhow!(
                "certified digest {} != payload digest {}; refusing to settle",
                hex::encode(submission.digest),
                hex::encode(digest.0),
            ));
        }
        // The payload must bind OUR state PDA; anything else belongs to a
        // different consumer deployment.
        if payload.state_pda != self.state.to_bytes() {
            return Err(anyhow!(
                "payload state pda {} != configured state pda {}; refusing to settle",
                hex::encode(payload.state_pda),
                self.state,
            ));
        }

        // A story_meta event requires the transition's buffer PDA as the
        // optional trailing account.
        let needs_buffer = payload.updates.iter().any(|update| {
            matches!(
                update,
                StateUpdate::Event { discriminant, .. } if *discriminant == STORY_META_DISCRIMINANT
            )
        });
        let buffer = needs_buffer.then(|| {
            find_buffer_program_address(&self.program_id, &self.state, payload.transition_index).0
        });

        let args = SettleArgs {
            payload,
            aggregated_g2: submission.aggregated_g2,
            aggregated_signature: submission.aggregated_signature,
            operators_signature_bitmap: submission.operators_signature_bitmap.clone(),
            expected_generation: submission.expected_generation,
        };

        let mut instructions = vec![ComputeBudgetInstruction::set_compute_unit_limit(
            self.compute_unit_limit,
        )];
        if let Some(price) = self.compute_unit_price {
            instructions.push(ComputeBudgetInstruction::set_compute_unit_price(price));
        }
        instructions.push(
            settle_ix(
                &self.program_id,
                &self.state,
                &self.ncn_config,
                &self.ncn,
                &self.snapshot,
                &self.restaking_config,
                buffer.as_ref(),
                &args,
            )
            .map_err(|e| anyhow!("settle instruction build: {e:?}"))?,
        );
        Ok(instructions)
    }
}

#[async_trait::async_trait]
impl<T: TaskData + AsRef<[u8]>> SolanaCertificateHandler for SettleCertificateHandler<T> {
    type TaskData = T;

    async fn handle_verification(
        &mut self,
        height: u64,
        submission: CertificateSubmission,
        task_data: Option<&T>,
    ) -> Result<SolanaExecutionResult> {
        let task = task_data.ok_or_else(|| {
            anyhow!("settle handler requires the task payload; none was assigned for this height")
        })?;
        let instructions = self.instructions(task.as_ref(), &submission)?;
        let result = self.sender.submit(height, "settle", &instructions).await?;
        info!(
            height,
            tx = %result.signature,
            success = result.success,
            state = %self.state,
            "settle transaction finalized"
        );
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh10::BorshSerialize as _;
    use settlement_core::instruction::SETTLE_DISCRIMINATOR;
    use settlement_core::payload::StoryMeta;

    fn deployment_with_settlement() -> NcnDeployment {
        NcnDeployment::parse(
            r#"{
                "rpcHttpUrl": "http://127.0.0.1:8899",
                "ncnProgramId": "Vote111111111111111111111111111111111111111",
                "ncn": "BPFLoaderUpgradeab1e11111111111111111111111",
                "restakingProgramId": "RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q",
                "settlementProgramId": "6XTdBk798fEpM2VPBXpkLPw4zJJLvASaiyHaEmj9Ripx",
                "appId": "1111111111111111111111111111111111111111111111111111111111111111"
            }"#,
        )
        .expect("deployment parses")
    }

    fn payload_for(state: Pubkey, buffer: Pubkey) -> SettlementPayload {
        let meta = StoryMeta {
            story_sha256: [0x33; 32],
            buffer,
            len: 457,
        };
        SettlementPayload {
            transition_index: 0,
            state_pda: state.to_bytes(),
            ix_discriminator: SETTLE_DISCRIMINATOR,
            updates: vec![
                StateUpdate::Store { data: [0x22; 32] },
                StateUpdate::Event {
                    discriminant: STORY_META_DISCRIMINANT,
                    payload: meta.try_to_vec().unwrap(),
                },
            ],
        }
    }

    fn submission_for(payload: &SettlementPayload) -> CertificateSubmission {
        CertificateSubmission {
            digest: payload.digest().unwrap().0,
            aggregated_g2: [2; 64],
            aggregated_signature: [3; 32],
            operators_signature_bitmap: vec![0b1111],
            expected_generation: 4,
        }
    }

    #[test]
    fn settle_instruction_carries_payload_cert_and_buffer() {
        let deployment = deployment_with_settlement();
        let handler: SettleCertificateHandler<()> =
            SettleCertificateHandler::new(&deployment, Keypair::new()).unwrap();
        let state = deployment.gk_state_pda().unwrap();
        let program_id = deployment.settlement_program_id().unwrap();
        let buffer = find_buffer_program_address(&program_id, &state, 0).0;

        let payload = payload_for(state, buffer);
        let payload_bytes = payload.try_to_vec().unwrap();
        let submission = submission_for(&payload);

        let instructions = handler.instructions(&payload_bytes, &submission).unwrap();
        assert_eq!(instructions.len(), 2, "cu limit + settle");
        let settle = instructions.last().unwrap();
        assert_eq!(settle.program_id, program_id);
        // state, ncn_config, ncn, snapshot, restaking_config, event_authority,
        // program, buffer.
        assert_eq!(settle.accounts.len(), 8);
        assert_eq!(settle.accounts[0].pubkey, state);
        assert!(settle.accounts[0].is_writable);
        assert_eq!(settle.accounts[7].pubkey, buffer);
        assert_eq!(&settle.data[..8], &SETTLE_DISCRIMINATOR);
        // The wire args round-trip with the certificate material intact.
        let args = SettleArgs::try_from_slice(&settle.data[8..]).unwrap();
        assert_eq!(args.payload, payload);
        assert_eq!(args.aggregated_g2, submission.aggregated_g2);
        assert_eq!(args.aggregated_signature, submission.aggregated_signature);
        assert_eq!(
            args.operators_signature_bitmap,
            submission.operators_signature_bitmap
        );
        assert_eq!(args.expected_generation, submission.expected_generation);
    }

    #[test]
    fn settle_without_story_meta_omits_the_buffer_account() {
        let deployment = deployment_with_settlement();
        let handler: SettleCertificateHandler<()> =
            SettleCertificateHandler::new(&deployment, Keypair::new()).unwrap();
        let state = deployment.gk_state_pda().unwrap();
        let payload = SettlementPayload {
            transition_index: 0,
            state_pda: state.to_bytes(),
            ix_discriminator: SETTLE_DISCRIMINATOR,
            updates: vec![StateUpdate::Store { data: [0x22; 32] }],
        };
        let payload_bytes = payload.try_to_vec().unwrap();
        let submission = submission_for(&payload);
        let instructions = handler.instructions(&payload_bytes, &submission).unwrap();
        assert_eq!(instructions.last().unwrap().accounts.len(), 7);
    }

    #[test]
    fn digest_mismatch_refuses_to_settle() {
        let deployment = deployment_with_settlement();
        let handler: SettleCertificateHandler<()> =
            SettleCertificateHandler::new(&deployment, Keypair::new()).unwrap();
        let state = deployment.gk_state_pda().unwrap();
        let program_id = deployment.settlement_program_id().unwrap();
        let buffer = find_buffer_program_address(&program_id, &state, 0).0;
        let payload = payload_for(state, buffer);
        let payload_bytes = payload.try_to_vec().unwrap();
        let mut submission = submission_for(&payload);
        submission.digest[0] ^= 1;
        assert!(handler.instructions(&payload_bytes, &submission).is_err());
    }

    #[test]
    fn foreign_state_pda_refuses_to_settle() {
        let deployment = deployment_with_settlement();
        let handler: SettleCertificateHandler<()> =
            SettleCertificateHandler::new(&deployment, Keypair::new()).unwrap();
        let foreign_state = Pubkey::new_unique();
        let program_id = deployment.settlement_program_id().unwrap();
        let buffer = find_buffer_program_address(&program_id, &foreign_state, 0).0;
        let payload = payload_for(foreign_state, buffer);
        let payload_bytes = payload.try_to_vec().unwrap();
        let submission = submission_for(&payload);
        assert!(handler.instructions(&payload_bytes, &submission).is_err());
    }
}
