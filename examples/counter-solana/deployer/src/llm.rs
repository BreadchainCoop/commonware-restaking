//! LLM settlement leg of the e2e stack (INTERFACES.md §4/§6): drives the
//! gaskiller-settlement program around the router/committee flow.
//!
//! - `llm-init`: `InitializeState` for the demo consumer app
//!   (`app_id = sha256("gaskiller-llm-demo")`, profile/env pins hashed from
//!   the checked-in fixture's source metadata), then emits every artifact the
//!   rest of the leg needs (producer-CLI inputs, patched deployment config,
//!   `frontend-config.json`).
//! - `llm-stage`: writes the story bytes into the transition's buffer PDA in
//!   MULTIPLE `WriteBuffer` chunks (the story fits one transaction; chunking
//!   proves the append path) and verifies the staged content hash.
//! - `llm-assert`: after the router lands `Settle`, asserts at `confirmed`:
//!   `commitment_root` == the payload's Store, `transition_count` == 1, the
//!   buffer hashes to `story_sha256` (and STAYS OPEN — the frontend reads it),
//!   the `story_meta` self-CPI is present in the settle transaction, and
//!   prints the story read back from chain.
//! - `llm-replay`: re-submits the EXACT settle args recovered from the landed
//!   transaction and asserts the program rejects them with
//!   `InvalidTransitionIndex` (0x9100) — the consumer-local replay gate.

use anyhow::{anyhow, bail, Context, Result};
use borsh::BorshDeserialize;
use jito_bytemuck::AccountDeserialize;
use ncn_program_core::config::Config as NcnConfig;
use ncn_program_core::snapshot::Snapshot;
use serde::{Deserialize, Serialize};
use settlement_core::buffer::{buffer_content_len, find_buffer_program_address};
use settlement_core::instruction::{
    initialize_state_ix, settle_ix, write_buffer_ix, InitializeStateArgs, SettleArgs,
    WriteBufferArgs, SETTLE_DISCRIMINATOR,
};
use settlement_core::payload::{
    SettlementPayload, StateUpdate, StoryMeta, STORY_META_DISCRIMINANT,
};
use settlement_core::state::GkState;
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Signature, Signer};
use solana_transaction_status_client_types::{UiInstruction, UiTransactionEncoding};
use std::path::Path;
use std::str::FromStr;
use std::time::{Duration, Instant};

use crate::{send, write_json};

/// The gaskiller-settlement program id (its `declare_id!`).
pub const DEFAULT_SETTLEMENT_PROGRAM_ID: &str = "6XTdBk798fEpM2VPBXpkLPw4zJJLvASaiyHaEmj9Ripx";

/// The demo consumer's application id seed: `app_id = sha256(seed)`.
pub const APP_ID_SEED: &[u8] = b"gaskiller-llm-demo";

/// `SettlementError::InvalidTransitionIndex` (settlement_core `error.rs`).
const INVALID_TRANSITION_INDEX: u32 = 0x9100;

fn sha256(bytes: &[u8]) -> [u8; 32] {
    solana_sdk::hash::hash(bytes).to_bytes()
}

// ---------------------------------------------------------------------------
// Producer fixture (llm-payload-producer JSON, §6 frozen field names)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct FixtureSource {
    pub sim_command: String,
    pub solidity_sdk_commit: String,
}

#[derive(Debug, Deserialize)]
pub struct ProducerFixture {
    pub prompt: String,
    pub story_utf8: String,
    pub story_sha256_hex: String,
    pub payload_borsh_base64: String,
    pub digest_hex: String,
    pub source: FixtureSource,
}

/// A fixture cross-checked against itself: payload decoded, digest and story
/// hash recomputed, the single `Store` and the `story_meta` event extracted.
pub struct VerifiedFixture {
    pub fixture: ProducerFixture,
    pub payload: SettlementPayload,
    pub new_root: [u8; 32],
    pub story_meta: StoryMeta,
}

