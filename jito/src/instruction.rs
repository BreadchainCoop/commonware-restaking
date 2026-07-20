//! Manual construction of the NCN program's `VerifyCertificate` instruction
//! (INTERFACES.md §2 — the FROZEN Phase 1 shape).
//!
//! Args (borsh, in order): `digest: [u8; 32]`, `aggregated_g2: [u8; 64]`
//! (compressed), `aggregated_signature: [u8; 32]` (compressed),
//! `operators_signature_bitmap: Vec<u8>`, `expected_generation: u64`.
//! Accounts: `[ncn_config, ncn, snapshot, restaking_config]` — ALL read-only,
//! no signer beyond the fee payer. STATELESS: mutates nothing.

use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;

/// The instruction discriminator prefix for `VerifyCertificate`.
///
/// FROZEN against Phase 1 (merged to jito-ncn-program `main`): the program
/// uses 1-byte borsh enum indices (NOT 8-byte anchor-style discriminators)
/// and `VerifyCertificate` sits at index 10 of `NCNProgramInstruction` — the
/// `CastVote` slot it replaced. The generated client
/// (`clients/rust/ncn_program/.../verify_certificate.rs`) emits the same
/// `VerifyCertificateInstructionData { discriminator: 10 }`. The
/// `discriminator_matches_generated_client` test below pins the FULL
/// instruction-data encoding (discriminator ‖ borsh(args)) byte-for-byte
/// against `ncn_program_core::instruction::NCNProgramInstruction`, the enum
/// shank generates that client from, so any upstream reordering fails loudly
/// on the next dep bump.
pub const VERIFY_CERTIFICATE_DISCRIMINATOR: &[u8] = &[10];

/// The `VerifyCertificate` argument block (post-discriminator instruction
/// data), byte-compatible with the borsh encoding shank/kinobi generate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyCertificateArgs {
    /// The certified 32-byte message digest.
    pub digest: [u8; 32],
    /// Aggregated G2 public key of the signers (compressed).
    pub aggregated_g2: [u8; 64],
    /// Aggregated G1 signature (compressed).
    pub aggregated_signature: [u8; 32],
    /// LSB-first signer bitmap over ON-CHAIN operator indices: operator `i`
    /// signed iff byte `i >> 3` bit `i & 7` is set. Length =
    /// `operators_registered.div_ceil(8)`.
    pub operators_signature_bitmap: Vec<u8>,
    /// Snapshot generation the certificate was assembled against, captured at
    /// assembly time (stale generations are rejected on-chain).
    pub expected_generation: u64,
}

impl VerifyCertificateArgs {
    /// Serializes the full instruction data: discriminator ‖ borsh(args).
    ///
    /// Hand-rolled borsh (fixed arrays = raw bytes, `Vec<u8>` = u32-LE length
    /// prefix + bytes, `u64` = LE) so the submitter does not need a borsh
    /// dependency; cross-checked against real borsh in the tests below.
    pub fn instruction_data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(
            VERIFY_CERTIFICATE_DISCRIMINATOR.len()
                + 32
                + 64
                + 32
                + 4
                + self.operators_signature_bitmap.len()
                + 8,
        );
        data.extend_from_slice(VERIFY_CERTIFICATE_DISCRIMINATOR);
        data.extend_from_slice(&self.digest);
        data.extend_from_slice(&self.aggregated_g2);
        data.extend_from_slice(&self.aggregated_signature);
        let bitmap_len = u32::try_from(self.operators_signature_bitmap.len())
            .expect("bitmap length exceeds u32 (impossible: max 256 operators)");
        data.extend_from_slice(&bitmap_len.to_le_bytes());
        data.extend_from_slice(&self.operators_signature_bitmap);
        data.extend_from_slice(&self.expected_generation.to_le_bytes());
        data
    }
}

