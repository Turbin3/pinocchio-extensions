use core::{
    mem::MaybeUninit,
    slice::{self, from_raw_parts},
};

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_with_bounds,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{instructions::MAX_MULTISIG_SIGNERS, write_bytes, UNINIT_BYTE};

/// Set transfer fee. Only supported for mints that include the
/// `TransferFeeConfig` extension.
///
/// Accounts expected by this instruction:
///
///   * Single authority
///   0. `[writable]` The mint.
///   1. `[signer]` The mint's fee account owner.
///
///   * Multisignature authority
///   0. `[writable]` The mint.
///   1. `[]` The mint's multisignature fee account owner.
///   2. `..2+M` `[signer]` M signer accounts.
pub struct SetTransferFee<'a, 'b, 'c>
where
    'a: 'b,
{
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// The fee account's owner.
    pub fee_account_authority: &'a AccountInfo,
    /// Amount of transfer collected as fees, expressed as basis points of
    /// the transfer amount
    pub transfer_fee_basis_points: u16,
    /// Maximum fee assessed on transfers
    pub maximum_fee: u64,
    /// The Signer accounts if `fee_account_authority` is a multisig
    pub signers: &'b [&'a AccountInfo],
    /// Token Program
    pub token_program: &'c Pubkey,
}

impl SetTransferFee<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let &Self {
            mint,
            fee_account_authority,
            transfer_fee_basis_points,
            maximum_fee,
            signers,
            token_program,
        } = self;

        if signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 2 + signers.len();

        // Account metadata
        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut acc_metas = [UNINIT_META; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_metas` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(mint.key()));
            // - Index 1 is always present
            if signers.is_empty() {
                acc_metas
                    .get_unchecked_mut(1)
                    .write(AccountMeta::readonly_signer(fee_account_authority.key()));
            } else {
                acc_metas
                    .get_unchecked_mut(1)
                    .write(AccountMeta::readonly(fee_account_authority.key()));
            }
        }

        for (account_meta, signer) in acc_metas[2..].iter_mut().zip(signers.iter()) {
            account_meta.write(AccountMeta::readonly_signer(signer.key()));
        }

        // Instruction data layout:
        // -  [0]: instruction TransferFeeExtension discriminator (1 byte, u8)
        // -  [1]: instruction SetTransferFee discriminator (1 byte, u8)
        // -  [2..4]: transfer_fee_basis_points (2 bytes, u16)
        // -  [4..12]: maximum_fee (8 bytes, u64)
        let mut instruction_data = [UNINIT_BYTE; 12];

        // -  [0]: instruction TransferFeeExtension discriminator (1 byte, u8)
        // -  [1]: instruction SetTransferFee discriminator (1 byte, u8)
        write_bytes(&mut instruction_data, &[26, 5]);

        // Set amount as u64 at offset [2..4]
        write_bytes(
            &mut instruction_data[2..4],
            transfer_fee_basis_points.to_le_bytes().as_ref(),
        );
        // Set fee as u64 at offset [4..12]
        write_bytes(
            &mut instruction_data[4..12],
            maximum_fee.to_le_bytes().as_ref(),
        );

        let instruction = Instruction {
            program_id: token_program,
            accounts: unsafe { slice::from_raw_parts(acc_metas.as_ptr() as _, num_accounts) },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 12) },
        };

        // Account info array
        const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_infos` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_infos.get_unchecked_mut(0).write(mint);
            // - Index 1 is always present
            acc_infos.get_unchecked_mut(1).write(fee_account_authority);
        }

        // Fill signer accounts
        for (account_info, signer) in acc_infos[2..].iter_mut().zip(signers.iter()) {
            account_info.write(signer);
        }

        invoke_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(&instruction, unsafe {
            slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts)
        })
    }
}
