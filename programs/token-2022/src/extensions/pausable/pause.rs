use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{
    extensions::pausable::{PAUSABLE_EXTENSION, PAUSE},
    write_bytes, UNINIT_BYTE,
};

/// Pause minting, burning, and transferring for the mint.
///
/// ### Accounts:
///   0. `[WRITE]` The mint to update.
///   1. `[SIGNER]` The mint's pause authority.
pub struct Pause<'a, 'b> {
    /// Mint account
    pub mint: &'a AccountInfo,
    /// Authority
    pub authority: &'a AccountInfo,
    /// Token program
    pub token_program: &'b Pubkey,
}

impl Pause<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data layout:
        // - [0]: token_instruction_type: (1 byte, u8)
        // - [1]: instruction_type: (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 2];

        // Set discriminator as u8 at offset [0] for PausableExtension token instruction
        write_bytes(&mut instruction_data, &[PAUSABLE_EXTENSION]);
        // Set discriminator as u8 at offset [1] for Pause instruction
        write_bytes(&mut instruction_data[1..2], &[PAUSE]);

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke(&instruction, &[self.mint, self.authority])
    }
}
