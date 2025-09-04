use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

use super::{write_optional_pubkey, TRANSFER_HOOK_EXTENSION_DISCRIMINATOR, UPDATE_DISCRIMINATOR};
use crate::{write_bytes, UNINIT_BYTE};

/// Update the transfer hook program id for a mint with the TransferHook extension.
///
/// **Note**: This implementation supports single-authority updates only.
/// Multisig authorities are not currently supported by this wrapper.
///
/// ### Accounts:
///   0. `[WRITE]` The mint.
///   1. `[SIGNER]` The transfer hook authority (single signer).
pub struct TransferHookUpdate<'a, 'b> {
    /// The mint to update.
    pub mint: &'a AccountInfo,
    /// Current transfer hook authority (must be signer).
    pub authority: &'a AccountInfo,
    /// New transfer hook program ID (None to disable).
    pub new_transfer_hook_program_id: Option<&'a Pubkey>,
    /// Token program (must be Token-2022).
    pub token_program: &'b Pubkey,
}

impl TransferHookUpdate<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data layout:
        // - [0]: main discriminator (1 byte, u8) = 36 (TransferHookExtension)
        // - [1]: sub discriminator (1 byte, u8) = 1 (Update)
        // - [2]: program_id presence flag + program_id (1 or 33 bytes)
        let mut instruction_data = [UNINIT_BYTE; 35]; // Max: 2(flags) + 33(program_id)
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
            &[UPDATE_DISCRIMINATOR],
        );
        offset += 1;

        // Write new_transfer_hook_program_id
        let program_id_len = write_optional_pubkey(
            &mut instruction_data[offset..],
            self.new_transfer_hook_program_id,
        );
        offset += program_id_len;

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, offset) },
        };

        invoke_signed(&instruction, &[self.mint, self.authority], signers)
    }
}
