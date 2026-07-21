//! One-shot deployer for the counter-solana e2e stack.
//!
//! Drives the REAL jito restaking + vault programs (built from source at the
//! pinned rev) and the REAL NCN program on a local `solana-test-validator`.
//! Nothing is mocked: every account transition below is a signed transaction
//! processed by the deployed programs.
//!
//! Three subcommands, matching the two-phase restart-warp choreography that
//! `scripts/solana_e2e_local.sh` performs (jito SlotToggles need
//! `current_epoch > activation_epoch + 1`, and epoch_length is hardcoded to
//! 432,000 slots at `InitializeConfig` — so the driver restarts the validator
//! with `--warp-slot` into epoch 2 between `deploy` and `activate`):
//!
//! - `deploy` (epoch 0): configs, NCN, operators, full handshake mesh, vault,
//!   depositor mint, delegations, BLS proof-of-possession registrations,
//!   ip/port registration; emits `ncn_deploy.json`, per-node BLS key files,
//!   the router connection file and `state.json`.
//! - `activate` (epoch 2, post-warp): vault update-state-tracker crank, NCN
//!   snapshot crank per operator, prints the resulting quorum view.
//! - `assert-verified`: polls the chain for a successful `VerifyCertificate`
//!   transaction (discriminator byte 10) at `confirmed` commitment.
//!
//! The LLM settlement leg adds `llm-init` / `llm-stage` / `llm-assert` /
//! `llm-replay` (see the `llm` module docs): the gaskiller-settlement consumer
//! around the same committee flow.

mod llm;
mod ncn_ix;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use jito_bytemuck::AccountDeserialize;
use ncn_program_core::account_payer::AccountPayer;
use ncn_program_core::config::Config as NcnConfig;
use ncn_program_core::constants::MAX_REALLOC_BYTES;
use ncn_program_core::g1_point::G1CompressedPoint;
use ncn_program_core::g2_point::G2CompressedPoint;
use ncn_program_core::ncn_operator_account::NCNOperatorAccount;
use ncn_program_core::privkey::PrivKey;
use ncn_program_core::schemes::Sha256Normalized;
use ncn_program_core::snapshot::Snapshot;
use ncn_program_core::utils::pop_message_digest;
use ncn_program_core::vault_registry::VaultRegistry;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, write_keypair_file, Keypair, Signature, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use solana_transaction_status_client_types::UiTransactionEncoding;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, Instant};

/// Canonical jito program ids — the addresses the source-built `.so`s are
/// loaded at by the driver script (declare_id env-injected at build time).
const DEFAULT_RESTAKING_PROGRAM_ID: &str = "RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q";
const DEFAULT_VAULT_PROGRAM_ID: &str = "Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8";
const DEFAULT_NCN_PROGRAM_ID: &str = "3fKQSi6VzzDUJSmeksS8qK6RB3Gs3UoZWtsQD3xagy45";

const VERIFY_CERTIFICATE_DISCRIMINATOR: u8 = 10;

/// Per-operator delegation (st tokens). Equal stakes: a 3-of-4 subset holds
/// 7,500 bps >= the program's 6,667 bps default threshold.
const DELEGATION_PER_OPERATOR: u64 = 100_000;
/// Deposited into the vault by the depositor (well above total delegations).
const DEPOSIT_AMOUNT: u64 = 500_000;
/// Minted to the depositor's st token account.
const DEPOSITOR_MINT_AMOUNT: u64 = 1_000_000;

#[derive(Parser)]
#[command(name = "counter-solana-deployer")]
struct Cli {
    /// RPC endpoint of the local test validator.
    #[arg(long, default_value = "http://127.0.0.1:8899")]
    rpc_url: String,
    /// Artifact directory (keys, deployment JSON, state).
    #[arg(long, default_value = ".solana-e2e")]
    out_dir: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Phase A (epoch 0): full jito + NCN bootstrap, all registrations.
    Deploy {
        #[arg(long, default_value = DEFAULT_RESTAKING_PROGRAM_ID)]
        restaking_program_id: String,
        #[arg(long, default_value = DEFAULT_VAULT_PROGRAM_ID)]
        vault_program_id: String,
        #[arg(long, default_value = DEFAULT_NCN_PROGRAM_ID)]
        ncn_program_id: String,
        #[arg(long, default_value_t = 4)]
        num_operators: usize,
        /// First node p2p port; node i listens on base_port + i.
        #[arg(long, default_value_t = 3101)]
        base_port: u16,
        /// Router p2p port (for the router connection file).
        #[arg(long, default_value_t = 3100)]
        router_port: u16,
    },
    /// Phase B (epoch 2, after the --warp-slot restart): cranks.
    Activate,
    /// Phase C: assert a successful on-chain VerifyCertificate at confirmed.
    AssertVerified {
        #[arg(long, default_value_t = 180)]
        timeout_secs: u64,
    },
    /// LLM leg: InitializeState for the settlement consumer + emit artifacts.
    LlmInit {
        /// The checked-in producer fixture (provenance + story + root).
        #[arg(long)]
        fixture: PathBuf,
        #[arg(long, default_value = llm::DEFAULT_SETTLEMENT_PROGRAM_ID)]
        settlement_program_id: String,
    },
    /// LLM leg: stage the story bytes into the transition buffer (chunked).
    LlmStage {
        /// The REGENERATED payload fixture (real state pda + discriminator).
        #[arg(long)]
        payload: PathBuf,
        /// Chunk size in bytes; must be smaller than the story so the append
        /// path is exercised with >= 2 WriteBuffer calls.
        #[arg(long, default_value_t = 256)]
        chunk_size: usize,
    },
    /// LLM leg: assert the settle outcome on-chain at confirmed.
    LlmAssert {
        #[arg(long)]
        payload: PathBuf,
        #[arg(long, default_value_t = 300)]
        timeout_secs: u64,
    },
    /// LLM leg: resubmit the landed Settle; must fail InvalidTransitionIndex.
    LlmReplay,
    /// Qwen leg (§8): InitializeState for the Qwen consumer + emit artifacts.
    QwenInit {
        /// The checked-in Qwen producer fixture (prompt/answer ids + root).
        #[arg(long)]
        fixture: PathBuf,
        #[arg(long, default_value = llm::DEFAULT_SETTLEMENT_PROGRAM_ID)]
        settlement_program_id: String,
    },
    /// Qwen leg (§8): assert the settle outcome (root, count, answer_ids).
    QwenAssert {
        #[arg(long)]
        payload: PathBuf,
        #[arg(long, default_value_t = 300)]
        timeout_secs: u64,
    },
}

