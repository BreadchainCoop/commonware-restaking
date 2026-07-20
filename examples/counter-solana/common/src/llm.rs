//! LLM settlement leg: task payload + node-side validator (INTERFACES.md §4/§6).
//!
//! The task the router broadcasts IS the borsh-encoded `SettlementPayload`
//! produced by the Gas Killer LLM payload producer (`llm-payload-producer`,
//! jito-ncn-program). The digest the quorum signs is
//! `sha256(borsh(SettlementPayload))` — exactly what the settlement program's
//! `Settle` instruction recomputes on-chain — so the node validator hashes the
//! raw task bytes, but only AFTER structurally validating that the payload
//! binds the node's OWN view of the consumer (state PDA derived from its own
//! deployment config, the settle discriminator, exactly one `Store`, and a
//! story_meta event whose buffer is the derived transition buffer PDA).

use anyhow::{Result, anyhow, ensure};
use borsh10::{BorshDeserialize as _, BorshSerialize as _};
use bytes::{Buf, BufMut};
use commonware_avs_core::validator::ValidatorTrait;
use commonware_codec::{EncodeSize, Read, Write};
use commonware_cryptography::sha256::Digest;
use commonware_cryptography::{Hasher, Sha256};
use settlement_core::buffer::find_buffer_program_address;
use settlement_core::instruction::SETTLE_DISCRIMINATOR;
use settlement_core::payload::{
    STORY_META_DISCRIMINANT, SettlementPayload, StateUpdate, StoryMeta,
};
use solana_sdk::pubkey::Pubkey;

/// Application namespace for the LLM settlement leg: p2p handshake domain and
/// `skip_digest` binding. Distinct from the counter demo's namespace so
/// certificates and handshakes can never cross between the two flows.
pub const LLM_APPLICATION_NAMESPACE: &[u8] = b"_COMMONWARE_AGGREGATION_SOLANA_LLM_";

/// Upper bound accepted for a task's borsh payload. Settle payloads are small
/// (the story rides the buffer account, not the task): the fixture payload is
/// ~150 bytes; 64 KiB leaves room for many events while still bounding decode.
const MAX_PAYLOAD_LEN: usize = 64 * 1024;

/// Task payload for the LLM settlement leg: the raw borsh
/// `SettlementPayload` bytes (the preimage of the certified digest).
#[derive(Debug, Clone, PartialEq)]
pub struct LlmTaskData {
    pub payload: Vec<u8>,
}

impl AsRef<[u8]> for LlmTaskData {
    fn as_ref(&self) -> &[u8] {
        &self.payload
    }
}

impl Write for LlmTaskData {
    fn write(&self, buf: &mut impl BufMut) {
        self.payload.write(buf);
    }
}

impl Read for LlmTaskData {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _: &()) -> Result<Self, commonware_codec::Error> {
        let payload = Vec::<u8>::read_cfg(buf, &(((..=MAX_PAYLOAD_LEN).into()), ()))?;
        Ok(Self { payload })
    }
}

impl EncodeSize for LlmTaskData {
    fn encode_size(&self) -> usize {
        self.payload.encode_size()
    }
}

/// `sha256(borsh(SettlementPayload))` as the engine's digest type — the
/// `MessageDigest` the operators certify and `Settle` verifies on-chain.
pub fn llm_payload_digest(payload_bytes: &[u8]) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(payload_bytes);
    hasher.finalize()
}

/// Structural validation shared by the router source (fail fast before
/// sequencing) and the node validator: parses the payload and checks that it
/// binds `(settlement_program, state_pda)`.
pub fn validate_settlement_payload(
    payload_bytes: &[u8],
    settlement_program_id: &Pubkey,
    state_pda: &Pubkey,
) -> Result<SettlementPayload> {
    let payload = SettlementPayload::try_from_slice(payload_bytes)
        .map_err(|e| anyhow!("task is not a borsh SettlementPayload: {e}"))?;
    // Canonical bytes: the digest covers the raw bytes, so insist the decoded
    // payload re-encodes to exactly what was signed over.
    let reencoded = payload
        .try_to_vec()
        .map_err(|e| anyhow!("payload re-serialization: {e}"))?;
    ensure!(reencoded == payload_bytes, "non-canonical payload encoding");

    ensure!(
        payload.ix_discriminator == SETTLE_DISCRIMINATOR,
        "payload does not bind the settle instruction"
    );
    ensure!(
        payload.state_pda == state_pda.to_bytes(),
        "payload state pda {} does not match this deployment's {}",
        hex::encode(payload.state_pda),
        state_pda,
    );

    let mut stores = 0usize;
    for update in &payload.updates {
        match update {
            StateUpdate::Store { .. } => stores += 1,
            StateUpdate::Event {
                discriminant,
                payload: event_payload,
            } => {
                if *discriminant == STORY_META_DISCRIMINANT {
                    let meta = StoryMeta::try_from_slice(event_payload)
                        .map_err(|e| anyhow!("malformed story_meta event: {e}"))?;
                    let (expected_buffer, _, _) = find_buffer_program_address(
                        settlement_program_id,
                        state_pda,
                        payload.transition_index,
                    );
                    ensure!(
                        meta.buffer == expected_buffer,
                        "story_meta buffer {} is not the transition buffer PDA {}",
                        meta.buffer,
                        expected_buffer,
                    );
                }
            }
        }
    }
    ensure!(stores == 1, "expected exactly one Store, got {stores}");
    Ok(payload)
}

