use core::{
    mem::MaybeUninit,
    slice::{self},
};

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed_with_bounds,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::instructions::{transfer_checked_with_fee_instruction_data, MAX_MULTISIG_SIGNERS};

/// Transfer, providing expected mint information and fees
///
/// This instruction succeeds if the mint has no configured transfer fee
/// and the provided fee is 0. This allows applications to use
/// `TransferCheckedWithFee` with any mint.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The source account. May include the
///      `TransferFeeAmount` extension.
///   1. `[]` The token mint. May include the `TransferFeeConfig` extension.
///   2. `[writable]` The destination account. May include the
///      `TransferFeeAmount` extension.
///   3. `[signer]` The source account's owner/delegate.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The source account.
///   1. `[]` The token mint.
///   2. `[writable]` The destination account.
///   3. `[]` The source account's multisignature owner/delegate.
///   4. `..4+M` `[signer]` M signer accounts.
pub struct TransferCheckedWithFee<'a, 'b, 'c>
where
    'a: 'b,
{
    /// Source Account
    pub source_account: &'a AccountInfo,
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// Destination Account
    pub destination: &'a AccountInfo,
    /// The source account's owner/delegate.
    pub source_account_authority: &'a AccountInfo,
    /// The amount of tokens to transfer.
    pub amount: u64,
    /// Expected number of base 10 digits to the right of the decimal place.
    pub decimals: u8,
    /// Expected fee assessed on this transfer, calculated off-chain based
    /// on the `transfer_fee_basis_points` and `maximum_fee` of the mint.
    /// May be 0 for a mint without a configured transfer fee.
    pub fee: u64,
    /// The Signer accounts if `source_account_authority` is a multisig
    pub signers: &'b [&'a AccountInfo],
    /// Token Program
    pub token_program: &'c Pubkey,
}

impl TransferCheckedWithFee<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            source_account,
            mint,
            destination,
            source_account_authority,
            amount,
            decimals,
            fee,
            signers: account_signers,
            token_program,
        } = self;

        if account_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 4 + account_signers.len();

        // Account metadata
        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut acc_metas = [UNINIT_META; 4 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_metas` is sized to 4 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(source_account.key()));
            // - Index 1 is always present
            acc_metas
                .get_unchecked_mut(1)
                .write(AccountMeta::readonly(mint.key()));
            // - Index 2 is always present
            acc_metas
                .get_unchecked_mut(2)
                .write(AccountMeta::writable(destination.key()));
            // - Index 3 is always present
            if account_signers.is_empty() {
                acc_metas
                    .get_unchecked_mut(3)
                    .write(AccountMeta::readonly_signer(source_account_authority.key()));
            } else {
                acc_metas
                    .get_unchecked_mut(3)
                    .write(AccountMeta::readonly(source_account_authority.key()));
            }
        }

        for (account_meta, signer) in acc_metas[4..].iter_mut().zip(account_signers.iter()) {
            account_meta.write(AccountMeta::readonly_signer(signer.key()));
        }

        let data = transfer_checked_with_fee_instruction_data(amount, decimals, fee);

        let instruction = Instruction {
            program_id: token_program,
            accounts: unsafe { slice::from_raw_parts(acc_metas.as_ptr() as _, num_accounts) },
            data,
        };

        // Account info array
        const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; 4 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_infos` is sized to 4 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_infos.get_unchecked_mut(0).write(source_account);
            // - Index 1 is always present
            acc_infos.get_unchecked_mut(1).write(mint);
            // - Index 2 is always present
            acc_infos.get_unchecked_mut(2).write(destination);
            // - Index 3 is always present
            acc_infos
                .get_unchecked_mut(3)
                .write(source_account_authority);
        }

        // Fill signer accounts
        for (account_info, signer) in acc_infos[4..].iter_mut().zip(account_signers.iter()) {
            account_info.write(signer);
        }

        invoke_signed_with_bounds::<{ 4 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
