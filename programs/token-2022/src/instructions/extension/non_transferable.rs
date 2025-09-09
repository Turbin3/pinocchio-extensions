use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::instructions::get_extension_from_bytes;

/// Discriminator for the InitializeNonTransferableMint.
pub const INITIALIZE_NON_TRANSFERABLE_MINT: u8 = 32;

/// State of the non-transferable for mint
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonTransferable;

impl super::Extension for NonTransferable {
    const TYPE: super::ExtensionType = super::ExtensionType::NonTransferable;
    const BASE_LEN: usize = Self::BASE_LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl NonTransferable {
    /// The length of the `NonTransferable` account data.
    pub const BASE_LEN: usize = core::mem::size_of::<NonTransferable>();

    /// Return a `NonTransferable` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&NonTransferable, ProgramError> {
        if !account_info.is_owned_by(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

/// State of the non-transferable for token account
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonTransferableAccount;

impl super::Extension for NonTransferableAccount {
    const TYPE: super::ExtensionType = super::ExtensionType::NonTransferableAccount;
    const BASE_LEN: usize = Self::BASE_LEN;
    const BASE_STATE: super::BaseState = super::BaseState::TokenAccount;
}

impl NonTransferableAccount {
    /// The length of the `NonTransferableAccount` account data.
    pub const BASE_LEN: usize = core::mem::size_of::<NonTransferableAccount>();

    /// Return a `NonTransferableAccount` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&NonTransferableAccount, ProgramError> {
        if !account_info.is_owned_by(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(ProgramError::InvalidAccountData)
    }
}

// Instructions
pub struct InitializeNonTransferableMint<'a, 'b> {
    /// The mint to initialize the non-transferable
    pub mint: &'a AccountInfo,
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl InitializeNonTransferableMint<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [AccountMeta::writable(self.mint.key())];

        // Instruction data Layout:
        //[0] u8: instruction discriminator

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: &[INITIALIZE_NON_TRANSFERABLE_MINT],
        };

        invoke_signed(&instruction, &[self.mint], signers)?;

        Ok(())
    }
}