/// Node-side validator for [`LlmTaskData`]: validates the payload against the
/// node's OWN deployment binding (never the router's bytes), then signs the
/// payload-bytes digest.
pub struct LlmSettleValidator {
    settlement_program_id: Pubkey,
    state_pda: Pubkey,
}

impl LlmSettleValidator {
    pub fn new(settlement_program_id: Pubkey, state_pda: Pubkey) -> Self {
        Self {
            settlement_program_id,
            state_pda,
        }
    }
}

#[async_trait::async_trait]
impl ValidatorTrait<LlmTaskData> for LlmSettleValidator {
    async fn expected_digest(&self, task: &LlmTaskData) -> Result<Digest> {
        validate_settlement_payload(&task.payload, &self.settlement_program_id, &self.state_pda)?;
        Ok(llm_payload_digest(&task.payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_codec::{DecodeExt, Encode};

    fn sample_payload(program: &Pubkey, state: &Pubkey) -> Vec<u8> {
        let (buffer, _, _) = find_buffer_program_address(program, state, 0);
        let meta = StoryMeta {
            story_sha256: [0x11; 32],
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
        .try_to_vec()
        .unwrap()
    }

    #[test]
    fn task_codec_roundtrip() {
        let task = LlmTaskData {
            payload: vec![1, 2, 3, 4, 5],
        };
        let encoded = task.encode();
        assert_eq!(encoded.len(), task.encode_size());
        assert_eq!(LlmTaskData::decode(encoded).unwrap(), task);
    }

    #[test]
    fn digest_is_sha256_of_the_raw_bytes() {
        // Cross-check against the settlement program's own digest routine.
        let program = Pubkey::new_unique();
        let state = Pubkey::new_unique();
        let bytes = sample_payload(&program, &state);
        let onchain = SettlementPayload::try_from_slice(&bytes)
            .unwrap()
            .digest()
            .unwrap();
        assert_eq!(llm_payload_digest(&bytes).as_ref(), onchain.0.as_slice());
    }

    #[test]
    fn validator_accepts_a_well_bound_payload() {
        let program = Pubkey::new_unique();
        let state = Pubkey::new_unique();
        let bytes = sample_payload(&program, &state);
        validate_settlement_payload(&bytes, &program, &state).expect("valid");
    }

    #[test]
    fn validator_rejects_foreign_state_pda() {
        let program = Pubkey::new_unique();
        let state = Pubkey::new_unique();
        let bytes = sample_payload(&program, &state);
        let err = validate_settlement_payload(&bytes, &program, &Pubkey::new_unique())
            .expect_err("foreign state must fail");
        assert!(err.to_string().contains("state pda"));
    }

    #[test]
    fn validator_rejects_wrong_discriminator_and_missing_store() {
        let program = Pubkey::new_unique();
        let state = Pubkey::new_unique();

        let mut wrong_disc =
            SettlementPayload::try_from_slice(&sample_payload(&program, &state)).unwrap();
        wrong_disc.ix_discriminator = [0; 8];
        let bytes = wrong_disc.try_to_vec().unwrap();
        assert!(validate_settlement_payload(&bytes, &program, &state).is_err());

        let mut no_store =
            SettlementPayload::try_from_slice(&sample_payload(&program, &state)).unwrap();
        no_store
            .updates
            .retain(|u| matches!(u, StateUpdate::Event { .. }));
        let bytes = no_store.try_to_vec().unwrap();
        assert!(validate_settlement_payload(&bytes, &program, &state).is_err());
    }

    #[test]
    fn validator_rejects_wrong_buffer_pda() {
        let program = Pubkey::new_unique();
        let state = Pubkey::new_unique();
        let mut payload =
            SettlementPayload::try_from_slice(&sample_payload(&program, &state)).unwrap();
        // Point the story_meta at a different transition's buffer.
        let (wrong_buffer, _, _) = find_buffer_program_address(&program, &state, 7);
        for update in &mut payload.updates {
            if let StateUpdate::Event {
                discriminant,
                payload: event_payload,
            } = update
            {
                if *discriminant == STORY_META_DISCRIMINANT {
                    let mut meta = StoryMeta::try_from_slice(event_payload).unwrap();
                    meta.buffer = wrong_buffer;
                    *event_payload = meta.try_to_vec().unwrap();
                }
            }
        }
        let bytes = payload.try_to_vec().unwrap();
        let err = validate_settlement_payload(&bytes, &program, &state)
            .expect_err("wrong buffer must fail");
        assert!(err.to_string().contains("buffer"));
    }

    #[test]
    fn validator_rejects_trailing_garbage() {
        let program = Pubkey::new_unique();
        let state = Pubkey::new_unique();
        let mut bytes = sample_payload(&program, &state);
        bytes.push(0);
        assert!(validate_settlement_payload(&bytes, &program, &state).is_err());
    }
}