/// Everything later phases need, persisted by `deploy`.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StackState {
    restaking_program_id: String,
    vault_program_id: String,
    ncn_program_id: String,
    ncn: String,
    vault: String,
    st_mint: String,
    vrt_mint: String,
    operators: Vec<String>,
    node_ports: Vec<u16>,
    router_port: u16,
}

/// The router/node deployment config (`NcnDeployment` in the jito crate).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NcnDeploymentJson {
    rpc_http_url: String,
    commitment: String,
    ncn_program_id: String,
    ncn: String,
    restaking_program_id: String,
    consensus_threshold_bps: u64,
    compute_unit_limit: u32,
}

/// BLS key file format consumed by the counter-solana node binaries.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct KeyFileJson {
    private_key: String,
}

/// Router connection file consumed by the node binaries (`--router`).
#[derive(Serialize)]
struct RouterConnectionJson {
    g2: String,
    address: String,
    port: u16,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    std::fs::create_dir_all(&cli.out_dir).context("create out dir")?;
    let client = RpcClient::new_with_commitment(cli.rpc_url.clone(), CommitmentConfig::confirmed());

    match cli.command {
        Command::Deploy {
            restaking_program_id,
            vault_program_id,
            ncn_program_id,
            num_operators,
            base_port,
            router_port,
        } => deploy(
            &client,
            &cli.rpc_url,
            &cli.out_dir,
            &Pubkey::from_str(&restaking_program_id)?,
            &Pubkey::from_str(&vault_program_id)?,
            &Pubkey::from_str(&ncn_program_id)?,
            num_operators,
            base_port,
            router_port,
        ),
        Command::Activate => activate(&client, &cli.out_dir),
        Command::AssertVerified { timeout_secs } => {
            assert_verified(&client, &cli.out_dir, timeout_secs)
        }
        Command::LlmInit {
            fixture,
            settlement_program_id,
        } => llm::llm_init(
            &client,
            &cli.rpc_url,
            &cli.out_dir,
            &fixture,
            &Pubkey::from_str(&settlement_program_id)?,
        ),
        Command::LlmStage {
            payload,
            chunk_size,
        } => llm::llm_stage(&client, &cli.out_dir, &payload, chunk_size),
        Command::LlmAssert {
            payload,
            timeout_secs,
        } => llm::llm_assert(&client, &cli.out_dir, &payload, timeout_secs),
        Command::LlmReplay => llm::llm_replay(&client, &cli.out_dir),
        Command::QwenInit {
            fixture,
            settlement_program_id,
        } => llm::qwen_init(
            &client,
            &cli.rpc_url,
            &cli.out_dir,
            &fixture,
            &Pubkey::from_str(&settlement_program_id)?,
        ),
        Command::QwenAssert {
            payload,
            timeout_secs,
        } => llm::qwen_assert(&client, &cli.out_dir, &payload, timeout_secs),
    }
}

// ---------------------------------------------------------------------------
// Transaction plumbing
// ---------------------------------------------------------------------------

