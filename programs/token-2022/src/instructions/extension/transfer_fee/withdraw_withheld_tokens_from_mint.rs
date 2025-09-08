use core::{mem::MaybeUninit, slice};

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed_with_bounds,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::instructions::{
    MAX_MULTISIG_SIGNERS, TRANSFER_FEE_EXTENSION, WITHDRAW_WITHHELD_TOKENS_FROM_MINT,
};

/// Transfer all withheld tokens in the mint to an account. Signed by the
/// mint's withdraw withheld tokens authority.
///
/// Accounts expected by this instruction:
///
///   * Single owner/delegate
///   0. `[writable]` The token mint. Must include the `TransferFeeConfig`
///      extension.
///   1. `[writable]` The fee receiver account. Must include the
///      `TransferFeeAmount` extension associated with the provided mint.
///   2. `[signer]` The mint's `withdraw_withheld_authority`.
///
///   * Multisignature owner/delegate
///   0. `[writable]` The token mint.
///   1. `[writable]` The destination account.
///   2. `[]` The mint's multisig `withdraw_withheld_authority`.
///   3. `..3+M` `[signer]` M signer accounts.
pub struct WithdrawWithheldTokensFromMint<'a, 'b, 'c>
where
    'a: 'b,
{
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// Destination Account
    pub destination: &'a AccountInfo,
    /// Withdraw withheld authority
    pub withdraw_withheld_authority: &'a AccountInfo,
    /// Signer Accounts
    pub signers: &'b [&'a AccountInfo],
    /// Token Program
    pub token_program: &'c Pubkey,
}

impl WithdrawWithheldTokensFromMint<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint,
            destination,
            withdraw_withheld_authority,
            signers: account_signers,
            token_program,
        } = self;

        if account_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 3 + account_signers.len();

        // Account metadata
        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut acc_metas = [UNINIT_META; 3 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_metas` is sized to 3 + MAX_MULTISIG_SIGNERS
            // - Index 0 is always present
            acc_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(mint.key()));
            // - Index 1 is always present
            acc_metas
                .get_unchecked_mut(1)
                .write(AccountMeta::writable(destination.key()));
            // - Index 2 is always present
            if account_signers.is_empty() {
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

        for (account_meta, signer) in acc_metas[3..].iter_mut().zip(account_signers.iter()) {
            account_meta.write(AccountMeta::readonly_signer(signer.key()));
        }

        // Instruction data layout:
        // -  [0]: instruction TransferFeeExtension discriminator (1 byte, u8)
        // -  [1]: instruction WithdrawWithheldTokensFromMint discriminator (1 byte, u8)
        let instruction_data = [TRANSFER_FEE_EXTENSION, WITHDRAW_WITHHELD_TOKENS_FROM_MINT];

        let instruction = Instruction {
            program_id: token_program,
            accounts: unsafe { slice::from_raw_parts(acc_metas.as_ptr() as _, num_accounts) },
            data: &instruction_data,
        };

        // Account info array
        const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; 3 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `account_infos` is sized to 3 + MAX_MULTISIG_SIGNERS
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
        for (account_info, signer) in acc_infos[3..].iter_mut().zip(account_signers.iter()) {
            account_info.write(signer);
        }

        invoke_signed_with_bounds::<{ 3 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