pub fn load_fixture(path: &Path) -> Result<VerifiedFixture> {
    use base64::Engine as _;
    let fixture: ProducerFixture = serde_json::from_str(
        &std::fs::read_to_string(path)
            .with_context(|| format!("reading fixture {}", path.display()))?,
    )
    .with_context(|| format!("parsing fixture {}", path.display()))?;

    let payload_bytes = base64::engine::general_purpose::STANDARD
        .decode(&fixture.payload_borsh_base64)
        .context("payload_borsh_base64")?;
    let payload = SettlementPayload::try_from_slice(&payload_bytes).context("payload borsh")?;

    let digest = sha256(&payload_bytes);
    if hex::encode(digest) != fixture.digest_hex.to_lowercase() {
        bail!(
            "fixture digest_hex {} != sha256(payload) {}",
            fixture.digest_hex,
            hex::encode(digest)
        );
    }
    let story_hash = sha256(fixture.story_utf8.as_bytes());
    if hex::encode(story_hash) != fixture.story_sha256_hex.to_lowercase() {
        bail!("fixture story_sha256_hex does not match sha256(story_utf8)");
    }

    let mut new_root = None;
    let mut story_meta = None;
    for update in &payload.updates {
        match update {
            StateUpdate::Store { data } => {
                if new_root.replace(*data).is_some() {
                    bail!("fixture payload has more than one Store");
                }
            }
            StateUpdate::Event {
                discriminant,
                payload: event_payload,
            } => {
                if *discriminant == STORY_META_DISCRIMINANT {
                    let meta = StoryMeta::try_from_slice(event_payload).context("story_meta")?;
                    if meta.story_sha256 != story_hash {
                        bail!("story_meta.story_sha256 does not match the story bytes");
                    }
                    if meta.len as usize != fixture.story_utf8.len() {
                        bail!("story_meta.len does not match the story length");
                    }
                    story_meta = Some(meta);
                }
            }
        }
    }
    Ok(VerifiedFixture {
        new_root: new_root.ok_or_else(|| anyhow!("fixture payload has no Store"))?,
        story_meta: story_meta.ok_or_else(|| anyhow!("fixture payload has no story_meta event"))?,
        fixture,
        payload,
    })
}

// ---------------------------------------------------------------------------
// Shared config plumbing
// ---------------------------------------------------------------------------

/// llm.json — the leg's own state, written by `llm-init`.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LlmState {
    settlement_program_id: String,
    state_pda: String,
    buffer_pda: String,
    app_id_hex: String,
    sim_profile_id_hex: String,
    env_commitment_hex: String,
}

fn read_llm_state(out: &Path) -> Result<LlmState> {
    Ok(serde_json::from_str(
        &std::fs::read_to_string(out.join("llm.json")).context("llm.json (run llm-init first)")?,
    )?)
}

/// The counter deployer's stack state (state.json), reused for the NCN keys.
fn read_stack_state(out: &Path) -> Result<crate::StackState> {
    Ok(serde_json::from_str(
        &std::fs::read_to_string(out.join("state.json"))
            .context("state.json (run deploy first)")?,
    )?)
}

fn read_gk_state(client: &RpcClient, state_pda: &Pubkey) -> Result<GkState> {
    let data = client
        .get_account_data(state_pda)
        .with_context(|| format!("gk state account {state_pda}"))?;
    let state =
        GkState::try_from_slice_unchecked(&data).map_err(|e| anyhow!("decode GkState: {e:?}"))?;
    Ok(*state)
}

/// POSIX single-quote escaping for llm_env.sh values.
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r#"'\''"#))
}

// ---------------------------------------------------------------------------
// llm-init
// ---------------------------------------------------------------------------

