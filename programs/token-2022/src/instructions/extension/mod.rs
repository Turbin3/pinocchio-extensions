use crate::{
    from_bytes_ref,
    state::{Mint, TokenAccount},
};
pub mod non_transferable;
pub use non_transferable::*;

pub const EXTENSIONS_PADDING: usize = 83;

pub const EXTENSION_START_OFFSET: usize = 1;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtensionType {
    /// Used as padding if the account size would otherwise be 355, same as a
    /// multisig
    Uninitialized,
    /// Includes transfer fee rate info and accompanying authorities to withdraw
    /// and set the fee
    TransferFeeConfig,
    /// Includes withheld transfer fees
    TransferFeeAmount,
    /// Includes an optional mint close authority
    MintCloseAuthority,
    /// Auditor configuration for confidential transfers
    ConfidentialTransferMint,
    /// State for confidential transfers
    ConfidentialTransferAccount,
    /// Specifies the default Account::state for new Accounts
    DefaultAccountState,
    /// Indicates that the Account owner authority cannot be changed
    ImmutableOwner,
    /// Require inbound transfers to have memo
    MemoTransfer,
    /// Indicates that the tokens from this mint can't be transferred
    NonTransferable,
    /// Tokens accrue interest over time,
    InterestBearingConfig,
    /// Locks privileged token operations from happening via CPI
    CpiGuard,
    /// Includes an optional permanent delegate
    PermanentDelegate,
    /// Indicates that the tokens in this account belong to a non-transferable
    /// mint
    NonTransferableAccount,
    /// Mint requires a CPI to a program implementing the "transfer hook"
    /// interface
    TransferHook,
    /// Indicates that the tokens in this account belong to a mint with a
    /// transfer hook
    TransferHookAccount,
    /// Includes encrypted withheld fees and the encryption public that they are
    /// encrypted under
    ConfidentialTransferFeeConfig,
    /// Includes confidential withheld transfer fees
    ConfidentialTransferFeeAmount,
    /// Mint contains a pointer to another account (or the same account) that
    /// holds metadata
    MetadataPointer,
    /// Mint contains token-metadata
    TokenMetadata,
    /// Mint contains a pointer to another account (or the same account) that
    /// holds group configurations
    GroupPointer,
    /// Mint contains token group configurations
    TokenGroup,
    /// Mint contains a pointer to another account (or the same account) that
    /// holds group member configurations
    GroupMemberPointer,
    /// Mint contains token group member configurations
    TokenGroupMember,
    /// Mint allowing the minting and burning of confidential tokens
    ConfidentialMintBurn,
    /// Tokens whose UI amount is scaled by a given amount
    ScaledUiAmount,
    /// Tokens where minting / burning / transferring can be paused
    Pausable,
    /// Indicates that the account belongs to a pausable mint
    PausableAccount,
}

impl ExtensionType {
    fn from_bytes(val: [u8; 2]) -> Option<Self> {
        let val = u16::from_le_bytes(val);
        let ext = match val {
            0 => ExtensionType::Uninitialized,
            1 => ExtensionType::TransferFeeConfig,
            2 => ExtensionType::TransferFeeAmount,
            3 => ExtensionType::MintCloseAuthority,
            4 => ExtensionType::ConfidentialTransferMint,
            5 => ExtensionType::ConfidentialTransferAccount,
            6 => ExtensionType::DefaultAccountState,
            7 => ExtensionType::ImmutableOwner,
            8 => ExtensionType::MemoTransfer,
            9 => ExtensionType::NonTransferable,
            10 => ExtensionType::InterestBearingConfig,
            11 => ExtensionType::CpiGuard,
            12 => ExtensionType::PermanentDelegate,
            13 => ExtensionType::NonTransferableAccount,
            14 => ExtensionType::TransferHook,
            15 => ExtensionType::TransferHookAccount,
            16 => ExtensionType::ConfidentialTransferFeeConfig,
            17 => ExtensionType::ConfidentialTransferFeeAmount,
            18 => ExtensionType::MetadataPointer,
            19 => ExtensionType::TokenMetadata,
            20 => ExtensionType::GroupPointer,
            21 => ExtensionType::TokenGroup,
            22 => ExtensionType::GroupMemberPointer,
            23 => ExtensionType::TokenGroupMember,
            24 => ExtensionType::ConfidentialMintBurn,
            25 => ExtensionType::ScaledUiAmount,
            26 => ExtensionType::Pausable,
            27 => ExtensionType::PausableAccount,
            _ => return None,
        };
        Some(ext)
    }

