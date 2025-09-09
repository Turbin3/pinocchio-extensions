use crate::extensions::memo_transfer::state::{
    encode_instruction_data, RequiredMemoTransfersInstruction,
};
use core::slice::from_raw_parts;
use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

/// CPI helper to disable required memos on a token **account**.
///
/// Accounts (ordered):
///   0. `[writable]` token account to update
///   1. `[signer]`  account owner (single-owner flow)
///
/// Multisig flow is not covered in this minimal wrapper. Extend by adding
/// additional signer `AccountMeta`s and `AccountInfo`s when needed.
pub struct DisableRequiredTransferMemos<'a, 'b> {
    /// Token Account to update.
    pub account: &'a AccountInfo,
    /// The account's owner (must sign).
    pub owner: &'a AccountInfo,
    /// Token program ID (Token-2022).
    pub token_program: &'b Pubkey,
}

impl DisableRequiredTransferMemos<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly_signer(self.owner.key()),
        ];

        let data = encode_instruction_data(RequiredMemoTransfersInstruction::Disable);

        let instruction = Instruction {
            accounts: &account_metas,
            data: unsafe { from_raw_parts(data.as_ptr() as _, data.len()) },
            program_id: self.token_program,
        };

        invoke_signed(&instruction, &[self.account, self.owner], signers)
    }
}