pub fn llm_init(
    client: &RpcClient,
    rpc_url: &str,
    out: &Path,
    fixture_path: &Path,
    settlement_program_id: &Pubkey,
) -> Result<()> {
    println!("== llm-init: settlement consumer bootstrap ==");
    let stack = read_stack_state(out)?;
    let ncn = Pubkey::from_str(&stack.ncn)?;
    let authority = read_keypair_file(out.join("authority.json"))
        .map_err(|e| anyhow!("read authority: {e}"))?;
    let verified = load_fixture(fixture_path)?;

    // The consumer identity + environment pins, all derived from public
    // material: the app id from a fixed seed, the profile/env pins from the
    // fixture's PROVENANCE metadata (the pinned solidity-sdk commit and the
    // exact simulation command that produced the story).
    let app_id = sha256(APP_ID_SEED);
    let sim_profile_id = sha256(verified.fixture.source.solidity_sdk_commit.as_bytes());
    let env_commitment = sha256(verified.fixture.source.sim_command.as_bytes());

    let (state_pda, _, _) = GkState::find_program_address(settlement_program_id, &ncn, &app_id);
    let (buffer_pda, _, _) = find_buffer_program_address(settlement_program_id, &state_pda, 0);
    println!("settlement program: {settlement_program_id}");
    println!("state pda:          {state_pda}");
    println!("buffer pda (t=0):   {buffer_pda}");

    send(
        client,
        "settlement InitializeState",
        &[initialize_state_ix(
            settlement_program_id,
            &state_pda,
            &ncn,
            &authority.pubkey(),
            &InitializeStateArgs {
                app_id,
                sim_profile_id,
                env_commitment,
            },
        )
        .map_err(|e| anyhow!("initialize_state ix: {e:?}"))?],
        &authority,
        &[],
    )?;

    // Read back and verify the created state.
    let state = read_gk_state(client, &state_pda)?;
    if state.transition_count() != 0
        || state.app_id() != &app_id
        || state.sim_profile_id() != &sim_profile_id
        || state.env_commitment() != &env_commitment
        || state.ncn() != &ncn
    {
        bail!("on-chain GkState does not match the initialize arguments");
    }
    println!("  gk_state verified on-chain (transition_count=0)");

    // Patch the shared deployment config with the settlement binding: nodes
    // derive the state PDA from THEIR OWN copy of this file.
    let deploy_path = out.join("ncn_deploy.json");
    let mut deploy: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&deploy_path)?)?;
    deploy["settlementProgramId"] = serde_json::Value::String(settlement_program_id.to_string());
    deploy["appId"] = serde_json::Value::String(hex::encode(app_id));
    std::fs::write(&deploy_path, serde_json::to_string_pretty(&deploy)?)?;
    println!("  ncn_deploy.json patched with the settlement binding");

    // Frontend artifact (§4 retain-until-indexed: the frontend reads the
    // story straight from the buffer account).
    write_json(
        &out.join("frontend-config.json"),
        &serde_json::json!({
            "rpcUrl": rpc_url,
            "ncnProgramId": stack.ncn_program_id,
            "settlementProgramId": settlement_program_id.to_string(),
            "statePda": state_pda.to_string(),
            "cluster": "localnet",
            "commitment": "confirmed",
        }),
    )?;
    println!("  frontend-config.json written");

    // The leg's own state + the producer CLI inputs.
    write_json(
        &out.join("llm.json"),
        &LlmState {
            settlement_program_id: settlement_program_id.to_string(),
            state_pda: state_pda.to_string(),
            buffer_pda: buffer_pda.to_string(),
            app_id_hex: hex::encode(app_id),
            sim_profile_id_hex: hex::encode(sim_profile_id),
            env_commitment_hex: hex::encode(env_commitment),
        },
    )?;
    std::fs::write(
        out.join("story.txt"),
        verified.fixture.story_utf8.as_bytes(),
    )?;

    let env_sh = format!(
        "# generated by counter-solana-deployer llm-init\n\
         export LLM_STATE_PDA={}\n\
         export LLM_STATE_PDA_HEX={}\n\
         export LLM_BUFFER_PDA={}\n\
         export LLM_BUFFER_PDA_HEX={}\n\
         export LLM_SETTLE_DISC_HEX={}\n\
         export LLM_NEW_ROOT_HEX={}\n\
         export LLM_PROMPT={}\n\
         export LLM_SIM_COMMAND={}\n\
         export LLM_SDK_COMMIT={}\n\
         export LLM_CHECKED_IN_DIGEST_HEX={}\n\
         export LLM_STORY_SHA256_HEX={}\n",
        state_pda,
        hex::encode(state_pda.to_bytes()),
        buffer_pda,
        hex::encode(buffer_pda.to_bytes()),
        hex::encode(SETTLE_DISCRIMINATOR),
        hex::encode(verified.new_root),
        shell_quote(&verified.fixture.prompt),
        shell_quote(&verified.fixture.source.sim_command),
        shell_quote(&verified.fixture.source.solidity_sdk_commit),
        verified.fixture.digest_hex,
        verified.fixture.story_sha256_hex,
    );
    std::fs::write(out.join("llm_env.sh"), env_sh)?;
    println!("  llm_env.sh + llm.json + story.txt written");
    println!("== llm-init complete ==");
    Ok(())
}

// ---------------------------------------------------------------------------
// llm-stage
// ---------------------------------------------------------------------------

