use {
    crate::instructions::extensions::ExtensionDiscriminator,
    core::{mem::MaybeUninit, slice},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::ProgramResult,
};

/// Initialize the Non-Transferable extension on a mint.
///
/// This instruction must be called after creating the mint but before initializing it.
///
/// Expected accounts:
///
/// 0. `[writable]` The mint account to initialize as non-transferable.
pub struct Initialize<'a, 'b> {
    /// The mint account to initialize.
    ///
    /// Note: This extension applies to the **Mint Account**, effectively making the
    /// token "Soulbound" for all holders. It cannot be applied to individual
    /// Token Accounts.
    pub mint: &'a AccountView,

    /// Token program.
    pub token_program: &'b Address,
}

impl Initialize<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint,
            token_program,
        } = self;

        // Account Metadata
        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 1];

        unsafe {
            // SAFETY:
            // - `instruction_accounts` is sized to 1
            // - Index 0 is always present (Mint)
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(mint.address()));
        }

        // Instruction Data
        // Unlike MemoTransfer or CpiGuard which toggle state on a Token Account,
        // NonTransferable permanently initializes a Mint.
        // Therefore, it does not accept a toggle byte (Enable/Disable).
        let data = &[ExtensionDiscriminator::InitializeNonTransferableMint as u8];

        let instruction = InstructionView {
            program_id: token_program,
            data,
            accounts: unsafe { slice::from_raw_parts(instruction_accounts.as_ptr() as _, 1) },
        };

        // Account Views for CPI
        const UNINIT_ACCOUNT_VIEWS: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut account_views = [UNINIT_ACCOUNT_VIEWS; 1];

        unsafe {
            // SAFETY:
            // - `account_views` is sized to 1
            account_views.get_unchecked_mut(0).write(mint);
        }

        invoke_signed_with_bounds::<1>(
            &instruction,
            unsafe { slice::from_raw_parts(account_views.as_ptr() as *const &AccountView, 1) },
            signers,
        )
    }
}
