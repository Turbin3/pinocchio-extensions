use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE};

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
        // - [0]                        : TransferFeeExtension discriminator (1 byte)
        // - [1]                        : InitializeTransferFeeConfig discriminator (1 byte)
        // - [2]                        : transfer_fee_config_authority presence flag (1 byte, u8)
        // - [3..35]                    : transfer_fee_config_authority pubkey (optional, 32 bytes)
        // - [35 or 3]                  : withdraw_withheld_authority presence flag (1 byte, u8)
        // - [36..68 or 4..36]          : withdraw_withheld_authority pubkey (optional, 32 bytes)
        // - [68..70 or 36..38 or 4..6] : transfer_fee_basis_points (2 bytes)
        // - [70..72 or 38..40 or 6..8] : maximum_fee (2 bytes)
        //
        // Size depends on presence of transfer_fee_config_authority and withdraw_withheld_authority
        let mut instruction_data = [UNINIT_BYTE; 72];
        let length: usize;

        // -  [0]: instruction TransferFeeExtension discriminator (1 byte, u8)
        // -  [1]: instruction WithdrawWithheldTokensFromMint discriminator (1 byte, u8)
        write_bytes(&mut instruction_data, &[26, 0]);

        if let Some(transfer_fee_config_authority) = self.transfer_fee_config_authority {
            // Set Option = `true` & transfer_fee_config_authority at offset [2..35]
            write_bytes(&mut instruction_data[2..3], &[1]);
            write_bytes(&mut instruction_data[3..35], transfer_fee_config_authority);
            if let Some(withdraw_withheld_authority) = self.withdraw_withheld_authority {
                // Set Option = `true` & withdraw_withheld_authority at offset [35..68]
                write_bytes(&mut instruction_data[35..36], &[1]);
                write_bytes(&mut instruction_data[36..68], withdraw_withheld_authority);
                // Set transfer_fee_basis_points as u16 at offset [68..70]
                write_bytes(
                    &mut instruction_data[68..70],
                    self.transfer_fee_basis_points.to_le_bytes().as_ref(),
                );
                // Set maximum_fee as u16 at offset [70..72]
                write_bytes(
                    &mut instruction_data[70..72],
                    self.maximum_fee.to_le_bytes().as_ref(),
                );

                length = 72;
            } else {
                // Set Option = `false` withdraw_withheld_authority presence flag
                write_bytes(&mut instruction_data[35..36], &[0]);
                // Set transfer_fee_basis_points as u16 at offset [36..38]
                write_bytes(
                    &mut instruction_data[36..38],
                    self.transfer_fee_basis_points.to_le_bytes().as_ref(),
                );
                // Set maximum_fee as u16 at offset [38..40]
                write_bytes(
                    &mut instruction_data[38..40],
                    self.maximum_fee.to_le_bytes().as_ref(),
                );
                length = 40;
            }
        } else if let Some(withdraw_withheld_authority) = self.withdraw_withheld_authority {
            // Set Option = `false` transfer_fee_config_authority presence flag
            write_bytes(&mut instruction_data[2..3], &[0]);
            // Set Option = `true` & withdraw_withheld_authority at offset [3..36]
            write_bytes(&mut instruction_data[3..4], &[1]);
            write_bytes(&mut instruction_data[4..36], withdraw_withheld_authority);
            // Set transfer_fee_basis_points as u16 at offset [36..38]
            write_bytes(
                &mut instruction_data[36..38],
                self.transfer_fee_basis_points.to_le_bytes().as_ref(),
            );
            // Set maximum_fee as u16 at offset [38..40]
            write_bytes(
                &mut instruction_data[38..40],
                self.maximum_fee.to_le_bytes().as_ref(),
            );

            length = 40;
        } else {
            // Set Option = `false` transfer_fee_config_authority presence flag
            write_bytes(&mut instruction_data[2..3], &[0]);
            // Set Option = `false` withdraw_withheld_authority presence flag
            write_bytes(&mut instruction_data[3..4], &[0]);
            // Set transfer_fee_basis_points as u16 at offset [4..6]
            write_bytes(
                &mut instruction_data[4..6],
                self.transfer_fee_basis_points.to_le_bytes().as_ref(),
            );
            // Set maximum_fee as u16 at offset [6..8]
            write_bytes(
                &mut instruction_data[6..8],
                self.maximum_fee.to_le_bytes().as_ref(),
            );

            length = 8;
        }

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, length) },
        };

        invoke(&instruction, &[self.mint])
    }
}
