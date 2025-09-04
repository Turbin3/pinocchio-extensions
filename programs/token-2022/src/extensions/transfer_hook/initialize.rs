use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    program::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

use super::{
    write_optional_pubkey, INITIALIZE_DISCRIMINATOR, TRANSFER_HOOK_EXTENSION_DISCRIMINATOR,
};
use crate::{write_bytes, UNINIT_BYTE};

/// Initialize a new mint with a transfer hook program.
///
/// This instruction must be called before `InitializeMint`.
///
/// ### Accounts:
///   0. `[WRITE]` The mint to initialize.
pub struct TransferHookInitialize<'a, 'b> {
    /// The mint to initialize with transfer hook extension.
    pub mint: &'a AccountInfo,
    /// Optional authority for the transfer hook.
    pub authority: Option<&'a Pubkey>,
    /// Optional transfer hook program ID.
    pub transfer_hook_program_id: Option<&'a Pubkey>,
    /// Token program (must be Token-2022).
    pub token_program: &'b Pubkey,
}

impl TransferHookInitialize<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // - [0]: main discriminator (1 byte, u8) = 36 (TransferHookExtension)
        // - [1]: sub discriminator (1 byte, u8) = 0 (Initialize)
        // - [2]: authority presence flag + authority (1 or 33 bytes)
        // - [2+auth_len]: program_id presence flag + program_id (1 or 33 bytes)
        let mut instruction_data = [UNINIT_BYTE; 68]; // Max: 2(flags) + 33(authority) + 33(program_id)
        let mut offset = 0;

        // Set main discriminator
        write_bytes(
            &mut instruction_data[offset..offset + 1],
            &[TRANSFER_HOOK_EXTENSION_DISCRIMINATOR],
        );
        offset += 1;

        // Set sub discriminator
        write_bytes(
            &mut instruction_data[offset..offset + 1],
            &[INITIALIZE_DISCRIMINATOR],
        );
        offset += 1;

        // Write authority (OptionalNonZeroPubkey)
        let auth_len = write_optional_pubkey(&mut instruction_data[offset..], self.authority);
        offset += auth_len;

        // Write transfer_hook_program_id (OptionalNonZeroPubkey)
        let program_id_len = write_optional_pubkey(
            &mut instruction_data[offset..],
            self.transfer_hook_program_id,
        );
        offset += program_id_len;

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, offset) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
