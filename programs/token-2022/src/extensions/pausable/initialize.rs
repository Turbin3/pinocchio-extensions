use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{
    extensions::pausable::{INITIALIZE, PAUSABLE_EXTENSION},
    write_bytes, UNINIT_BYTE,
};

/// Initialize the pausable extension for the given mint account.
///
/// Fails if the account has already been initialized, so must be called
/// before `InitializeMint`.
///
/// ### Accounts:
///   0. `[WRITE]` The mint account to initialize.
///
///  ### Data:
///   0. `authority` Pubkey of the mint's pause authority.
pub struct Initialize<'a, 'b> {
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// Authority
    pub authority: &'a Pubkey,
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl Initialize<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // - [0]: token_instruction_type: (1 byte, u8)
        // - [1]: instruction_type: (1 byte, u8)
        // - [2..34] authority (32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 34];

        // Set discriminator as u8 at offset [0] for PausableExtension token instruction
        write_bytes(&mut instruction_data, &[PAUSABLE_EXTENSION]);
        // Set discriminator as u8 at offset [1] for Initialize instruction
        write_bytes(&mut instruction_data[1..2], &[INITIALIZE]);
        // Set authority as Pubkey at offset [2..34]
        write_bytes(&mut instruction_data[2..34], self.authority);

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke(&instruction, &[self.mint])
    }
}
