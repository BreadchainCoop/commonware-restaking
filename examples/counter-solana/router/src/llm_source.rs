//! One-shot [`TaskSource`] for the LLM settlement leg.
//!
//! Loads the payload fixture emitted by `llm-payload-producer`
//! (jito-ncn-program, INTERFACES.md §6): a JSON document whose
//! `payload_borsh_base64` is the borsh `SettlementPayload` and whose
//! `digest_hex` is `sha256` of those bytes. The source re-derives the digest
//! from the bytes (and structurally validates the payload against THIS
//! deployment's settlement binding) before sequencing exactly one task; after
//! that it parks forever — the LLM leg settles a single transition.

use anyhow::{Context as _, Result, ensure};
use async_trait::async_trait;
use base64::Engine as _;
use commonware_avs_router::sequencer::{SequencedTask, TaskSource};
use counter_solana_common::{LlmTaskData, llm_payload_digest, validate_settlement_payload};
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use std::path::Path;
use std::time::Duration;

/// The subset of the producer fixture the router consumes.
#[derive(Debug, Deserialize)]
struct ProducerFixture {
    payload_borsh_base64: String,
    digest_hex: String,
}

/// Yields the fixture's settlement payload exactly once.
pub struct LlmSource {
    task: Option<SequencedTask<LlmTaskData>>,
}

impl LlmSource {
    /// Loads and cross-checks the producer fixture. `settlement_program_id`
    /// and `state_pda` come from the router's own deployment config.
    pub fn from_fixture(
        path: &Path,
        settlement_program_id: &Pubkey,
        state_pda: &Pubkey,
    ) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("reading LLM payload fixture {}", path.display()))?;
        let fixture: ProducerFixture =
            serde_json::from_str(&contents).context("parsing LLM payload fixture")?;
        let payload = base64::engine::general_purpose::STANDARD
            .decode(&fixture.payload_borsh_base64)
            .context("payload_borsh_base64 is not valid base64")?;

        // The payload must bind THIS deployment before it is ever announced.
        validate_settlement_payload(&payload, settlement_program_id, state_pda)?;

        // The digest we sequence is recomputed from the bytes; the fixture's
        // digest_hex must agree (a mismatch means a stale or tampered fixture).
        let digest = llm_payload_digest(&payload);
        ensure!(
            hex::encode(digest.as_ref()) == fixture.digest_hex.to_lowercase(),
            "fixture digest_hex {} does not match sha256(payload) {}",
            fixture.digest_hex,
            hex::encode(digest.as_ref()),
        );

        let task = LlmTaskData { payload };
        Ok(Self {
            task: Some(SequencedTask {
                announce: task.clone(),
                digest,
                task,
            }),
        })
    }

    /// The digest of the loaded payload (for logging).
    pub fn digest(&self) -> Option<commonware_cryptography::sha256::Digest> {
        self.task.as_ref().map(|t| t.digest)
    }
}

#[async_trait]
impl TaskSource<LlmTaskData> for LlmSource {
    async fn next_task(&mut self) -> Option<SequencedTask<LlmTaskData>> {
        if let Some(task) = self.task.take() {
            return Some(task);
        }
        // One transition per run: park forever (returning None would shut the
        // sequencer down while the submitter may still be finalizing).
        loop {
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh10::BorshSerialize as _;
    use settlement_core::buffer::find_buffer_program_address;
    use settlement_core::instruction::SETTLE_DISCRIMINATOR;
    use settlement_core::payload::{
        STORY_META_DISCRIMINANT, SettlementPayload, StateUpdate, StoryMeta,
    };

    fn write_fixture(dir: &Path, payload: &[u8], digest_hex: &str) -> std::path::PathBuf {
        let path = dir.join("llm_payload.json");
        let json = serde_json::json!({
            "prompt": "Once upon a time",
            "story_utf8": "irrelevant here",
            "story_sha256_hex": "00",
            "payload_borsh_base64": base64::engine::general_purpose::STANDARD.encode(payload),
            "digest_hex": digest_hex,
            "source": {"sim_command": "x", "solidity_sdk_commit": "y"},
        });
        std::fs::write(&path, serde_json::to_string(&json).unwrap()).unwrap();
        path
    }

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

    #[tokio::test]
    async fn yields_the_fixture_task_exactly_once() {
        let dir = std::env::temp_dir().join(format!("llm-source-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let program = Pubkey::new_unique();
        let state = Pubkey::new_unique();
        let payload = sample_payload(&program, &state);
        let digest = llm_payload_digest(&payload);
        let path = write_fixture(&dir, &payload, &hex::encode(digest.as_ref()));

        let mut source = LlmSource::from_fixture(&path, &program, &state).expect("fixture loads");
        let task = source.next_task().await.expect("one task");
        assert_eq!(task.digest, digest);
        assert_eq!(task.task.payload, payload);
        assert_eq!(task.announce, task.task);

        // The second pull parks: give it a moment and assert it is pending.
        let pending = tokio::time::timeout(Duration::from_millis(50), source.next_task()).await;
        assert!(pending.is_err(), "source must park after the single task");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_a_digest_mismatch_and_foreign_binding() {
        let dir = std::env::temp_dir().join(format!("llm-source-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let program = Pubkey::new_unique();
        let state = Pubkey::new_unique();
        let payload = sample_payload(&program, &state);

        let path = write_fixture(&dir, &payload, &hex::encode([0u8; 32]));
        assert!(LlmSource::from_fixture(&path, &program, &state).is_err());

        let digest = llm_payload_digest(&payload);
        let path = write_fixture(&dir, &payload, &hex::encode(digest.as_ref()));
        assert!(LlmSource::from_fixture(&path, &program, &Pubkey::new_unique()).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
