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
        // - [2..34]: program_id (32 bytes, OptionalNonZeroPubkey - zeros if None)
        let mut instruction_data = [UNINIT_BYTE; 34]; // Fixed: 2(discriminators) + 32(program_id)
         // Set discriminators at fixed positions
        write_bytes(
            &mut instruction_data[0..2],
            &[TRANSFER_HOOK_EXTENSION_DISCRIMINATOR, UPDATE_DISCRIMINATOR],
        );

        // Write new_transfer_hook_program_id at fixed position [2..34]
        write_optional_pubkey(
            &mut instruction_data[2..],
            self.new_transfer_hook_program_id,
        );

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 34) },
        };

        invoke_signed(&instruction, &[self.mint, self.authority], signers)
    }
}