    pub fn to_bytes(&self) -> [u8; 2] {
        u16::to_le_bytes(*self as u16)
    }
}

pub const EXTENSION_LENGTH_LEN: usize = 2;
pub const EXTENSION_TYPE_LEN: usize = 2;

pub enum BaseState {
    Mint,
    TokenAccount,
}

pub trait Extension {
    const TYPE: ExtensionType;
    const BASE_LEN: usize;
    const BASE_STATE: BaseState;
}

pub fn get_extension_from_bytes<T: Extension + Clone + Copy>(acc_data_bytes: &[u8]) -> Option<&T> {
    let ext_bytes = match T::BASE_STATE {
        BaseState::Mint => {
            &acc_data_bytes[Mint::BASE_LEN + EXTENSIONS_PADDING + EXTENSION_START_OFFSET..]
        }
        BaseState::TokenAccount => {
            &acc_data_bytes[TokenAccount::BASE_LEN + EXTENSION_START_OFFSET..]
        }
    };
    let mut start = 0;
    let end = ext_bytes.len();
    while start < end {
        let ext_type_idx = start;
        let ext_len_idx = ext_type_idx + 2;
        let ext_data_idx = ext_len_idx + EXTENSION_LENGTH_LEN;

        let ext_type: [u8; 2] = ext_bytes[ext_type_idx..ext_type_idx + EXTENSION_TYPE_LEN]
            .try_into()
            .ok()?;
        let ext_type = ExtensionType::from_bytes(ext_type)?;
        let ext_len: [u8; 2] = ext_bytes[ext_len_idx..ext_len_idx + EXTENSION_LENGTH_LEN]
            .try_into()
            .ok()?;

        let ext_len = u16::from_le_bytes(ext_len);

        if ext_type == T::TYPE && ext_len as usize == T::BASE_LEN {
            return Some(unsafe {
                from_bytes_ref(&ext_bytes[ext_data_idx..ext_data_idx + T::BASE_LEN])
            });
        }

        start = start + EXTENSION_TYPE_LEN + EXTENSION_LENGTH_LEN + ext_len as usize;
    }
    None
}

pub fn get_extension_data_bytes_for_variable_pack<T: Extension + Clone>(
    acc_data_bytes: &[u8],
) -> Option<&[u8]> {
    let ext_bytes = match T::BASE_STATE {
        BaseState::Mint => {
            &acc_data_bytes[Mint::BASE_LEN + EXTENSIONS_PADDING + EXTENSION_START_OFFSET..]
        }
        BaseState::TokenAccount => {
            &acc_data_bytes[TokenAccount::BASE_LEN + EXTENSION_START_OFFSET..]
        }
    };
    let mut start = 0;
    let end = ext_bytes.len();
    while start < end {
        let ext_type_idx = start;
        let ext_len_idx = ext_type_idx + 2;
        let ext_data_idx = ext_len_idx + EXTENSION_LENGTH_LEN;

        let ext_type: [u8; 2] = ext_bytes[ext_type_idx..ext_type_idx + EXTENSION_TYPE_LEN]
            .try_into()
            .ok()?;

        let ext_type = ExtensionType::from_bytes(ext_type)?;
        let ext_len: [u8; 2] = ext_bytes[ext_len_idx..ext_len_idx + EXTENSION_LENGTH_LEN]
            .try_into()
            .ok()?;

        let ext_len = u16::from_le_bytes(ext_len);

        if ext_type == T::TYPE {
            return Some(&ext_bytes[ext_data_idx..ext_data_idx + ext_len as usize]);
        }

        start = start + EXTENSION_TYPE_LEN + EXTENSION_LENGTH_LEN + ext_len as usize;
    }
    None
}
