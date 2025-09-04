use core::{mem::MaybeUninit, slice};

use pinocchio::{
    account_info::AccountInfo,
    cpi::{invoke_with_bounds, MAX_CPI_ACCOUNTS},
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::instructions::MAX_MULTISIG_SIGNERS;
/// Transfer all withheld tokens to an account. Signed by the mint's
/// withdraw withheld tokens authority.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[]` The token mint. Must include the `TransferFeeConfig`
///      extension.
///   1. `[writable]` The fee receiver account. Must include the
///      `TransferFeeAmount` extension and be associated with the provided
///      mint.
///   2. `[signer]` The mint's `withdraw_withheld_authority`.
///   3. `..3+N` `[writable]` The source accounts to withdraw from.
///
///   * Multisignature owner/delegate
///   0. `[]` The token mint.
///   1. `[writable]` The destination account.
///   2. `[]` The mint's multisig `withdraw_withheld_authority`.
///   3. `..3+M` `[signer]` M signer accounts.
///   4. `3+M+1..3+M+N` `[writable]` The source accounts to withdraw from.
pub struct WithdrawWithheldTokensFromAccounts<'a, 'b, 'c>
where
    'a: 'b,
{
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// Destination Account
    pub destination: &'a AccountInfo,
    /// Withdraw withheld authority
    pub withdraw_withheld_authority: &'a AccountInfo,
    /// Number of token accounts harvested
    pub num_token_accounts: u8,
    /// Signer Accounts
    pub signers: &'b [&'a AccountInfo],
    /// Source Accounts
    pub source_accounts: &'b [&'a AccountInfo],
    /// Token Program
    pub token_program: &'c Pubkey,
}

impl WithdrawWithheldTokensFromAccounts<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        let &Self {
            mint,
            destination,
            withdraw_withheld_authority,
            num_token_accounts,
            signers,
            source_accounts,
            token_program,
        } = self;

        if signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        if source_accounts.len() != num_token_accounts as usize {
            return Err(ProgramError::InvalidArgument);
        }

        if (3 + num_token_accounts as usize + signers.len()) > MAX_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 3 + num_token_accounts as usize + signers.len();

        // Account metadata
        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut acc_metas = [UNINIT_META; MAX_CPI_ACCOUNTS];

        unsafe {
            // SAFETY:
            // - `account_metas` is sized to 3 + num_token_accounts + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::readonly(mint.key()));
            // - Index 1 is always present
            acc_metas
                .get_unchecked_mut(1)
                .write(AccountMeta::writable(destination.key()));
            // - Index 2 is always present
            if signers.is_empty() {
                acc_metas
                    .get_unchecked_mut(2)
                    .write(AccountMeta::readonly_signer(
                        withdraw_withheld_authority.key(),
                    ));
            } else {
                acc_metas
                    .get_unchecked_mut(2)
                    .write(AccountMeta::readonly(withdraw_withheld_authority.key()));
            }
        }

        for (account_meta, signer) in acc_metas[3..].iter_mut().zip(signers.iter()) {
            account_meta.write(AccountMeta::readonly_signer(signer.key()));
        }

        for (account_meta, source_account) in acc_metas[(3 + signers.len())..]
            .iter_mut()
            .zip(source_accounts.iter())
        {
            account_meta.write(AccountMeta::writable(source_account.key()));
        }

        // Instruction data layout:
        // -  [0]: instruction TransferFeeExtension discriminator (1 byte, u8)
        // -  [1]: instruction WithdrawWithheldTokensFromAccounts discriminator (1 byte, u8)
        // -  [2]: num_token_accounts (1 byte, u8)
        let instruction_data = [26, 3, num_token_accounts];

        let instruction = Instruction {
            program_id: token_program,
            accounts: unsafe { slice::from_raw_parts(acc_metas.as_ptr() as _, num_accounts) },
            data: &instruction_data,
        };

        // Account info array
        const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; MAX_CPI_ACCOUNTS];

        unsafe {
            // SAFETY:
            // - `account_infos` is sized to 3 + num_token_accounts + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_infos.get_unchecked_mut(0).write(mint);
            // - Index 1 is always present
            acc_infos.get_unchecked_mut(1).write(destination);
            // - Index 2 is always present
            acc_infos
                .get_unchecked_mut(2)
                .write(withdraw_withheld_authority);
        }

        // Fill signer accounts
        for (account_info, signer) in acc_infos[3..].iter_mut().zip(signers.iter()) {
            account_info.write(signer);
        }

        // Fill source accounts
        for (account_info, source_account) in acc_infos[(3 + signers.len())..]
            .iter_mut()
            .zip(source_accounts.iter())
        {
            account_info.write(source_account);
        }

        invoke_with_bounds::<{ MAX_CPI_ACCOUNTS }>(&instruction, unsafe {
            slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts)
        })
    }
}
