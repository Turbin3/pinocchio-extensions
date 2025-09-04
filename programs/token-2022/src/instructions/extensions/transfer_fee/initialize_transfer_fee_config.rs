use crate::{write_bytes, UNINIT_BYTE};
use core::slice::from_raw_parts;
use pinocchio::account_info::AccountInfo;
use pinocchio::cpi::invoke;
use pinocchio::instruction::{AccountMeta, Instruction};
use pinocchio::pubkey::Pubkey;
use pinocchio::ProgramResult;

/// Initialize the transfer fee on a new mint.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The mint to initialize.
pub struct InitializeTransferFeeConfig<'a, 'b> {
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// Optional transfer fee config authority
    pub transfer_fee_config_authority: Option<&'a Pubkey>,
    /// Optional withdraw withheld authority
    pub withdraw_withheld_authority: Option<&'a Pubkey>,
    /// Transfer fee basis points
    pub transfer_fee_basis_points: u16,
    /// Maximum fee
    pub maximum_fee: u16,
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl InitializeTransferFeeConfig<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // - [0]                        : discriminator (1 byte)
        // - [1]                        : transfer_fee_config_authority presence flag (1 byte, u8)
        // - [2..34]                    : transfer_fee_config_authority pubkey (optional, 32 bytes)
        // - [34 or 2]                  : withdraw_withheld_authority presence flag (1 byte, u8)
        // - [35..67 or 3..35]          : withdraw_withheld_authority pubkey (optional, 32 bytes)
        // - [67..69 or 35..37 or 3..5] : transfer_fee_basis_points (2 bytes)
        // - [69..71 or 37..39 or 5..7] : maximum_fee (2 bytes)
        //
        // Size depends on presence of transfer_fee_config_authority and withdraw_withheld_authority
        let mut instruction_data = [UNINIT_BYTE; 71];
        let length: usize;

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[0]);

        if let Some(transfer_fee_config_authority) = self.transfer_fee_config_authority {
            // Set Option = `true` & transfer_fee_config_authority at offset [1..34]
            write_bytes(&mut instruction_data[1..2], &[1]);
            write_bytes(&mut instruction_data[2..34], transfer_fee_config_authority);
            if let Some(withdraw_withheld_authority) = self.withdraw_withheld_authority {
                // Set Option = `true` & withdraw_withheld_authority at offset [34..67]
                write_bytes(&mut instruction_data[34..35], &[1]);
                write_bytes(&mut instruction_data[35..67], withdraw_withheld_authority);
                // Set transfer_fee_basis_points as u16 at offset [67..69]
                write_bytes(
                    &mut instruction_data[67..69],
                    self.transfer_fee_basis_points.to_le_bytes().as_ref(),
                );
                // Set maximum_fee as u16 at offset [69..71]
                write_bytes(
                    &mut instruction_data[69..71],
                    self.maximum_fee.to_le_bytes().as_ref(),
                );

                length = 71;
            } else {
                // Set Option = `false` withdraw_withheld_authority presence flag
                write_bytes(&mut instruction_data[34..35], &[0]);
                // Set transfer_fee_basis_points as u16 at offset [35..37]
                write_bytes(
                    &mut instruction_data[35..37],
                    self.transfer_fee_basis_points.to_le_bytes().as_ref(),
                );
                // Set maximum_fee as u16 at offset [37..39]
                write_bytes(
                    &mut instruction_data[37..39],
                    self.maximum_fee.to_le_bytes().as_ref(),
                );
                length = 39;
            }
        } else if let Some(withdraw_withheld_authority) = self.withdraw_withheld_authority {
            // Set Option = `false` transfer_fee_config_authority presence flag
            write_bytes(&mut instruction_data[1..2], &[0]);
            // Set Option = `true` & withdraw_withheld_authority at offset [2..35]
            write_bytes(&mut instruction_data[2..3], &[1]);
            write_bytes(&mut instruction_data[3..35], withdraw_withheld_authority);
            // Set transfer_fee_basis_points as u16 at offset [35..37]
            write_bytes(
                &mut instruction_data[35..37],
                self.transfer_fee_basis_points.to_le_bytes().as_ref(),
            );
            // Set maximum_fee as u16 at offset [37..39]
            write_bytes(
                &mut instruction_data[37..39],
                self.maximum_fee.to_le_bytes().as_ref(),
            );

            length = 39;
        } else {
            // Set Option = `false` transfer_fee_config_authority presence flag
            write_bytes(&mut instruction_data[1..2], &[0]);
            // Set Option = `false` withdraw_withheld_authority presence flag
            write_bytes(&mut instruction_data[2..3], &[0]);
            // Set transfer_fee_basis_points as u16 at offset [3..5]
            write_bytes(
                &mut instruction_data[3..5],
                self.transfer_fee_basis_points.to_le_bytes().as_ref(),
            );
            // Set maximum_fee as u16 at offset [5..7]
            write_bytes(
                &mut instruction_data[5..7],
                self.maximum_fee.to_le_bytes().as_ref(),
            );

            length = 7;
        }

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, length) },
        };

        invoke(&instruction, &[self.mint])
    }
}
