use core::slice::from_raw_parts;

use solana_account_view::AccountView;
use solana_address::Address;
use solana_instruction_view::{
    cpi::{invoke_signed, Signer},
    InstructionAccount, InstructionView,
};
use solana_program_error::ProgramResult;

use crate::{
    instructions::extensions::ExtensionDiscriminator,
    write_bytes,
    UNINIT_BYTE,
};

const INITIALIZE_DATA_LEN: usize = 36;

/// Initialize a new mint with interest accrual.
///
/// ### Accounts:
///   0. `[WRITE]` The mint to initialize.
pub struct Initialize<'a, 'b> {
    /// Mint Account
    pub mint: &'a AccountView,
    /// Optional authority that can set the interest rate
    pub rate_authority: Option<&'a Address>,
    /// The initial interest rate
    pub rate: i16,
    /// Token Program
    pub token_program: &'b Address,
}

impl Initialize<'_, '_> {
    pub const DISCRIMINATOR: u8 = 0;

    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction accounts
        let instruction_accounts: [InstructionAccount; 1] =
            [InstructionAccount::writable(self.mint.address())];

        let instruction_data = initialize_instruction_data(self.rate_authority, self.rate);

        let instruction = InstructionView {
            program_id: self.token_program,
            accounts: &instruction_accounts,
            data: unsafe {
                from_raw_parts(instruction_data.as_ptr() as *const u8, INITIALIZE_DATA_LEN)
            },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}

#[inline(always)]
fn initialize_instruction_data(
    rate_authority: Option<&Address>,
    rate: i16,
) -> [core::mem::MaybeUninit<u8>; INITIALIZE_DATA_LEN] {
    // -  [0]     : extension discriminator (1 byte, u8)
    // -  [1]     : instruction discriminator (1 byte, u8)
    // -  [2..34] : rate_authority address (32 bytes, optional)
    // -  [34..36]: rate (2 bytes, i16)
    let mut instruction_data = [UNINIT_BYTE; INITIALIZE_DATA_LEN];
    write_bytes(
        &mut instruction_data,
        &[ExtensionDiscriminator::InterestBearingMint as u8],
    );
    write_bytes(&mut instruction_data[1..2], &[Initialize::DISCRIMINATOR]);
    if let Some(auth) = rate_authority {
        write_bytes(&mut instruction_data[2..34], auth.as_array());
    } else {
        write_bytes(&mut instruction_data[2..34], &[0u8; 32]);
    }
    write_bytes(&mut instruction_data[34..36], &rate.to_le_bytes());

    instruction_data
}
