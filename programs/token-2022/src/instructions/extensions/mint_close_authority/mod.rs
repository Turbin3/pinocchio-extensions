use {
    crate::{
        instructions::{extensions::ExtensionDiscriminator, MAX_MULTISIG_SIGNERS},
        write_bytes, UNINIT_BYTE,
    },
    core::{mem::MaybeUninit, slice},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};
/// Update the multiplier for the Scaled UI Amount extension on a mint account.
///
/// Expected accounts:
///
/// **Single authority**
/// 0. `[writable]` The mint account to initialize the close authority.
///
/// **Multisignature authority**
/// 0. `[writable]` The mint account to initialize the close authority.
/// 1. `[signer]` M signer accounts (as required by the multisig).
pub struct InitializeMintCloseAuthority<'a, 'b, 'c> {
    /// The mint account to initialize the close authority
    pub mint_account: &'a AccountView,
    /// Signer accounts if the authority is a multisig.
    pub signers: &'c [&'a AccountView],
    /// The public key for the account that can close the mint
    pub close_authority: &'b Option<Address>,
    /// Token program (Token-2022).
    pub token_program: &'b Address,
}

impl InitializeMintCloseAuthority<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let &Self {
            mint_account,
            signers: multisig_accounts,
            close_authority,
            token_program,
            ..
        } = self;
        if multisig_accounts.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        const UNINIT_INSTRUCTION_ACCOUNTS: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut accounts = [UNINIT_INSTRUCTION_ACCOUNTS; 1 + MAX_MULTISIG_SIGNERS];

        // SAFETY:
        // - `instruction_accounts` is sized to 1 + MAX_MULTISIG_SIGNERS
        // Index 0 and 1 are always present
        unsafe {
            accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(mint_account.address()));
        }

        for (account, signer) in accounts[1..].iter_mut().zip(multisig_accounts.iter()) {
            account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        let mut data = [UNINIT_BYTE; 34];
        write_bytes(
            &mut data[0..1],
            &[ExtensionDiscriminator::MintCloseAuthority as u8],
        );

        if let Some(close_authority) = close_authority {
            write_bytes(&mut data[1..2], &[1]);
            write_bytes(&mut data[2..34], &close_authority.to_bytes());
        } else {
            write_bytes(&mut data[1..2], &[0]);
            write_bytes(&mut data[2..34], &Address::default().to_bytes());
        }
        let data = unsafe { &*(data.as_ptr() as *const [u8; 34]) };

        let num_accounts = 1 + multisig_accounts.len();

        let instruction = InstructionView {
            program_id: token_program,
            data,
            accounts: unsafe { slice::from_raw_parts(accounts.as_ptr() as _, num_accounts) },
        };

        // Account view array
        const UNINIT_ACCOUNT_VIEWS: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut account_views = [UNINIT_ACCOUNT_VIEWS; 1 + MAX_MULTISIG_SIGNERS];

        // SAFETY:
        // - `account_views` is sized to 1 + MAX_MULTISIG_SIGNERS
        // Index 0 and 1 are always present
        unsafe {
            account_views.get_unchecked_mut(0).write(mint_account);
        }

        // Fill signer accounts
        for (account_view, signer) in account_views[1..].iter_mut().zip(multisig_accounts.iter()) {
            account_view.write(signer);
        }

        invoke_signed_with_bounds::<{ 1 + MAX_MULTISIG_SIGNERS }>(
            &instruction,
            unsafe { slice::from_raw_parts(account_views.as_ptr() as _, num_accounts) },
            signers,
        )
    }
}