pub fn llm_stage(
    client: &RpcClient,
    out: &Path,
    payload_path: &Path,
    chunk_size: usize,
) -> Result<()> {
    println!("== llm-stage: WriteBuffer the story bytes (chunked) ==");
    let llm = read_llm_state(out)?;
    let settlement_pid = Pubkey::from_str(&llm.settlement_program_id)?;
    let authority = read_keypair_file(out.join("authority.json"))
        .map_err(|e| anyhow!("read authority: {e}"))?;

    let verified = load_fixture(payload_path)?;
    let state_pda = Pubkey::new_from_array(verified.payload.state_pda);
    if state_pda.to_string() != llm.state_pda {
        bail!(
            "payload state pda {state_pda} != initialized state pda {} — regenerate the payload",
            llm.state_pda
        );
    }
    let transition_index = verified.payload.transition_index;
    let (buffer_pda, _, _) =
        find_buffer_program_address(&settlement_pid, &state_pda, transition_index);
    if buffer_pda != verified.story_meta.buffer {
        bail!(
            "derived buffer {buffer_pda} != story_meta buffer {} — regenerate the payload",
            verified.story_meta.buffer
        );
    }

    let story = verified.fixture.story_utf8.as_bytes();
    if chunk_size == 0 || chunk_size >= story.len() {
        bail!(
            "chunk-size {chunk_size} must be in (0, {}) to exercise the multi-chunk path",
            story.len()
        );
    }
    let chunks: Vec<&[u8]> = story.chunks(chunk_size).collect();
    println!(
        "staging {} story bytes into {buffer_pda} in {} chunks of <= {chunk_size} B",
        story.len(),
        chunks.len()
    );
    let mut offset = 0u32;
    for (i, chunk) in chunks.iter().enumerate() {
        send(
            client,
            &format!(
                "settlement WriteBuffer[{i}] offset={offset} len={}",
                chunk.len()
            ),
            &[write_buffer_ix(
                &settlement_pid,
                &state_pda,
                &buffer_pda,
                &authority.pubkey(),
                &WriteBufferArgs {
                    transition_index,
                    offset,
                    bytes: chunk.to_vec(),
                    max_size: story.len() as u32,
                },
            )
            .map_err(|e| anyhow!("write_buffer ix: {e:?}"))?],
            &authority,
            &[],
        )?;
        offset += chunk.len() as u32;
    }

    // Read the staged content back and verify the digest the settle will check.
    let data = client.get_account_data(&buffer_pda)?;
    let content_len =
        buffer_content_len(data.len()).map_err(|e| anyhow!("buffer bounds: {e:?}"))?;
    let len = verified.story_meta.len as usize;
    if len > content_len {
        bail!("staged buffer content region {content_len} < story len {len}");
    }
    let staged_hash = sha256(&data[..len]);
    if staged_hash != verified.story_meta.story_sha256 {
        bail!(
            "staged buffer hash {} != story_sha256 {}",
            hex::encode(staged_hash),
            hex::encode(verified.story_meta.story_sha256)
        );
    }
    println!(
        "== llm-stage complete: sha256(buffer[..{len}]) == story_sha256 ({}) ==",
        hex::encode(staged_hash)
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// llm-assert
// ---------------------------------------------------------------------------

/// A landed settle transaction: signature, the settle instruction's data,
/// and the datas of the self-CPI inner instructions (the emitted events).
type SettleTx = (Signature, Vec<u8>, Vec<Vec<u8>>);

/// Finds the (successful) settle transaction touching `state_pda`.
fn find_settle_tx(
    client: &RpcClient,
    settlement_pid: &Pubkey,
    state_pda: &Pubkey,
) -> Result<Option<SettleTx>> {
    let sigs = client.get_signatures_for_address_with_config(
        state_pda,
        GetConfirmedSignaturesForAddress2Config {
            before: None,
            until: None,
            limit: Some(50),
            commitment: Some(CommitmentConfig::confirmed()),
        },
    )?;
    for entry in &sigs {
        if entry.err.is_some() {
            continue;
        }
        let sig = Signature::from_str(&entry.signature)?;
        let tx = client.get_transaction_with_config(
            &sig,
            solana_client::rpc_config::RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Base64),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )?;
        let Some(decoded) = tx.transaction.transaction.decode() else {
            continue;
        };
        let keys = decoded.message.static_account_keys();
        let settle_data = decoded.message.instructions().iter().find_map(|ix| {
            (keys.get(ix.program_id_index as usize) == Some(settlement_pid)
                && ix.data.get(..8) == Some(&SETTLE_DISCRIMINATOR))
            .then(|| ix.data.clone())
        });
        let Some(settle_data) = settle_data else {
            continue;
        };
        // Collect the self-CPI inner instruction datas addressed to the
        // settlement program (the emitted events).
        let mut events = Vec::new();
        if let Some(meta) = tx.transaction.meta.as_ref() {
            if meta.err.is_some() {
                continue;
            }
            let inner: Vec<_> = Option::from(meta.inner_instructions.clone()).unwrap_or_default();
            for group in inner {
                for ix in group.instructions {
                    let UiInstruction::Compiled(compiled) = ix else {
                        continue;
                    };
                    if keys.get(compiled.program_id_index as usize) != Some(settlement_pid) {
                        continue;
                    }
                    let data = bs58::decode(&compiled.data)
                        .into_vec()
                        .context("inner instruction data bs58")?;
                    events.push(data);
                }
            }
        }
        return Ok(Some((sig, settle_data, events)));
    }
    Ok(None)
}

pub fn llm_assert(
    client: &RpcClient,
    out: &Path,
    payload_path: &Path,
    timeout_secs: u64,
) -> Result<()> {
    println!("== llm-assert: settle outcome on-chain (confirmed) ==");
    let llm = read_llm_state(out)?;
    let settlement_pid = Pubkey::from_str(&llm.settlement_program_id)?;
    let verified = load_fixture(payload_path)?;
    let state_pda = Pubkey::new_from_array(verified.payload.state_pda);
    let expected_count = verified.payload.transition_index + 1;

    // 1. Wait for the transition counter to advance.
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        match read_gk_state(client, &state_pda) {
            Ok(state) if state.transition_count() >= expected_count => break,
            Ok(state) => {
                println!(
                    "  waiting: transition_count={} (want {expected_count})",
                    state.transition_count()
                );
            }
            Err(e) => println!("  waiting: {e}"),
        }
        if Instant::now() > deadline {
            bail!("settle did not land within {timeout_secs}s");
        }
        std::thread::sleep(Duration::from_secs(2));
    }

    // 2. State: exactly one transition, root == the payload's Store.
    let state = read_gk_state(client, &state_pda)?;
    if state.transition_count() != expected_count {
        bail!(
            "transition_count {} != expected {expected_count}",
            state.transition_count()
        );
    }
    if state.commitment_root() != &verified.new_root {
        bail!(
            "commitment_root {} != fixture root {}",
            hex::encode(state.commitment_root()),
            hex::encode(verified.new_root)
        );
    }
    println!(
        "  commitment_root == fixture root ({}), transition_count == {expected_count}",
        hex::encode(verified.new_root)
    );

    // 3. Buffer: still open (retain-until-indexed) and hashes to story_sha256.
    let buffer = verified.story_meta.buffer;
    let data = client
        .get_account_data(&buffer)
        .context("buffer account must remain OPEN after settle (frontend reads it)")?;
    let len = verified.story_meta.len as usize;
    let content_len =
        buffer_content_len(data.len()).map_err(|e| anyhow!("buffer bounds: {e:?}"))?;
    if len > content_len {
        bail!("buffer content region {content_len} < story len {len}");
    }
    let staged_hash = sha256(&data[..len]);
    if staged_hash != verified.story_meta.story_sha256 {
        bail!("buffer hash does not match story_sha256 after settle");
    }
    println!("  buffer OPEN, sha256(buffer[..{len}]) == story_sha256");

    // 4. The settle transaction carries the story_meta self-CPI event.
    let (sig, settle_data, events) = find_settle_tx(client, &settlement_pid, &state_pda)?
        .ok_or_else(|| anyhow!("no successful settle transaction found on {state_pda}"))?;
    let args = SettleArgs::try_from_slice(&settle_data[8..]).context("landed settle args")?;
    if args.payload != verified.payload {
        bail!("landed settle payload does not match the fixture payload");
    }
    let story_meta_event = events
        .iter()
        .find(|data| data.get(..8) == Some(&STORY_META_DISCRIMINANT))
        .ok_or_else(|| anyhow!("settle tx has no story_meta self-CPI inner instruction"))?;
    let event_meta =
        StoryMeta::try_from_slice(&story_meta_event[8..]).context("story_meta event payload")?;
    if event_meta != verified.story_meta {
        bail!("emitted story_meta does not match the fixture story_meta");
    }
    println!("  story_meta self-CPI event present in settle tx {sig}");
    std::fs::write(out.join("llm_settle_tx.txt"), sig.to_string())?;

    // 5. The story, read back from chain.
    let story = std::str::from_utf8(&data[..len]).context("story is not UTF-8")?;
    println!("--- story read back from chain ({len} bytes) ---");
    println!("{story}");
    println!("--- end story ---");
    println!(
        "== llm-assert PASSED: settle tx {sig}, digest {} ==",
        verified.fixture.digest_hex
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// llm-replay
// ---------------------------------------------------------------------------

pub fn llm_replay(client: &RpcClient, out: &Path) -> Result<()> {
    println!("== llm-replay: resubmitting the landed Settle (must fail 0x9100) ==");
    let llm = read_llm_state(out)?;
    let stack = read_stack_state(out)?;
    let settlement_pid = Pubkey::from_str(&llm.settlement_program_id)?;
    let state_pda = Pubkey::from_str(&llm.state_pda)?;
    let authority = read_keypair_file(out.join("authority.json"))
        .map_err(|e| anyhow!("read authority: {e}"))?;

    let (sig, settle_data, _) = find_settle_tx(client, &settlement_pid, &state_pda)?
        .ok_or_else(|| anyhow!("no successful settle transaction found on {state_pda}"))?;
    println!("  recovered settle args from tx {sig}");
    let args = SettleArgs::try_from_slice(&settle_data[8..]).context("landed settle args")?;

    // Rebuild the identical instruction against the same accounts.
    let ncn_pid = Pubkey::from_str(&stack.ncn_program_id)?;
    let ncn = Pubkey::from_str(&stack.ncn)?;
    let restaking_pid = Pubkey::from_str(&stack.restaking_program_id)?;
    let ncn_config = NcnConfig::find_program_address(&ncn_pid, &ncn).0;
    let snapshot = Snapshot::find_program_address(&ncn_pid, &ncn).0;
    let restaking_config =
        jito_restaking_core::config::Config::find_program_address(&restaking_pid).0;
    let (buffer, _, _) =
        find_buffer_program_address(&settlement_pid, &state_pda, args.payload.transition_index);
    let ix = settle_ix(
        &settlement_pid,
        &state_pda,
        &ncn_config,
        &ncn,
        &snapshot,
        &restaking_config,
        Some(&buffer),
        &args,
    )
    .map_err(|e| anyhow!("settle ix rebuild: {e:?}"))?;

    match send(
        client,
        "settlement Settle (replay)",
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
            ix,
        ],
        &authority,
        &[],
    ) {
        Ok(sig) => bail!("REPLAY UNEXPECTEDLY SUCCEEDED (tx {sig}) — replay gate is broken"),
        Err(e) => {
            let msg = e.to_string();
            let code_hex = format!("{INVALID_TRANSITION_INDEX:#x}");
            if msg.contains(&code_hex) {
                println!("  replay rejected with InvalidTransitionIndex ({code_hex})");
                println!("== llm-replay PASSED ==");
                Ok(())
            } else {
                bail!("replay failed with an unexpected error (want {code_hex}): {msg}")
            }
        }
    }
}

// ===========================================================================
// §8 — Qwen answer settlement leg (real-model demo; supersedes the story leg).
//
// The settlement program is model-agnostic: it settles a digest + diff + event.
// The Qwen payload is `[Store{commitment_root}, Event{qwen_answer}]` — the
// answer token ids ride the event inline (no buffer, no story staging). This
// module reuses the story leg's on-chain plumbing (InitializeState, GkState
// reads, settle-tx recovery, the replay gate) and only swaps the payload the
// producer feeds and the asserts (answer_ids read straight from the event).
// ===========================================================================

use borsh::BorshSerialize as _;

/// The demo Qwen consumer's application id seed (distinct from the story leg).
pub const QWEN_APP_ID_SEED: &[u8] = b"gaskiller-qwen-demo";

/// `sha256("gk:qwen_answer")[..8]` — the §8 Qwen answer event discriminant.
pub const QWEN_ANSWER_DISCRIMINANT: [u8; 8] = [0x2d, 0x80, 0x95, 0x5b, 0xd7, 0x65, 0x66, 0x84];

/// The §8 `QwenAnswer` event payload (borsh; must match the producer + browser).
#[derive(Debug, Clone, PartialEq, Eq, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub struct QwenAnswer {
    pub model: u8,
    pub prompt_ids: Vec<u32>,
    pub answer_ids: Vec<u32>,
    pub manifest: [u8; 32],
}

