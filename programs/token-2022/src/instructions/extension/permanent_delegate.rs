use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{self, AccountMeta, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::instructions::permanent_delegate_instruction_data;

use super::get_extension_from_bytes;

/// State of the permanent delegate
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PermanentDelegate {
    /// Optional permanent delegate for transferring or burning tokens
    pub delegate: Pubkey,
}

impl super::Extension for PermanentDelegate {
    const TYPE: super::ExtensionType = super::ExtensionType::PermanentDelegate;
    const BASE_LEN: usize = Self::BASE_LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl PermanentDelegate {
    /// The length of the `PermanentDelegate` account data.
    pub const BASE_LEN: usize = core::mem::size_of::<PermanentDelegate>();

    /// Return a `PermanentDelegate` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&PermanentDelegate, ProgramError> {
        if !account_info.is_owned_by(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

// Instructions

pub struct InitializePermanentDelegate<'a, 'b> {
    /// The mint to initialize the permanent delegate
    pub mint: &'a AccountInfo,
    /// The public key for the account that can close the mint
    pub delegate: Pubkey,
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl InitializePermanentDelegate<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint.key())];

        let data = permanent_delegate_instruction_data(self.delegate);

        let instruction = instruction::Instruction {
            program_id: &self.token_program,
            accounts: &account_metas,
            data,
        };

        invoke_signed(&instruction, &[self.mint], signers)?;

        Ok(())
    }
}
