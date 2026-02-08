use core::{mem::MaybeUninit, slice};

use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{
    cpi::{invoke_signed, invoke_with_bounds, Signer},
    InstructionAccount, InstructionView,
};
use solana_program_error::{ProgramError, ProgramResult};

use crate::{
    instructions::{
        extensions::ExtensionDiscriminator,
        MAX_MULTISIG_SIGNERS,
    },
    write_bytes,
    UNINIT_BYTE,
};

const UPDATE_RATE_DATA_LEN: usize = 4;

/// Update the interest rate. Only supported for mints that include the `InterestBearingConfig`
/// extension.
///
/// **Single authority**
///   0. `[WRITE]` The mint.
///   1. `[SIGNER]` The mint rate authority.
///
/// **Multisignature authority**
///   0. `[WRITE]` The mint.
///   1. `[]` The mint's multisignature rate authority.
///   2. `..2+M` `[SIGNER]` M signer accounts.
pub struct UpdateRate<'a, 'b, 'c> {
    /// Mint Account
    pub mint: &'a AccountView,
    /// The rate authority.
    pub authority: &'a AccountView,
    /// The new interest rate
    pub rate: i16,
    /// The signer accounts if `authority` is a multisig
    pub signers: &'b [&'a AccountView],
    /// Token Program
    pub token_program: &'c Address,
}

impl UpdateRate<'_, '_, '_> {
    pub const DISCRIMINATOR: u8 = 1;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let is_multisig = !self.signers.is_empty();

        if is_multisig {
            self.invoke_multisig()
        } else {
            self.invoke_single_owner(signers)
        }
    }

    #[inline(always)]
    fn invoke_single_owner(&self, signers: &[Signer]) -> ProgramResult {
        let instruction_accounts: [InstructionAccount; 2] = [
            InstructionAccount::writable(self.mint.address()),
            InstructionAccount::readonly_signer(self.authority.address()),
        ];

        let instruction_data = update_rate_instruction_data(self.rate);

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &instruction_accounts,
            data: unsafe {
                slice::from_raw_parts(instruction_data.as_ptr() as *const u8, UPDATE_RATE_DATA_LEN)
            },
        };

        invoke_signed(&instruction, &[self.mint, self.authority], signers)
    }

    #[inline(always)]
    fn invoke_multisig(&self) -> ProgramResult {
        let &Self {
            mint,
            authority,
            rate,
            signers: multisig_signers,
            token_program,
        } = self;

        if multisig_signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(ProgramError::InvalidArgument);
        }

        let num_accounts = 2 + multisig_signers.len();

        // Instruction accounts
        const UNINIT_INSTRUCTION_ACCOUNT: MaybeUninit<InstructionAccount> =
            MaybeUninit::<InstructionAccount>::uninit();
        let mut instruction_accounts = [UNINIT_INSTRUCTION_ACCOUNT; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `instruction_accounts` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 and 1 are always present
            instruction_accounts
                .get_unchecked_mut(0)
                .write(InstructionAccount::writable(mint.address()));
            instruction_accounts
                .get_unchecked_mut(1)
                .write(InstructionAccount::readonly(authority.address()));
        }

        for (instruction_account, signer) in instruction_accounts[2..]
            .iter_mut()
            .zip(multisig_signers.iter())
        {
            instruction_account.write(InstructionAccount::readonly_signer(signer.address()));
        }

        let instruction_data = update_rate_instruction_data(rate);

        let instruction = InstructionView {
            program_id: token_program,
            accounts: unsafe {
                slice::from_raw_parts(instruction_accounts.as_ptr() as _, num_accounts)
            },
            data: unsafe {
                slice::from_raw_parts(instruction_data.as_ptr() as *const u8, UPDATE_RATE_DATA_LEN)
            },
        };

        // Account view array
        const UNINIT_VIEW: MaybeUninit<&AccountView> = MaybeUninit::uninit();
        let mut acc_views = [UNINIT_VIEW; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY:
            // - `acc_views` is sized to 2 + MAX_MULTISIG_SIGNERS
            // - Index 0 and 1 are always present
            acc_views.get_unchecked_mut(0).write(mint);
            acc_views.get_unchecked_mut(1).write(authority);
        }

        // Fill signer accounts
        for (account_view, signer) in acc_views[2..].iter_mut().zip(multisig_signers.iter()) {
            account_view.write(signer);
        }

        invoke_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(&instruction, unsafe {
            slice::from_raw_parts(acc_views.as_ptr() as _, num_accounts)
        })
    }
}

#[inline(always)]
fn update_rate_instruction_data(
    rate: i16,
) -> [core::mem::MaybeUninit<u8>; UPDATE_RATE_DATA_LEN] {
    // Instruction data layout:
    // -  [0]    : extension discriminator (1 byte, u8)
    // -  [1]    : instruction discriminator (1 byte, u8)
    // -  [2..4] : rate (2 bytes, i16)
    let mut instruction_data = [UNINIT_BYTE; UPDATE_RATE_DATA_LEN];
    write_bytes(
        &mut instruction_data,
        &[ExtensionDiscriminator::InterestBearingMint as u8],
    );
    write_bytes(&mut instruction_data[1..2], &[UpdateRate::DISCRIMINATOR]);
    write_bytes(&mut instruction_data[2..4], &rate.to_le_bytes());

    instruction_data
}

