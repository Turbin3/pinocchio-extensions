use core::{
    mem::MaybeUninit,
    slice::{self, from_raw_parts},
};

use pinocchio::{
    account_info::AccountInfo,
    cpi::{invoke_signed, invoke_with_bounds},
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{
    extensions::pausable::{PAUSABLE_EXTENSION, PAUSE},
    instructions::MAX_MULTISIG_SIGNERS,
    write_bytes, UNINIT_BYTE,
};

/// Pause minting, burning, and transferring for the mint.
///
/// ### Accounts:
///   0. `[WRITE]` The mint to update.
///   1. `[SIGNER]` The mint's pause authority.
pub struct Pause<'a, 'b, 'c> {
    /// Mint account
    pub mint: &'a AccountInfo,
    /// Authority
    pub authority: &'a AccountInfo,
    /// Signer Accounts (for multisig support)
    pub signers: &'b [&'a AccountInfo],
    /// Token program
    pub token_program: &'c Pubkey,
}

impl Pause<'_, '_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        if self.signers.len() > MAX_MULTISIG_SIGNERS {
            return Err(pinocchio::program_error::ProgramError::InvalidArgument);
        }

        if self.signers.is_empty() {
            self.invoke_single_owner(signers)
        } else {
            self.invoke_multisig()
        }
    }

    #[inline(always)]
    fn invoke_single_owner(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data layout:
        // - [0]: token_instruction_type: (1 byte, u8)
        // - [1]: instruction_type: (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 2];

        // Set discriminator as u8 at offset [0] for PausableExtension token instruction
        write_bytes(&mut instruction_data, &[PAUSABLE_EXTENSION]);
        // Set discriminator as u8 at offset [1] for Pause instruction
        write_bytes(&mut instruction_data[1..2], &[PAUSE]);

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        invoke_signed(&instruction, &[self.mint, self.authority], signers)
    }

    #[inline(always)]
    fn invoke_multisig(&self) -> ProgramResult {
        let num_accounts = 2 + self.signers.len();

        const UNINIT_META: MaybeUninit<AccountMeta> = MaybeUninit::<AccountMeta>::uninit();
        let mut account_metas = [UNINIT_META; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY
            account_metas
                .get_unchecked_mut(0)
                .write(AccountMeta::writable(self.mint.key()));
            account_metas
                .get_unchecked_mut(1)
                .write(AccountMeta::readonly(self.authority.key()));
        }

        for (account_meta, signer) in account_metas[2..].iter_mut().zip(self.signers.iter()) {
            account_meta.write(AccountMeta::readonly_signer(signer.key()));
        }

        // Instruction data layout:
        // - [0]: token_instruction_type: (1 byte, u8)
        // - [1]: instruction_type: (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 2];

        // Set discriminator as u8 at offset [0] for PausableExtension token instruction
        write_bytes(&mut instruction_data, &[PAUSABLE_EXTENSION]);
        // Set discriminator as u8 at offset [1] for Pause instruction
        write_bytes(&mut instruction_data[1..2], &[PAUSE]);

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: unsafe { from_raw_parts(account_metas.as_ptr() as _, num_accounts) },
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, instruction_data.len()) },
        };

        const UNINIT_INFO: MaybeUninit<&AccountInfo> = MaybeUninit::uninit();
        let mut acc_infos = [UNINIT_INFO; 2 + MAX_MULTISIG_SIGNERS];

        unsafe {
            // SAFETY
            acc_infos.get_unchecked_mut(0).write(self.mint);
            acc_infos.get_unchecked_mut(1).write(self.authority);
        }

        for (account_info, signer) in acc_infos[2..].iter_mut().zip(self.signers.iter()) {
            account_info.write(signer);
        }

        invoke_with_bounds::<{ 2 + MAX_MULTISIG_SIGNERS }>(&instruction, unsafe {
            slice::from_raw_parts(acc_infos.as_ptr() as _, num_accounts)
        })
    }
}
