use {
    crate::instructions::extensions::ExtensionDiscriminator,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{cpi, InstructionAccount, InstructionView},
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
        let &Self {
            mint,
            token_program,
        } = self;

        // Account Metadata
        let accounts = [InstructionAccount::writable(mint.address())];

        // Instruction Data
        // Unlike MemoTransfer or CpiGuard which toggle state on a Token Account,
        // NonTransferable permanently initializes a Mint.
        // Therefore, it does not accept a toggle byte 0/1 (Enable/Disable)
        // and does not need an authority since it's permanent
        let data = &[ExtensionDiscriminator::InitializeNonTransferableMint as u8];

        let instruction = InstructionView {
            program_id: token_program,
            data,
            accounts: &accounts,
        };

        cpi::invoke(&instruction, &[mint])
    }
}
