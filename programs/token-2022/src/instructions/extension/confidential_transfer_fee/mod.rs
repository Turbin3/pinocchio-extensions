pub mod consts;
pub mod initialize_confidential_transfer_fee_config;
pub mod state;

pub use consts::*;
pub use initialize_confidential_transfer_fee_config::*;
pub use state::*;

use pinocchio::pubkey::Pubkey;

extern crate alloc;

use super::{get_extension_from_bytes, EncryptedBalance, PodElGamalCiphertext, PodElGamalPubkey};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConfidentialTransferFeeConfig {
    /// Optional authority to set the withdraw withheld authority ElGamal key
    pub authority: Pubkey,

    /// Withheld fees from accounts must be encrypted with this ElGamal key.
    ///
    /// Note that whoever holds the ElGamal private key for this ElGamal public
    /// key has the ability to decode any withheld fee amount that are
    /// associated with accounts. When combined with the fee parameters, the
    /// withheld fee amounts can reveal information about transfer amounts.
    pub withdraw_withheld_authority_elgamal_pubkey: PodElGamalPubkey,

    /// If `false`, the harvest of withheld tokens to mint is rejected.
    pub harvest_to_mint_enabled: u8,

    /// Withheld confidential transfer fee tokens that have been moved to the
    /// mint for withdrawal.
    pub withheld_amount: PodElGamalCiphertext,
}

impl super::Extension for ConfidentialTransferFeeConfig {
    const TYPE: super::ExtensionType = super::ExtensionType::ConfidentialTransferFeeConfig;
    const BASE_LEN: usize = Self::BASE_LEN;
    const BASE_STATE: super::BaseState = super::BaseState::Mint;
}

impl ConfidentialTransferFeeConfig {
    /// The length of the `ConfidentialTransferFeeConfig` account data.
    pub const BASE_LEN: usize = core::mem::size_of::<ConfidentialTransferFeeConfig>();

    /// Return a `ConfidentialTransferFeeConfig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &pinocchio::account_info::AccountInfo,
    ) -> Result<&ConfidentialTransferFeeConfig, pinocchio::program_error::ProgramError> {
        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(pinocchio::program_error::ProgramError::InvalidAccountData)
    }
}

/// Confidential transfer fee
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConfidentialTransferFeeAmount {
    /// Amount withheld during confidential transfers, to be harvest to the mint
    pub withheld_amount: EncryptedBalance,
}

impl super::Extension for ConfidentialTransferFeeAmount {
    const TYPE: super::ExtensionType = super::ExtensionType::ConfidentialTransferFeeAmount;
    const BASE_LEN: usize = Self::BASE_LEN;
    const BASE_STATE: super::BaseState = super::BaseState::TokenAccount;
}

impl ConfidentialTransferFeeAmount {
    /// The length of the `ConfidentialTransferFeeAmount` account data.
    pub const BASE_LEN: usize = core::mem::size_of::<ConfidentialTransferFeeAmount>();

    /// Return a `ConfidentialTransferFeeAmount` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info_unchecked(
        account_info: &pinocchio::account_info::AccountInfo,
    ) -> Result<&ConfidentialTransferFeeAmount, pinocchio::program_error::ProgramError> {
        get_extension_from_bytes(unsafe { account_info.borrow_data_unchecked() })
            .ok_or(pinocchio::program_error::ProgramError::InvalidAccountData)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WithheldTokensInfo {
    /// The available balance
    pub withheld_amount: PodElGamalCiphertext,
}
