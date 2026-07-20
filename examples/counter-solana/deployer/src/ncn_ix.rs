//! NCN program instruction builders, assembled from
//! `ncn_program_core::instruction::NCNProgramInstruction` — the enum the
//! on-chain dispatcher deserializes and the shank/kinobi clients generate
//! from. Serializing the enum with borsh 0.10 yields `discriminator ‖ args`
//! exactly; account orders mirror the enum's `#[account(...)]` annotations.

use borsh::BorshSerialize;
use ncn_program_core::instruction::NCNProgramInstruction;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program;

fn data(ix: &NCNProgramInstruction) -> Vec<u8> {
    ix.try_to_vec().expect("NCNProgramInstruction serializes")
}

/// `InitializeConfig` — accounts: config(w), ncn, ncn_fee_wallet,
/// ncn_admin(s), tie_breaker_admin, account_payer(w), system_program.
#[allow(clippy::too_many_arguments)]
pub fn initialize_config(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    ncn_fee_wallet: &Pubkey,
    ncn_admin: &Pubkey,
    tie_breaker_admin: &Pubkey,
    account_payer: &Pubkey,
    epochs_before_stall: u64,
    epochs_after_consensus_before_close: u64,
    valid_slots_after_consensus: u64,
    minimum_stake: u128,
    ncn_fee_bps: u16,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*config, false),
            AccountMeta::new_readonly(*ncn, false),
            AccountMeta::new_readonly(*ncn_fee_wallet, false),
            AccountMeta::new_readonly(*ncn_admin, true),
            AccountMeta::new_readonly(*tie_breaker_admin, false),
            AccountMeta::new(*account_payer, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: data(&NCNProgramInstruction::InitializeConfig {
            epochs_before_stall,
            epochs_after_consensus_before_close,
            valid_slots_after_consensus,
            minimum_stake,
            ncn_fee_bps,
        }),
    }
}

/// `InitializeVaultRegistry` — accounts: config, vault_registry(w), ncn,
/// account_payer(w), system_program.
pub fn initialize_vault_registry(
    program_id: &Pubkey,
    config: &Pubkey,
    vault_registry: &Pubkey,
    ncn: &Pubkey,
    account_payer: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new(*vault_registry, false),
            AccountMeta::new_readonly(*ncn, false),
            AccountMeta::new(*account_payer, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: data(&NCNProgramInstruction::InitializeVaultRegistry),
    }
}

/// `RegisterVault` (permissionless) — accounts: config, vault_registry(w),
/// ncn, vault, ncn_vault_ticket.
pub fn register_vault(
    program_id: &Pubkey,
    config: &Pubkey,
    vault_registry: &Pubkey,
    ncn: &Pubkey,
    vault: &Pubkey,
    ncn_vault_ticket: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new(*vault_registry, false),
            AccountMeta::new_readonly(*ncn, false),
            AccountMeta::new_readonly(*vault, false),
            AccountMeta::new_readonly(*ncn_vault_ticket, false),
        ],
        data: data(&NCNProgramInstruction::RegisterVault),
    }
}

/// `AdminRegisterStMint` — accounts: config, ncn, st_mint, vault_registry(w),
/// admin(w+s).
pub fn admin_register_st_mint(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    st_mint: &Pubkey,
    vault_registry: &Pubkey,
    admin: &Pubkey,
    weight_bps: u16,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new_readonly(*ncn, false),
            AccountMeta::new_readonly(*st_mint, false),
            AccountMeta::new(*vault_registry, false),
            AccountMeta::new(*admin, true),
        ],
        data: data(&NCNProgramInstruction::AdminRegisterStMint { weight_bps }),
    }
}

/// `InitializeSnapshot` — accounts: ncn, snapshot(w), account_payer(w),
/// system_program.
pub fn initialize_snapshot(
    program_id: &Pubkey,
    ncn: &Pubkey,
    snapshot: &Pubkey,
    account_payer: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(*ncn, false),
            AccountMeta::new(*snapshot, false),
            AccountMeta::new(*account_payer, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: data(&NCNProgramInstruction::InitializeSnapshot {}),
    }
}

