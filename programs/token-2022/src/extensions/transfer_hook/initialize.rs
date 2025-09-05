use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
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
        // - [2..34]: authority (32 bytes, OptionalNonZeroPubkey - zeros if None)
        // - [34..66]: program_id (32 bytes, OptionalNonZeroPubkey - zeros if None)
        let mut instruction_data = [UNINIT_BYTE; 66]; // Fixed: 2(discriminators) + 32(authority) + 32(program_id)
        // Set discriminators at fixed positions
        write_bytes(
            &mut instruction_data[0..2],
            &[
                TRANSFER_HOOK_EXTENSION_DISCRIMINATOR,
                INITIALIZE_DISCRIMINATOR,
            ],
        );

        // Write authority at fixed position [2..34]
        write_optional_pubkey(&mut instruction_data[2..], self.authority);

        // Write transfer_hook_program_id at fixed position [34..66]
        write_optional_pubkey(&mut instruction_data[34..], self.transfer_hook_program_id);

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 66) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