/// The read-only account set of `VerifyCertificate` (INTERFACES.md §2 order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifyCertificateAccounts {
    /// NCN program `Config` PDA.
    pub ncn_config: Pubkey,
    /// The NCN account.
    pub ncn: Pubkey,
    /// NCN program `Snapshot` PDA.
    pub snapshot: Pubkey,
    /// jito-restaking global config PDA.
    pub restaking_config: Pubkey,
}

/// Builds the `VerifyCertificate` instruction: 4 read-only accounts, manual
/// data encoding. The fee payer is added by the transaction, not here.
pub fn verify_certificate_instruction(
    program_id: &Pubkey,
    accounts: &VerifyCertificateAccounts,
    args: &VerifyCertificateArgs,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(accounts.ncn_config, false),
            AccountMeta::new_readonly(accounts.ncn, false),
            AccountMeta::new_readonly(accounts.snapshot, false),
            AccountMeta::new_readonly(accounts.restaking_config, false),
        ],
        data: args.instruction_data(),
    }
}

/// Builds the LSB-first on-chain signer bitmap from the signers' ON-CHAIN
/// operator indices: operator `i` signed iff byte `i >> 3` bit `i & 7` is 1
/// (INTERFACES.md §1). Indices at or above `operators_registered` are
/// rejected — a bitmap claiming an unregistered index can never verify.
///
/// Delegates to `ncn_program_core::utils::create_signer_bitmap` (which takes
/// the NON-signer complement) so the emitted bytes are bit-exact with the
/// program's canonical builder — including its convention that padding bits
/// beyond `operators_registered` in the last byte are SET (all-ones
/// initialization).
pub fn signer_bitmap(
    signer_operator_indices: &[u64],
    operators_registered: u64,
) -> Result<Vec<u8>, anyhow::Error> {
    let total = usize::try_from(operators_registered)
        .map_err(|_| anyhow::anyhow!("operators_registered overflows usize"))?;
    let mut signed = vec![false; total];
    for &index in signer_operator_indices {
        if index >= operators_registered {
            return Err(anyhow::anyhow!(
                "signer operator index {index} out of range (operators_registered = {operators_registered})"
            ));
        }
        signed[index as usize] = true;
    }
    let non_signers: Vec<usize> = (0..total).filter(|&i| !signed[i]).collect();
    Ok(ncn_program_core::utils::create_signer_bitmap(
        &non_signers,
        total,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    // The derive macro emits `borsh::`-prefixed paths, so the renamed dev-dep
    // must be visible under its canonical name inside this module.
    use borsh10 as borsh;
    use borsh10::BorshSerialize;

    /// The args struct as shank/kinobi's generated client would serialize it
    /// (borsh 0.10, matching the NCN program's borsh pin).
    #[derive(BorshSerialize)]
    struct BorshArgs {
        digest: [u8; 32],
        aggregated_g2: [u8; 64],
        aggregated_signature: [u8; 32],
        operators_signature_bitmap: Vec<u8>,
        expected_generation: u64,
    }

    fn sample_args() -> VerifyCertificateArgs {
        VerifyCertificateArgs {
            digest: [0xAB; 32],
            aggregated_g2: core::array::from_fn(|i| i as u8),
            aggregated_signature: [0x11; 32],
            operators_signature_bitmap: vec![0b0000_0101, 0b1000_0000],
            expected_generation: 42,
        }
    }

    #[test]
    fn hand_rolled_encoding_matches_borsh() {
        let args = sample_args();
        let borsh = BorshArgs {
            digest: args.digest,
            aggregated_g2: args.aggregated_g2,
            aggregated_signature: args.aggregated_signature,
            operators_signature_bitmap: args.operators_signature_bitmap.clone(),
            expected_generation: args.expected_generation,
        };
        let mut expected = VERIFY_CERTIFICATE_DISCRIMINATOR.to_vec();
        expected.extend(borsh.try_to_vec().expect("borsh serializes"));
        assert_eq!(args.instruction_data(), expected);
    }

    /// FROZEN: pins the const AND the complete instruction-data encoding to
    /// `ncn_program_core::instruction::NCNProgramInstruction` — the enum the
    /// shank/kinobi client generation reads, whose borsh serialization IS
    /// `discriminator ‖ args`. The generated client's
    /// `VerifyCertificateInstructionData` carries the same byte (10). If a
    /// dep bump reorders the enum, this fails before anything reaches a
    /// validator.
    #[test]
    fn discriminator_matches_generated_client() {
        let args = sample_args();
        let program_enum =
            ncn_program_core::instruction::NCNProgramInstruction::VerifyCertificate {
                digest: args.digest,
                aggregated_g2: args.aggregated_g2,
                aggregated_signature: args.aggregated_signature,
                operators_signature_bitmap: args.operators_signature_bitmap.clone(),
                expected_generation: args.expected_generation,
            };
        let canonical = program_enum.try_to_vec().expect("enum serializes");
        // First byte(s): the discriminator const, frozen at the generated
        // client's value (1-byte kinobi/borsh enum index, value 10).
        assert_eq!(VERIFY_CERTIFICATE_DISCRIMINATOR, &canonical[..1]);
        assert_eq!(VERIFY_CERTIFICATE_DISCRIMINATOR, &[10]);
        assert_eq!(VERIFY_CERTIFICATE_DISCRIMINATOR.len(), 1);
        // Full differential: our hand-rolled encoding == the program enum's
        // borsh encoding, byte for byte.
        assert_eq!(args.instruction_data(), canonical);
    }

    #[test]
    fn instruction_accounts_are_read_only_in_interface_order() {
        let accounts = VerifyCertificateAccounts {
            ncn_config: Pubkey::new_unique(),
            ncn: Pubkey::new_unique(),
            snapshot: Pubkey::new_unique(),
            restaking_config: Pubkey::new_unique(),
        };
        let program_id = Pubkey::new_unique();
        let ix = verify_certificate_instruction(&program_id, &accounts, &sample_args());
        assert_eq!(ix.program_id, program_id);
        let metas: Vec<(Pubkey, bool, bool)> = ix
            .accounts
            .iter()
            .map(|m| (m.pubkey, m.is_signer, m.is_writable))
            .collect();
        assert_eq!(
            metas,
            vec![
                (accounts.ncn_config, false, false),
                (accounts.ncn, false, false),
                (accounts.snapshot, false, false),
                (accounts.restaking_config, false, false),
            ]
        );
    }

    #[test]
    fn signer_bitmap_is_lsb_first_per_operator_index() {
        // Operators 0, 2 and 15 of 16 signed (no padding bits at 16).
        let bitmap = signer_bitmap(&[0, 2, 15], 16).unwrap();
        assert_eq!(bitmap, vec![0b0000_0101, 0b1000_0000]);
        // 9 operators round up to 2 bytes; bits 9..15 are PADDING and stay
        // set, matching the program builder's all-ones initialization.
        let bitmap = signer_bitmap(&[8], 9).unwrap();
        assert_eq!(bitmap, vec![0b0000_0000, 0b1111_1111]);
        // Out-of-range index rejected.
        assert!(signer_bitmap(&[16], 16).is_err());
        // Empty signer set: registered bits cleared, padding bits set.
        assert_eq!(signer_bitmap(&[], 3).unwrap(), vec![0b1111_1000]);
    }

    #[test]
    fn signer_bitmap_matches_program_core_non_signer_builder() {
        // ncn-program-core builds the same bitmap from the complement (the
        // NON-signer indices); the two constructions must agree bit-for-bit.
        let total = 20usize;
        let signers: Vec<u64> = vec![0, 3, 7, 11, 19];
        let non_signers: Vec<usize> = (0..total)
            .filter(|i| !signers.contains(&(*i as u64)))
            .collect();
        let ours = signer_bitmap(&signers, total as u64).unwrap();
        let theirs = ncn_program_core::utils::create_signer_bitmap(&non_signers, total);
        assert_eq!(ours, theirs);
    }
}