/// `ReallocSnapshot` — accounts: ncn, config, snapshot(w), account_payer(w),
/// system_program.
pub fn realloc_snapshot(
    program_id: &Pubkey,
    ncn: &Pubkey,
    config: &Pubkey,
    snapshot: &Pubkey,
    account_payer: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(*ncn, false),
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new(*snapshot, false),
            AccountMeta::new(*account_payer, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: data(&NCNProgramInstruction::ReallocSnapshot {}),
    }
}

/// `RegisterOperator` — accounts: config, ncn_operator_account(w), ncn,
/// operator, operator_admin(s), ncn_operator_state, snapshot(w),
/// restaking_config, account_payer(w), system_program.
#[allow(clippy::too_many_arguments)]
pub fn register_operator(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn_operator_account: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    operator_admin: &Pubkey,
    ncn_operator_state: &Pubkey,
    snapshot: &Pubkey,
    restaking_config: &Pubkey,
    account_payer: &Pubkey,
    g1_pubkey: [u8; 32],
    g2_pubkey: [u8; 64],
    signature: [u8; 64],
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new(*ncn_operator_account, false),
            AccountMeta::new_readonly(*ncn, false),
            AccountMeta::new_readonly(*operator, false),
            AccountMeta::new_readonly(*operator_admin, true),
            AccountMeta::new_readonly(*ncn_operator_state, false),
            AccountMeta::new(*snapshot, false),
            AccountMeta::new_readonly(*restaking_config, false),
            AccountMeta::new(*account_payer, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: data(&NCNProgramInstruction::RegisterOperator {
            g1_pubkey,
            g2_pubkey,
            signature,
        }),
    }
}

/// `UpdateOperatorIpPort` — accounts: config, ncn_operator_account(w), ncn,
/// operator, operator_admin(s).
#[allow(clippy::too_many_arguments)]
pub fn update_operator_ip_port(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn_operator_account: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    operator_admin: &Pubkey,
    ip_address: [u8; 4],
    port: u16,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new(*ncn_operator_account, false),
            AccountMeta::new_readonly(*ncn, false),
            AccountMeta::new_readonly(*operator, false),
            AccountMeta::new_readonly(*operator_admin, true),
        ],
        data: data(&NCNProgramInstruction::UpdateOperatorIpPort { ip_address, port }),
    }
}

/// `SnapshotVaultOperatorDelegation` — accounts: config, restaking_config,
/// ncn, operator, vault, vault_ncn_ticket, ncn_vault_ticket,
/// ncn_operator_state, vault_operator_delegation, snapshot(w),
/// vault_registry.
#[allow(clippy::too_many_arguments)]
pub fn snapshot_vault_operator_delegation(
    program_id: &Pubkey,
    config: &Pubkey,
    restaking_config: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    vault: &Pubkey,
    vault_ncn_ticket: &Pubkey,
    ncn_vault_ticket: &Pubkey,
    ncn_operator_state: &Pubkey,
    vault_operator_delegation: &Pubkey,
    snapshot: &Pubkey,
    vault_registry: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new_readonly(*restaking_config, false),
            AccountMeta::new_readonly(*ncn, false),
            AccountMeta::new_readonly(*operator, false),
            AccountMeta::new_readonly(*vault, false),
            AccountMeta::new_readonly(*vault_ncn_ticket, false),
            AccountMeta::new_readonly(*ncn_vault_ticket, false),
            AccountMeta::new_readonly(*ncn_operator_state, false),
            AccountMeta::new_readonly(*vault_operator_delegation, false),
            AccountMeta::new(*snapshot, false),
            AccountMeta::new_readonly(*vault_registry, false),
        ],
        data: data(&NCNProgramInstruction::SnapshotVaultOperatorDelegation {}),
    }
}
