use alloc::vec;
use core::slice::from_raw_parts;
use pinocchio::account_info::AccountInfo;
use pinocchio::cpi::{invoke, slice_invoke};
use pinocchio::instruction::{AccountMeta, Instruction};
use pinocchio::ProgramResult;
use pinocchio::pubkey::Pubkey;
use pinocchio::sysvars::clock::UnixTimestamp;
use crate::{write_bytes, UNINIT_BYTE};

#[repr(C)]
pub struct InterestBearingConfig{
    pub rate_authority: Option<Pubkey>,

    pub initialization_timestamp: UnixTimestamp,

    pub pre_update_average_rate: i16,

    pub last_update_timestamp: UnixTimestamp,

    pub current_rate: i16

}

#[repr(u8)]
pub enum InterestBearingMintInstruction {
    Initialize = 0,
    UpdateRate = 1,
}

/// Instruction data for Initialize
#[repr(C)]
pub struct InitializeInstructionData {
    pub rate_authority: [u8; 32], // Pubkey
    pub rate: i16,
}


/// Initialize the interest bearing mint
pub struct InitializeInterestBearingMint<'a, 'b> {
    pub mint: &'a AccountInfo,
    pub rate_authority: &'a Pubkey,
    pub initial_rate: i16,
    pub token_program: &'b Pubkey,
}

impl InitializeInterestBearingMint<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // - [0]              : discriminator (1 byte)
        // - [1]              : rate_authority presence flag (1 byte)
        // - [2..34]          : rate_authority pubkey (optional, 32 bytes)
        // - [34..36 or 2..4] : initial_rate (2 bytes)
        //
        // Size depends on presence of rate_authority
        let mut instruction_data = [UNINIT_BYTE; 36];
        let length: usize;

        // Write discriminator at byte 0
        write_bytes(&mut instruction_data[0..1], &[InterestBearingMintInstruction::Initialize as u8]);

        if let Some(rate_auth) = Some(self.rate_authority) {
            // Rate authority present: write flag 1 and pubkey bytes
            write_bytes(&mut instruction_data[1..2], &[1]);
            write_bytes(&mut instruction_data[2..34], rate_auth.as_ref());
            // Write initial_rate after pubkey at [34..36]
            write_bytes(&mut instruction_data[34..36], self.initial_rate.to_le_bytes().as_ref());
            length = 36;
        } else {
            // Rate authority absent: write flag 0, no pubkey bytes
            write_bytes(&mut instruction_data[1..2], &[0]);
            // Write initial_rate immediately after flag at [2..4]
            write_bytes(&mut instruction_data[2..4], self.initial_rate.to_le_bytes().as_ref());
            length = 4;
        }

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, length) },
        };

        invoke(&instruction, &[self.mint])
    }
}


/// Update rate of interest bearing mint
pub struct UpdateInterestBearingMintRate<'a, 'b> {
    pub mint: &'a AccountInfo,
    pub rate_authority: &'a AccountInfo,
    pub signers: &'a [&'a AccountInfo],
    pub new_rate: i16,
    pub token_program: &'b Pubkey,
}

impl UpdateInterestBearingMintRate<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Accounts required:
        // 0. `[writable]` Mint
        // 1. `[readonly]` Rate authority (single or multisig)
        // 2..n. `[readonly, signer]` Additional multisig signers as needed

        let mut accounts = vec![
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly(self.rate_authority.key()),
        ];

        for signer in self.signers {
            accounts.push(AccountMeta::readonly(signer.key()));
        }

        // Instruction data layout:
        // [0]: discriminator (1 byte)
        // [1..3]: new_rate (i16 little endian)
        let mut ix_data = [UNINIT_BYTE; 3];

        // Set discriminator
        write_bytes(&mut ix_data[0..1], &[InterestBearingMintInstruction::UpdateRate as u8]);

        // Set new_rate (2 bytes)
        write_bytes(&mut ix_data[1..3], self.new_rate.to_le_bytes().as_ref());

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &accounts,
            data: unsafe { from_raw_parts(ix_data.as_ptr() as _, ix_data.len()) },
        };

        let mut cpi_accounts = vec![self.mint, self.rate_authority];
        cpi_accounts.extend_from_slice(self.signers);


        // Can't use this as signers is DST
        // invoke(&instruction, &cpi_accounts[..])

        slice_invoke(&instruction, &cpi_accounts)

    }
}