/// Provenance of a Qwen run (§8 frozen field names).
#[derive(Debug, Deserialize)]
pub struct QwenFixtureSource {
    pub cmd: String,
    pub sdk_commit: String,
}

/// The §8 Qwen producer fixture (frozen field names; Track Q3 decodes it).
#[derive(Debug, Deserialize)]
pub struct QwenProducerFixture {
    pub prompt: String,
    pub prompt_ids: Vec<u32>,
    pub answer_ids: Vec<u32>,
    pub answer_text: String,
    pub commitment_root: String,
    pub manifest: String,
    pub payload_borsh_base64: String,
    pub digest_hex: String,
    pub source: QwenFixtureSource,
}

/// A Qwen fixture cross-checked against itself.
pub struct VerifiedQwenFixture {
    pub fixture: QwenProducerFixture,
    pub payload: SettlementPayload,
    pub new_root: [u8; 32],
    pub answer: QwenAnswer,
}

/// Decode + self-verify a Qwen fixture: digest recompute, canonical borsh,
/// exactly one `Store` equal to `commitment_root`, and a `qwen_answer` event
/// whose ids/manifest match the top-level fields.
pub fn load_qwen_fixture(path: &Path) -> Result<VerifiedQwenFixture> {
    use base64::Engine as _;
    let fixture: QwenProducerFixture = serde_json::from_str(
        &std::fs::read_to_string(path)
            .with_context(|| format!("reading qwen fixture {}", path.display()))?,
    )
    .with_context(|| format!("parsing qwen fixture {}", path.display()))?;

    let payload_bytes = base64::engine::general_purpose::STANDARD
        .decode(&fixture.payload_borsh_base64)
        .context("payload_borsh_base64")?;
    let payload = SettlementPayload::try_from_slice(&payload_bytes).context("payload borsh")?;
    ensure_canonical(&payload, &payload_bytes)?;

    let digest = sha256(&payload_bytes);
    if hex::encode(digest) != fixture.digest_hex.to_lowercase() {
        bail!(
            "fixture digest_hex {} != sha256(payload) {}",
            fixture.digest_hex,
            hex::encode(digest)
        );
    }

    let mut new_root = None;
    let mut answer = None;
    for update in &payload.updates {
        match update {
            StateUpdate::Store { data } => {
                if new_root.replace(*data).is_some() {
                    bail!("qwen fixture payload has more than one Store");
                }
            }
            StateUpdate::Event {
                discriminant,
                payload: event_payload,
            } => {
                if *discriminant == QWEN_ANSWER_DISCRIMINANT {
                    let decoded =
                        QwenAnswer::try_from_slice(event_payload).context("qwen_answer event")?;
                    if decoded.prompt_ids != fixture.prompt_ids {
                        bail!("qwen_answer prompt_ids do not match the fixture");
                    }
                    if decoded.answer_ids != fixture.answer_ids {
                        bail!("qwen_answer answer_ids do not match the fixture");
                    }
                    if hex::encode(decoded.manifest) != fixture.manifest.to_lowercase() {
                        bail!("qwen_answer manifest does not match the fixture");
                    }
                    answer = Some(decoded);
                } else {
                    bail!("unexpected event discriminant in qwen payload");
                }
            }
        }
    }
    let new_root = new_root.ok_or_else(|| anyhow!("qwen fixture payload has no Store"))?;
    if hex::encode(new_root) != fixture.commitment_root.to_lowercase() {
        bail!("Store data != fixture commitment_root");
    }
    Ok(VerifiedQwenFixture {
        answer: answer.ok_or_else(|| anyhow!("qwen fixture has no qwen_answer event"))?,
        new_root,
        fixture,
        payload,
    })
}

