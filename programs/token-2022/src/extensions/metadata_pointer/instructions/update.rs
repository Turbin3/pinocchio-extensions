// programs/token-2022/src/extensions/metadata_pointer/instructions/update.rs

use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::UNINIT_BYTE;
use crate::extensions::metadata_pointer::state::encode_update_instruction_data;

pub struct MetadataPointerUpdate<'a, 'b> {
    /// The mint to update.
    pub mint: &'a AccountInfo,
    /// Current metadata pointer authority (must sign).
    pub authority: &'a AccountInfo,
    /// New metadata address (None to clear).
    pub new_metadata_address: Option<&'a Pubkey>,
    /// Token program (Token-2022).
    pub token_program: &'b Pubkey,
}

impl MetadataPointerUpdate<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account meta layout
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Encode: [39, 1, new_metadata_address(32)]
        let mut instruction_data = [UNINIT_BYTE; 34];
        let written =
            encode_update_instruction_data(&mut instruction_data, self.new_metadata_address);

        let ix = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, written) },
        };

        invoke_signed(&ix, &[self.mint, self.authority], signers)
    }
}
