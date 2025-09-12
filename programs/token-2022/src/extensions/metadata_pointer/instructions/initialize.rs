use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::UNINIT_BYTE;
use crate::extensions::metadata_pointer::state::encode_initialize_instruction_data;

pub struct MetadataPointerInitialize<'a, 'b> {
    /// The mint to initialize with the metadata pointer extension.
    pub mint: &'a AccountInfo,
    /// Optional authority that can later update the metadata address.
    pub authority: Option<&'a Pubkey>,
    /// Optional initial metadata address.
    pub metadata_address: Option<&'a Pubkey>,
    /// Token program (Token-2022).
    pub token_program: &'b Pubkey,
}

impl MetadataPointerInitialize<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account meta layout
        let account_metas = [AccountMeta::writable(self.mint.key())];

        // Encode: [39, 0, authority(32), metadata_address(32)]
        let mut instruction_data = [UNINIT_BYTE; 66];
        let written = encode_initialize_instruction_data(
            &mut instruction_data,
            self.authority,
            self.metadata_address,
        );

        let ix = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, written) },
        };

        invoke_signed(&ix, &[self.mint], signers)
    }
}