fn send(
    client: &RpcClient,
    label: &str,
    ixs: &[Instruction],
    payer: &Keypair,
    extra_signers: &[&Keypair],
) -> Result<Signature> {
    let mut signers: Vec<&Keypair> = vec![payer];
    signers.extend_from_slice(extra_signers);
    let mut last_err = None;
    for attempt in 0..5 {
        let blockhash = client
            .get_latest_blockhash()
            .context("get_latest_blockhash")?;
        let tx =
            Transaction::new_signed_with_payer(ixs, Some(&payer.pubkey()), &signers, blockhash);
        match client.send_and_confirm_transaction(&tx) {
            Ok(sig) => {
                println!("  ok  {label} ({sig})");
                return Ok(sig);
            }
            Err(e) => {
                let msg = e.to_string();
                // Preflight/simulation errors are deterministic program
                // rejections — retrying cannot help, fail loudly.
                if msg.contains("custom program error") || msg.contains("InstructionError") {
                    return Err(anyhow!("{label}: {msg}"));
                }
                println!("  retry {label} (attempt {attempt}): {msg}");
                last_err = Some(anyhow!("{label}: {msg}"));
                std::thread::sleep(Duration::from_millis(1500));
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow!("{label}: send failed")))
}

fn airdrop(client: &RpcClient, to: &Pubkey, sol: u64) -> Result<()> {
    let lamports = sol * LAMPORTS_PER_SOL;
    let sig = client.request_airdrop(to, lamports).context("airdrop")?;
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        if client
            .confirm_transaction_with_commitment(&sig, CommitmentConfig::confirmed())?
            .value
        {
            return Ok(());
        }
        if Instant::now() > deadline {
            bail!("airdrop to {to} not confirmed within 60s");
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

/// Waits until the cluster has moved past `after_slot`. jito's `SlotToggle`
/// refuses activation in the very slot the toggle was created (the
/// `slot_added == slot` guard), so every init -> warmup pair must straddle a
/// slot boundary.
fn wait_past_slot(client: &RpcClient, after_slot: u64) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        if client.get_slot()? > after_slot {
            return Ok(());
        }
        if Instant::now() > deadline {
            bail!("cluster did not advance past slot {after_slot} within 60s");
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn load_or_create_keypair(path: &Path) -> Result<Keypair> {
    if path.exists() {
        read_keypair_file(path).map_err(|e| anyhow!("read {}: {e}", path.display()))
    } else {
        let kp = Keypair::new();
        write_keypair_file(&kp, path).map_err(|e| anyhow!("write {}: {e}", path.display()))?;
        Ok(kp)
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    std::fs::write(path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Phase A — deploy (epoch 0)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn deploy(
    client: &RpcClient,
    rpc_url: &str,
    out: &Path,
    restaking_pid: &Pubkey,
    vault_pid: &Pubkey,
    ncn_pid: &Pubkey,
    num_operators: usize,
    base_port: u16,
    router_port: u16,
) -> Result<()> {
    println!("== phase A: deploy (must complete inside epoch 0) ==");
    let slot = client.get_slot()?;
    println!("current slot: {slot}");

    // One authority for every admin role + fee payer: real transactions,
    // minimal key sprawl. (jito operator/vault/ncn admin separation is not
    // under test here; the BLS operator identities ARE distinct.)
    let authority = load_or_create_keypair(&out.join("authority.json"))?;
    airdrop(client, &authority.pubkey(), 1000)?;
    println!("authority: {}", authority.pubkey());

    // --- jito restaking + vault configs -----------------------------------
    let restaking_config =
        jito_restaking_core::config::Config::find_program_address(restaking_pid).0;
    let vault_config = jito_vault_core::config::Config::find_program_address(vault_pid).0;
    send(
        client,
        "restaking InitializeConfig",
        &[jito_restaking_sdk::sdk::initialize_config(
            restaking_pid,
            &restaking_config,
            &authority.pubkey(),
            vault_pid,
        )],
        &authority,
        &[],
    )?;
    send(
        client,
        "vault InitializeConfig",
        &[jito_vault_sdk::sdk::initialize_config(
            vault_pid,
            &vault_config,
            &authority.pubkey(),
            restaking_pid,
            &authority.pubkey(),
            0,
        )],
        &authority,
        &[],
    )?;

    // --- NCN ----------------------------------------------------------------
    let ncn_base = Keypair::new();
    let ncn =
        jito_restaking_core::ncn::Ncn::find_program_address(restaking_pid, &ncn_base.pubkey()).0;
    send(
        client,
        "restaking InitializeNcn",
        &[jito_restaking_sdk::sdk::initialize_ncn(
            restaking_pid,
            &restaking_config,
            &ncn,
            &authority.pubkey(),
            &ncn_base.pubkey(),
        )],
        &authority,
        &[&ncn_base],
    )?;
    println!("ncn: {ncn}");

    // --- operators + NCN<->operator handshake (warmups land in epoch 0) ----
    let mut operators = Vec::with_capacity(num_operators);
    for i in 0..num_operators {
        let base = Keypair::new();
        let operator = jito_restaking_core::operator::Operator::find_program_address(
            restaking_pid,
            &base.pubkey(),
        )
        .0;
        let ncn_operator_state =
            jito_restaking_core::ncn_operator_state::NcnOperatorState::find_program_address(
                restaking_pid,
                &ncn,
                &operator,
            )
            .0;
        send(
            client,
            &format!("restaking InitializeOperator[{i}]"),
            &[jito_restaking_sdk::sdk::initialize_operator(
                restaking_pid,
                &restaking_config,
                &operator,
                &authority.pubkey(),
                &base.pubkey(),
                0,
            )],
            &authority,
            &[&base],
        )?;
        send(
            client,
            &format!("restaking InitializeNcnOperatorState[{i}]"),
            &[jito_restaking_sdk::sdk::initialize_ncn_operator_state(
                restaking_pid,
                &restaking_config,
                &ncn,
                &operator,
                &ncn_operator_state,
                &authority.pubkey(),
                &authority.pubkey(),
            )],
            &authority,
            &[],
        )?;
        operators.push(operator);
    }
    // SlotToggle refuses same-slot activation: cross a slot boundary before
    // warming the freshly created NcnOperatorStates up.
    wait_past_slot(client, client.get_slot()?)?;
    for (i, operator) in operators.iter().enumerate() {
        let ncn_operator_state =
            jito_restaking_core::ncn_operator_state::NcnOperatorState::find_program_address(
                restaking_pid,
                &ncn,
                operator,
            )
            .0;
        send(
            client,
            &format!("restaking Ncn/Operator warmups[{i}]"),
            &[
                jito_restaking_sdk::sdk::ncn_warmup_operator(
                    restaking_pid,
                    &restaking_config,
                    &ncn,
                    operator,
                    &ncn_operator_state,
                    &authority.pubkey(),
                ),
                jito_restaking_sdk::sdk::operator_warmup_ncn(
                    restaking_pid,
                    &restaking_config,
                    &ncn,
                    operator,
                    &ncn_operator_state,
                    &authority.pubkey(),
                ),
            ],
            &authority,
            &[],
        )?;
    }

    // --- st mint + vault ----------------------------------------------------
    let st_mint = Keypair::new();
    let mint_rent = client.get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)?;
    send(
        client,
        "create st mint",
        &[
            system_instruction::create_account(
                &authority.pubkey(),
                &st_mint.pubkey(),
                mint_rent,
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint2(
                &spl_token::id(),
                &st_mint.pubkey(),
                &authority.pubkey(),
                None,
                9,
            )?,
        ],
        &authority,
        &[&st_mint],
    )?;

    let vault_base = Keypair::new();
    let vrt_mint = Keypair::new();
    let vault =
        jito_vault_core::vault::Vault::find_program_address(vault_pid, &vault_base.pubkey()).0;
    let burn_vault = jito_vault_core::burn_vault::BurnVault::find_program_address(
        vault_pid,
        &vault_base.pubkey(),
    )
    .0;
    let ata = spl_associated_token_account::get_associated_token_address;
    let create_ata = |owner: &Pubkey, mint: &Pubkey| {
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            &authority.pubkey(),
            owner,
            mint,
            &spl_token::id(),
        )
    };
    let init_amount = jito_vault_core::vault::Vault::DEFAULT_INITIALIZATION_TOKEN_AMOUNT;
    send(
        client,
        "st ATAs + seed authority st balance",
        &[
            create_ata(&vault, &st_mint.pubkey()),
            create_ata(&authority.pubkey(), &st_mint.pubkey()),
            spl_token::instruction::mint_to(
                &spl_token::id(),
                &st_mint.pubkey(),
                &ata(&authority.pubkey(), &st_mint.pubkey()),
                &authority.pubkey(),
                &[],
                init_amount,
            )?,
        ],
        &authority,
        &[],
    )?;
    send(
        client,
        "vault InitializeVault",
        &[jito_vault_sdk::sdk::initialize_vault(
            vault_pid,
            &vault_config,
            &vault,
            &vrt_mint.pubkey(),
            &st_mint.pubkey(),
            &ata(&authority.pubkey(), &st_mint.pubkey()),
            &ata(&vault, &st_mint.pubkey()),
            &burn_vault,
            &ata(&burn_vault, &vrt_mint.pubkey()),
            &authority.pubkey(),
            &vault_base.pubkey(),
            0,
            0,
            0,
            9,
            init_amount,
        )],
        &authority,
        &[&vrt_mint, &vault_base],
    )?;
    println!("vault: {vault}");
    send(
        client,
        "post-init ATAs (vrt fee accounts)",
        &[
            create_ata(&authority.pubkey(), &vrt_mint.pubkey()),
            spl_token::instruction::mint_to(
                &spl_token::id(),
                &st_mint.pubkey(),
                &ata(&authority.pubkey(), &st_mint.pubkey()),
                &authority.pubkey(),
                &[],
                DEPOSITOR_MINT_AMOUNT,
            )?,
        ],
        &authority,
        &[],
    )?;

    // --- NCN <-> vault + operator <-> vault handshake -----------------------
    let ncn_vault_ticket =
        jito_restaking_core::ncn_vault_ticket::NcnVaultTicket::find_program_address(
            restaking_pid,
            &ncn,
            &vault,
        )
        .0;
    let vault_ncn_ticket = jito_vault_core::vault_ncn_ticket::VaultNcnTicket::find_program_address(
        vault_pid, &vault, &ncn,
    )
    .0;
    send(
        client,
        "NcnVaultTicket init",
        &[jito_restaking_sdk::sdk::initialize_ncn_vault_ticket(
            restaking_pid,
            &restaking_config,
            &ncn,
            &vault,
            &ncn_vault_ticket,
            &authority.pubkey(),
            &authority.pubkey(),
        )],
        &authority,
        &[],
    )?;
    send(
        client,
        "VaultNcnTicket init",
        &[jito_vault_sdk::sdk::initialize_vault_ncn_ticket(
            vault_pid,
            &vault_config,
            &vault,
            &ncn,
            &ncn_vault_ticket,
            &vault_ncn_ticket,
            &authority.pubkey(),
            &authority.pubkey(),
        )],
        &authority,
        &[],
    )?;
    for (i, operator) in operators.iter().enumerate() {
        let operator_vault_ticket =
            jito_restaking_core::operator_vault_ticket::OperatorVaultTicket::find_program_address(
                restaking_pid,
                operator,
                &vault,
            )
            .0;
        let vault_operator_delegation =
            jito_vault_core::vault_operator_delegation::VaultOperatorDelegation::find_program_address(
                vault_pid, &vault, operator,
            )
            .0;
        send(
            client,
            &format!("OperatorVaultTicket[{i}] init + delegation init"),
            &[
                jito_restaking_sdk::sdk::initialize_operator_vault_ticket(
                    restaking_pid,
                    &restaking_config,
                    operator,
                    &vault,
                    &operator_vault_ticket,
                    &authority.pubkey(),
                    &authority.pubkey(),
                ),
                jito_vault_sdk::sdk::initialize_vault_operator_delegation(
                    vault_pid,
                    &vault_config,
                    &vault,
                    operator,
                    &operator_vault_ticket,
                    &vault_operator_delegation,
                    &authority.pubkey(),
                    &authority.pubkey(),
                ),
            ],
            &authority,
            &[],
        )?;
    }
    // Cross a slot boundary, then warm the whole vault mesh up.
    wait_past_slot(client, client.get_slot()?)?;
    send(
        client,
        "NcnVaultTicket + VaultNcnTicket warmups",
        &[
            jito_restaking_sdk::sdk::warmup_ncn_vault_ticket(
                restaking_pid,
                &restaking_config,
                &ncn,
                &vault,
                &ncn_vault_ticket,
                &authority.pubkey(),
            ),
            jito_vault_sdk::sdk::warmup_vault_ncn_ticket(
                vault_pid,
                &vault_config,
                &vault,
                &ncn,
                &vault_ncn_ticket,
                &authority.pubkey(),
            ),
        ],
        &authority,
        &[],
    )?;
    for (i, operator) in operators.iter().enumerate() {
        let operator_vault_ticket =
            jito_restaking_core::operator_vault_ticket::OperatorVaultTicket::find_program_address(
                restaking_pid,
                operator,
                &vault,
            )
            .0;
        send(
            client,
            &format!("OperatorVaultTicket[{i}] warmup"),
            &[jito_restaking_sdk::sdk::warmup_operator_vault_ticket(
                restaking_pid,
                &restaking_config,
                operator,
                &vault,
                &operator_vault_ticket,
                &authority.pubkey(),
            )],
            &authority,
            &[],
        )?;
    }

    // --- deposit + delegate -------------------------------------------------
    send(
        client,
        "vault MintTo (deposit)",
        &[jito_vault_sdk::sdk::mint_to(
            vault_pid,
            &vault_config,
            &vault,
            &vrt_mint.pubkey(),
            &authority.pubkey(),
            &ata(&authority.pubkey(), &st_mint.pubkey()),
            &ata(&vault, &st_mint.pubkey()),
            &ata(&authority.pubkey(), &vrt_mint.pubkey()),
            &ata(&authority.pubkey(), &vrt_mint.pubkey()),
            None,
            DEPOSIT_AMOUNT,
            0,
        )],
        &authority,
        &[],
    )?;
    for (i, operator) in operators.iter().enumerate() {
        let vault_operator_delegation =
            jito_vault_core::vault_operator_delegation::VaultOperatorDelegation::find_program_address(
                vault_pid, &vault, operator,
            )
            .0;
        send(
            client,
            &format!("vault AddDelegation[{i}] ({DELEGATION_PER_OPERATOR})"),
            &[jito_vault_sdk::sdk::add_delegation(
                vault_pid,
                &vault_config,
                &vault,
                operator,
                &vault_operator_delegation,
                &authority.pubkey(),
                DELEGATION_PER_OPERATOR,
            )],
            &authority,
            &[],
        )?;
    }

    // --- NCN program: config, registry, snapshot ---------------------------
    let ncn_config = NcnConfig::find_program_address(ncn_pid, &ncn).0;
    let vault_registry = VaultRegistry::find_program_address(ncn_pid, &ncn).0;
    let snapshot = Snapshot::find_program_address(ncn_pid, &ncn).0;
    let account_payer = AccountPayer::find_program_address(ncn_pid, &ncn).0;
    airdrop(client, &account_payer, 100)?;
    // The protocol fee wallet must exist for fee accounting.
    airdrop(
        client,
        &ncn_program_core::fees::FeeConfig::PROTOCOL_FEE_WALLET,
        1,
    )?;
    send(
        client,
        "ncn InitializeConfig",
        &[ncn_ix::initialize_config(
            ncn_pid,
            &ncn_config,
            &ncn,
            &authority.pubkey(),
            &authority.pubkey(),
            &authority.pubkey(),
            &account_payer,
            10,     // epochs_before_stall
            10,     // epochs_after_consensus_before_close
            10_000, // valid_slots_after_consensus (freshness window, slots)
            100,    // minimum_stake
            400,    // ncn_fee_bps
        )],
        &authority,
        &[],
    )?;
    send(
        client,
        "ncn InitializeVaultRegistry",
        &[ncn_ix::initialize_vault_registry(
            ncn_pid,
            &ncn_config,
            &vault_registry,
            &ncn,
            &account_payer,
        )],
        &authority,
        &[],
    )?;
    send(
        client,
        "ncn AdminRegisterStMint + RegisterVault",
        &[
            ncn_ix::admin_register_st_mint(
                ncn_pid,
                &ncn_config,
                &ncn,
                &st_mint.pubkey(),
                &vault_registry,
                &authority.pubkey(),
                10_000,
            ),
            ncn_ix::register_vault(
                ncn_pid,
                &ncn_config,
                &vault_registry,
                &ncn,
                &vault,
                &ncn_vault_ticket,
            ),
        ],
        &authority,
        &[],
    )?;
    send(
        client,
        "ncn InitializeSnapshot",
        &[ncn_ix::initialize_snapshot(
            ncn_pid,
            &ncn,
            &snapshot,
            &account_payer,
        )],
        &authority,
        &[],
    )?;
    let num_reallocs = (Snapshot::SIZE as u64)
        .div_ceil(MAX_REALLOC_BYTES)
        .saturating_sub(1);
    // Chunk reallocs to stay under tx size/CU limits.
    let realloc_ix =
        ncn_ix::realloc_snapshot(ncn_pid, &ncn, &ncn_config, &snapshot, &account_payer);
    let mut remaining = num_reallocs;
    while remaining > 0 {
        let batch = remaining.min(10);
        let ixs: Vec<Instruction> = (0..batch).map(|_| realloc_ix.clone()).collect();
        send(
            client,
            &format!("ncn ReallocSnapshot x{batch} ({remaining} left)"),
            &ixs,
            &authority,
            &[],
        )?;
        remaining -= batch;
    }

    // --- BLS registration (real proof-of-possession) ------------------------
    let restaking_config_pda = restaking_config;
    let mut node_ports = Vec::with_capacity(num_operators);
    for (i, operator) in operators.iter().enumerate() {
        let privkey = PrivKey::from_random();
        let g1 =
            G1CompressedPoint::try_from(privkey).map_err(|e| anyhow!("G1 derivation: {e:?}"))?;
        let g2 =
            G2CompressedPoint::try_from(&privkey).map_err(|e| anyhow!("G2 derivation: {e:?}"))?;
        let pop_digest = pop_message_digest(&ncn, operator, &g1.0);
        let signature = privkey
            .sign::<Sha256Normalized>(&pop_digest)
            .map_err(|e| anyhow!("PoP signing: {e:?}"))?;

        let ncn_operator_account =
            NCNOperatorAccount::find_program_address(ncn_pid, &ncn, operator).0;
        let ncn_operator_state =
            jito_restaking_core::ncn_operator_state::NcnOperatorState::find_program_address(
                restaking_pid,
                &ncn,
                operator,
            )
            .0;
        let port = base_port + i as u16;
        send(
            client,
            &format!("ncn RegisterOperator[{i}] (PoP) + ip/port 127.0.0.1:{port}"),
            &[
                ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
                ncn_ix::register_operator(
                    ncn_pid,
                    &ncn_config,
                    &ncn_operator_account,
                    &ncn,
                    operator,
                    &authority.pubkey(),
                    &ncn_operator_state,
                    &snapshot,
                    &restaking_config_pda,
                    &account_payer,
                    g1.0,
                    g2.0,
                    signature.0,
                ),
                ncn_ix::update_operator_ip_port(
                    ncn_pid,
                    &ncn_config,
                    &ncn_operator_account,
                    &ncn,
                    operator,
                    &authority.pubkey(),
                    [127, 0, 0, 1],
                    port,
                ),
            ],
            &authority,
            &[],
        )?;
        write_json(
            &out.join(format!("node-{i}.bls.json")),
            &KeyFileJson {
                private_key: hex::encode(privkey.0),
            },
        )?;
        node_ports.push(port);
    }

    // Router identity: its own BLS key (p2p identity, never registered).
    let router_key = PrivKey::from_random();
    let router_g2 = G2CompressedPoint::try_from(&router_key)
        .map_err(|e| anyhow!("router G2 derivation: {e:?}"))?;
    write_json(
        &out.join("router.bls.json"),
        &KeyFileJson {
            private_key: hex::encode(router_key.0),
        },
    )?;
    write_json(
        &out.join("router-connection.json"),
        &RouterConnectionJson {
            g2: hex::encode(router_g2.0),
            address: "127.0.0.1".to_string(),
            port: router_port,
        },
    )?;

    // --- artifacts ----------------------------------------------------------
    write_json(
        &out.join("ncn_deploy.json"),
        &NcnDeploymentJson {
            rpc_http_url: rpc_url.to_string(),
            commitment: "confirmed".to_string(),
            ncn_program_id: ncn_pid.to_string(),
            ncn: ncn.to_string(),
            restaking_program_id: restaking_pid.to_string(),
            consensus_threshold_bps: 6667,
            compute_unit_limit: 1_000_000,
        },
    )?;
    write_json(
        &out.join("state.json"),
        &StackState {
            restaking_program_id: restaking_pid.to_string(),
            vault_program_id: vault_pid.to_string(),
            ncn_program_id: ncn_pid.to_string(),
            ncn: ncn.to_string(),
            vault: vault.to_string(),
            st_mint: st_mint.pubkey().to_string(),
            vrt_mint: vrt_mint.pubkey().to_string(),
            operators: operators.iter().map(|o| o.to_string()).collect(),
            node_ports,
            router_port,
        },
    )?;

    let end_slot = client.get_slot()?;
    println!(
        "== deploy complete at slot {end_slot} (must be < 432000: {}) ==",
        if end_slot < 432_000 { "OK" } else { "VIOLATED" }
    );
    if end_slot >= 432_000 {
        bail!("phase A crossed the epoch boundary; restart from scratch");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Phase B — activate (post-warp, epoch 2)
// ---------------------------------------------------------------------------

fn activate(client: &RpcClient, out: &Path) -> Result<()> {
    println!("== phase B: activate (post --warp-slot restart) ==");
    let state: StackState =
        serde_json::from_str(&std::fs::read_to_string(out.join("state.json"))?)?;
    let authority = read_keypair_file(out.join("authority.json"))
        .map_err(|e| anyhow!("read authority: {e}"))?;
    let restaking_pid = Pubkey::from_str(&state.restaking_program_id)?;
    let vault_pid = Pubkey::from_str(&state.vault_program_id)?;
    let ncn_pid = Pubkey::from_str(&state.ncn_program_id)?;
    let ncn = Pubkey::from_str(&state.ncn)?;
    let vault = Pubkey::from_str(&state.vault)?;
    let st_mint = Pubkey::from_str(&state.st_mint)?;
    let vrt_mint = Pubkey::from_str(&state.vrt_mint)?;
    let operators: Vec<Pubkey> = state
        .operators
        .iter()
        .map(|s| Pubkey::from_str(s))
        .collect::<Result<_, _>>()?;

    let slot = client.get_slot()?;
    let vault_config_pda = jito_vault_core::config::Config::find_program_address(&vault_pid).0;
    let vault_config_data = client.get_account_data(&vault_config_pda)?;
    let vault_config =
        jito_vault_core::config::Config::try_from_slice_unchecked(&vault_config_data)
            .map_err(|e| anyhow!("decode vault config: {e:?}"))?;
    let epoch_length = vault_config.epoch_length();
    let ncn_epoch = slot / epoch_length;
    println!("slot {slot}, epoch_length {epoch_length}, ncn_epoch {ncn_epoch}");
    if ncn_epoch < 2 {
        bail!("expected the validator to be warped into epoch >= 2, got epoch {ncn_epoch}");
    }

    // --- vault update-state-tracker crank (required each epoch) ------------
    let tracker =
        jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker::find_program_address(
            &vault_pid, &vault, ncn_epoch,
        )
        .0;
    send(
        client,
        "vault InitializeVaultUpdateStateTracker",
        &[jito_vault_sdk::sdk::initialize_vault_update_state_tracker(
            &vault_pid,
            &vault_config_pda,
            &vault,
            &tracker,
            &authority.pubkey(),
            jito_vault_sdk::instruction::WithdrawalAllocationMethod::Greedy,
        )],
        &authority,
        &[],
    )?;
    // Crank order: ascending delegation index, rotated by epoch (the
    // program's required visitation order — mirrors the upstream client).
    for i in 0..operators.len() {
        let idx = (i + ncn_epoch as usize) % operators.len();
        let operator = &operators[idx];
        let delegation =
            jito_vault_core::vault_operator_delegation::VaultOperatorDelegation::find_program_address(
                &vault_pid, &vault, operator,
            )
            .0;
        send(
            client,
            &format!("vault CrankVaultUpdateStateTracker[{idx}]"),
            &[jito_vault_sdk::sdk::crank_vault_update_state_tracker(
                &vault_pid,
                &vault_config_pda,
                &vault,
                operator,
                &delegation,
                &tracker,
            )],
            &authority,
            &[],
        )?;
    }
    let ata = spl_associated_token_account::get_associated_token_address;
    send(
        client,
        "vault CloseVaultUpdateStateTracker + UpdateVaultBalance",
        &[
            jito_vault_sdk::sdk::close_vault_update_state_tracker(
                &vault_pid,
                &vault_config_pda,
                &vault,
                &tracker,
                &authority.pubkey(),
                ncn_epoch,
            ),
            jito_vault_sdk::sdk::update_vault_balance(
                &vault_pid,
                &vault_config_pda,
                &vault,
                &ata(&vault, &st_mint),
                &vrt_mint,
                &ata(&authority.pubkey(), &vrt_mint),
                &spl_token::id(),
            ),
        ],
        &authority,
        &[],
    )?;

    // --- NCN snapshot crank -------------------------------------------------
    let ncn_config = NcnConfig::find_program_address(&ncn_pid, &ncn).0;
    let vault_registry = VaultRegistry::find_program_address(&ncn_pid, &ncn).0;
    let snapshot_pda = Snapshot::find_program_address(&ncn_pid, &ncn).0;
    let restaking_config =
        jito_restaking_core::config::Config::find_program_address(&restaking_pid).0;
    let vault_ncn_ticket = jito_vault_core::vault_ncn_ticket::VaultNcnTicket::find_program_address(
        &vault_pid, &vault, &ncn,
    )
    .0;
    let ncn_vault_ticket =
        jito_restaking_core::ncn_vault_ticket::NcnVaultTicket::find_program_address(
            &restaking_pid,
            &ncn,
            &vault,
        )
        .0;
    for (i, operator) in operators.iter().enumerate() {
        let ncn_operator_state =
            jito_restaking_core::ncn_operator_state::NcnOperatorState::find_program_address(
                &restaking_pid,
                &ncn,
                operator,
            )
            .0;
        let delegation =
            jito_vault_core::vault_operator_delegation::VaultOperatorDelegation::find_program_address(
                &vault_pid, &vault, operator,
            )
            .0;
        send(
            client,
            &format!("ncn SnapshotVaultOperatorDelegation[{i}]"),
            &[
                ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
                ncn_ix::snapshot_vault_operator_delegation(
                    &ncn_pid,
                    &ncn_config,
                    &restaking_config,
                    &ncn,
                    operator,
                    &vault,
                    &vault_ncn_ticket,
                    &ncn_vault_ticket,
                    &ncn_operator_state,
                    &delegation,
                    &snapshot_pda,
                    &vault_registry,
                ),
            ],
            &authority,
            &[],
        )?;
    }

    // --- report the resulting quorum view -----------------------------------
    let snapshot_data = client.get_account_data(&snapshot_pda)?;
    let snapshot = Snapshot::try_from_slice_unchecked(&snapshot_data)
        .map_err(|e| anyhow!("decode snapshot: {e:?}"))?;
    let total_stake: u128 = snapshot
        .operator_snapshots()
        .iter()
        .map(|op| op.stake_weight().stake_weight())
        .sum();
    println!(
        "snapshot: operators_registered={} generation={} last_snapshot_slot={} total_stake={}",
        snapshot.operators_registered(),
        snapshot.generation(),
        snapshot.last_snapshot_slot(),
        total_stake,
    );
    for operator in &operators {
        let op = snapshot
            .find_operator_snapshot(operator)
            .ok_or_else(|| anyhow!("operator {operator} missing from snapshot"))?;
        println!(
            "  operator {operator}: stake={} active={} has_min_stake={}",
            op.stake_weight().stake_weight(),
            op.is_active(),
            op.has_minimum_stake(),
        );
        if !(op.is_active() && op.has_minimum_stake()) {
            bail!("operator {operator} cannot vote — activation failed");
        }
    }
    println!(
        "== activate complete: all {} operators can vote ==",
        operators.len()
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Phase C — assert a successful VerifyCertificate landed
// ---------------------------------------------------------------------------

fn assert_verified(client: &RpcClient, out: &Path, timeout_secs: u64) -> Result<()> {
    let state: StackState =
        serde_json::from_str(&std::fs::read_to_string(out.join("state.json"))?)?;
    let ncn_pid = Pubkey::from_str(&state.ncn_program_id)?;
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    println!("== phase C: waiting for a successful VerifyCertificate (confirmed) ==");
    loop {
        let sigs = client.get_signatures_for_address_with_config(
            &ncn_pid,
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
            let is_verify = decoded.message.instructions().iter().any(|ix| {
                keys.get(ix.program_id_index as usize) == Some(&ncn_pid)
                    && ix.data.first() == Some(&VERIFY_CERTIFICATE_DISCRIMINATOR)
            });
            if is_verify {
                let err_none = tx
                    .transaction
                    .meta
                    .as_ref()
                    .map(|m| m.err.is_none())
                    .unwrap_or(false);
                if err_none {
                    println!(
                        "VERIFIED on-chain: tx {} slot {} — successful VerifyCertificate",
                        entry.signature, tx.slot
                    );
                    return Ok(());
                }
            }
        }
        if Instant::now() > deadline {
            bail!("no successful VerifyCertificate observed within {timeout_secs}s");
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}