fn ensure_canonical(payload: &SettlementPayload, bytes: &[u8]) -> Result<()> {
    let reencoded = payload
        .try_to_vec()
        .map_err(|e| anyhow!("payload re-serialize: {e:?}"))?;
    anyhow::ensure!(reencoded == bytes, "non-canonical payload encoding");
    Ok(())
}

/// llm-init for the Qwen leg: InitializeState + emit the producer-regen env,
/// the frontend config (model=qwen + answer-event coordinates), and llm.json.
pub fn qwen_init(
    client: &RpcClient,
    rpc_url: &str,
    out: &Path,
    fixture_path: &Path,
    settlement_program_id: &Pubkey,
) -> Result<()> {
    println!("== qwen-init: settlement consumer bootstrap (Qwen answer) ==");
    let stack = read_stack_state(out)?;
    let ncn = Pubkey::from_str(&stack.ncn)?;
    let authority = read_keypair_file(out.join("authority.json"))
        .map_err(|e| anyhow!("read authority: {e}"))?;
    let verified = load_qwen_fixture(fixture_path)?;

    let app_id = sha256(QWEN_APP_ID_SEED);
    let sim_profile_id = sha256(verified.fixture.source.sdk_commit.as_bytes());
    let env_commitment = sha256(verified.fixture.source.cmd.as_bytes());

    let (state_pda, _, _) = GkState::find_program_address(settlement_program_id, &ncn, &app_id);
    println!("settlement program: {settlement_program_id}");
    println!("state pda:          {state_pda}");

    send(
        client,
        "settlement InitializeState (qwen)",
        &[initialize_state_ix(
            settlement_program_id,
            &state_pda,
            &ncn,
            &authority.pubkey(),
            &InitializeStateArgs {
                app_id,
                sim_profile_id,
                env_commitment,
            },
        )
        .map_err(|e| anyhow!("initialize_state ix: {e:?}"))?],
        &authority,
        &[],
    )?;

    let state = read_gk_state(client, &state_pda)?;
    if state.transition_count() != 0
        || state.app_id() != &app_id
        || state.sim_profile_id() != &sim_profile_id
        || state.env_commitment() != &env_commitment
        || state.ncn() != &ncn
    {
        bail!("on-chain GkState does not match the initialize arguments");
    }
    println!("  gk_state verified on-chain (transition_count=0)");

    // Patch the shared deployment config with the settlement binding.
    let deploy_path = out.join("ncn_deploy.json");
    let mut deploy: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&deploy_path)?)?;
    deploy["settlementProgramId"] = serde_json::Value::String(settlement_program_id.to_string());
    deploy["appId"] = serde_json::Value::String(hex::encode(app_id));
    std::fs::write(&deploy_path, serde_json::to_string_pretty(&deploy)?)?;
    println!("  ncn_deploy.json patched with the settlement binding");

    // Frontend config: model=qwen + the answer-event coordinates so Track Q3
    // can find the settle tx's qwen_answer self-CPI and BPE-decode answer_ids.
    write_json(
        &out.join("frontend-config.json"),
        &serde_json::json!({
            "rpcUrl": rpc_url,
            "ncnProgramId": stack.ncn_program_id,
            "settlementProgramId": settlement_program_id.to_string(),
            "statePda": state_pda.to_string(),
            "cluster": "localnet",
            "commitment": "confirmed",
            "model": "qwen",
            "qwenModelTag": verified.answer.model,
            "answerEvent": {
                "discriminantHex": hex::encode(QWEN_ANSWER_DISCRIMINANT),
                "transitionIndex": verified.payload.transition_index,
                "manifestHex": verified.fixture.manifest,
            },
            "prompt": verified.fixture.prompt,
            "promptIds": verified.fixture.prompt_ids,
        }),
    )?;
    println!("  frontend-config.json written (model=qwen)");

    // Leg state (reused by qwen-replay) + producer-regen env.
    write_json(
        &out.join("llm.json"),
        &LlmState {
            settlement_program_id: settlement_program_id.to_string(),
            state_pda: state_pda.to_string(),
            buffer_pda: state_pda.to_string(), // unused for qwen (answer is inline)
            app_id_hex: hex::encode(app_id),
            sim_profile_id_hex: hex::encode(sim_profile_id),
            env_commitment_hex: hex::encode(env_commitment),
        },
    )?;

    let env_sh = format!(
        "# generated by counter-solana-deployer qwen-init\n\
         export LLM_STATE_PDA={}\n\
         export LLM_STATE_PDA_HEX={}\n\
         export LLM_SETTLE_DISC_HEX={}\n\
         export LLM_NEW_ROOT_HEX={}\n\
         export LLM_MANIFEST_HEX={}\n\
         export LLM_MODEL_TAG={}\n\
         export LLM_PROMPT={}\n\
         export LLM_PROMPT_IDS={}\n\
         export LLM_ANSWER_IDS={}\n\
         export LLM_ANSWER_TEXT={}\n\
         export LLM_SIM_COMMAND={}\n\
         export LLM_SDK_COMMIT={}\n\
         export LLM_CHECKED_IN_DIGEST_HEX={}\n",
        state_pda,
        hex::encode(state_pda.to_bytes()),
        hex::encode(SETTLE_DISCRIMINATOR),
        hex::encode(verified.new_root),
        verified.fixture.manifest,
        verified.answer.model,
        shell_quote(&verified.fixture.prompt),
        verified
            .fixture
            .prompt_ids
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(","),
        verified
            .fixture
            .answer_ids
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(","),
        shell_quote(&verified.fixture.answer_text),
        shell_quote(&verified.fixture.source.cmd),
        shell_quote(&verified.fixture.source.sdk_commit),
        verified.fixture.digest_hex,
    );
    std::fs::write(out.join("llm_env.sh"), env_sh)?;
    println!("  llm_env.sh + llm.json written");
    println!("== qwen-init complete ==");
    Ok(())
}

