use core::{mem::MaybeUninit, slice};

use pinocchio::{
    account_info::AccountInfo,
    cpi::{invoke_signed_with_bounds, MAX_CPI_ACCOUNTS},
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::instructions::{TRANSFER_FEE_EXTENSION, WITHDRAW_WITHHELD_TOKENS_FROM_MINT};

/// Permissionless instruction to transfer all withheld tokens to the mint.
///
/// Succeeds for frozen accounts.
///
/// Accounts provided should include the `TransferFeeAmount` extension. If
/// not, the account is skipped.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint.
///   1. `..1+N` `[writable]` The source accounts to harvest from.
pub struct HarvestWithheldTokensToMint<'a, 'b, 'c>
where
    'a: 'b,
{
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// Source Accounts
    pub source_accounts: &'b [&'a AccountInfo],
    /// Token Program
    pub token_program: &'c Pubkey,
}

impl HarvestWithheldTokensToMint<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint,
            source_accounts,
            token_program,
        } = self;

        if (1 + source_accounts.len()) > MAX_CPI_ACCOUNTS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 1 + source_accounts.len();

        // Account metadata
        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut acc_metas = [UNINIT_META; 1 + MAX_CPI_ACCOUNTS];

        unsafe {
            // SAFETY:
            // - `account_metas` is sized to 1 + MAX_CPI_ACCOUNTS
            // - Index 0 is always present
            acc_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(mint.key()));
        }

        for (account_meta, source_account) in acc_metas[1..].iter_mut().zip(source_accounts.iter())
        {
            account_meta.write(AccountMeta::writable(source_account.key()));
        }

        // Instruction data layout:
        // -  [0]: instruction TransferFeeExtension discriminator (1 byte, u8)
        // -  [1]: instruction HarvestWithheldTokensToMint discriminator (1 byte, u8)
        let instruction_data = [TRANSFER_FEE_EXTENSION, WITHDRAW_WITHHELD_TOKENS_FROM_MINT];

        let instruction = Instruction {
            program_id: token_program,
            accounts: unsafe { slice::from_raw_parts(acc_metas.as_ptr() as _, num_accounts) },
            data: &instruction_data,
        };

        // Account info array
        const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; 1 + MAX_CPI_ACCOUNTS];

        unsafe {
            // SAFETY:
            // - `account_infos` is sized to 1 + MAX_CPI_ACCOUNTS
            // - Index 0 is always present
            acc_infos.get_unchecked_mut(0).write(mint);
        }

        // Fill source accounts
        for (account_info, source_account) in acc_infos[1..].iter_mut().zip(source_accounts.iter())
        {
            account_info.write(source_account);
        }

        invoke_signed_with_bounds::<{ 1 + MAX_CPI_ACCOUNTS }>(
            &instruction,
            unsafe { slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