/// qwen-assert: after the router lands `Settle`, assert at `confirmed` that the
/// commitment_root == the Qwen root, transition_count == 1, and the qwen_answer
/// self-CPI event carries the real answer_ids. No buffer (the answer is inline).
pub fn qwen_assert(
    client: &RpcClient,
    out: &Path,
    payload_path: &Path,
    timeout_secs: u64,
) -> Result<()> {
    println!("== qwen-assert: settle outcome on-chain (confirmed) ==");
    let llm = read_llm_state(out)?;
    let settlement_pid = Pubkey::from_str(&llm.settlement_program_id)?;
    let verified = load_qwen_fixture(payload_path)?;
    let state_pda = Pubkey::new_from_array(verified.payload.state_pda);
    let expected_count = verified.payload.transition_index + 1;

    // 1. Wait for the transition counter to advance.
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        match read_gk_state(client, &state_pda) {
            Ok(state) if state.transition_count() >= expected_count => break,
            Ok(state) => println!(
                "  waiting: transition_count={} (want {expected_count})",
                state.transition_count()
            ),
            Err(e) => println!("  waiting: {e}"),
        }
        if Instant::now() > deadline {
            bail!("settle did not land within {timeout_secs}s");
        }
        std::thread::sleep(Duration::from_secs(2));
    }

    // 2. State: exactly one transition, root == the payload's Store.
    let state = read_gk_state(client, &state_pda)?;
    if state.transition_count() != expected_count {
        bail!(
            "transition_count {} != expected {expected_count}",
            state.transition_count()
        );
    }
    if state.commitment_root() != &verified.new_root {
        bail!(
            "commitment_root {} != fixture root {}",
            hex::encode(state.commitment_root()),
            hex::encode(verified.new_root)
        );
    }
    println!(
        "  commitment_root == fixture root ({}), transition_count == {expected_count}",
        hex::encode(verified.new_root)
    );

    // 3. The settle transaction carries the qwen_answer self-CPI event with the
    //    real answer ids.
    let (sig, settle_data, events) = find_settle_tx(client, &settlement_pid, &state_pda)?
        .ok_or_else(|| anyhow!("no successful settle transaction found on {state_pda}"))?;
    let args = SettleArgs::try_from_slice(&settle_data[8..]).context("landed settle args")?;
    if args.payload != verified.payload {
        bail!("landed settle payload does not match the fixture payload");
    }
    let qwen_event = events
        .iter()
        .find(|data| data.get(..8) == Some(&QWEN_ANSWER_DISCRIMINANT))
        .ok_or_else(|| anyhow!("settle tx has no qwen_answer self-CPI inner instruction"))?;
    let event_answer =
        QwenAnswer::try_from_slice(&qwen_event[8..]).context("qwen_answer event payload")?;
    if event_answer != verified.answer {
        bail!("emitted qwen_answer does not match the fixture answer");
    }
    if event_answer.answer_ids != verified.fixture.answer_ids {
        bail!("emitted answer_ids do not match the fixture answer_ids");
    }
    println!(
        "  qwen_answer self-CPI event present in settle tx {sig}: answer_ids {:?}",
        event_answer.answer_ids
    );
    std::fs::write(out.join("llm_settle_tx.txt"), sig.to_string())?;

    // 4. The answer, decoded for humans (Track Q3 BPE-decodes answer_ids live).
    println!("--- qwen answer (from the fixture text) ---");
    println!("{}", verified.fixture.answer_text);
    println!("--- end answer ---");
    println!(
        "== qwen-assert PASSED: settle tx {sig}, digest {} ==",
        verified.fixture.digest_hex
    );
    Ok(())
}
