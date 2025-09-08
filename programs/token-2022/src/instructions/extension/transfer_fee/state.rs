use core::slice::from_raw_parts;

use pinocchio::pubkey::Pubkey;

use crate::{
    instructions::{
        INITIALIZE_TRANSFER_FEE_CONFIG, SET_TRANSFER_FEE, TRANSFER_CHECKED_WITH_FEE,
        TRANSFER_FEE_EXTENSION,
    },
    write_bytes, UNINIT_BYTE,
};

pub fn initialize_transfer_fee_config_instruction_data<'a>(
    transfer_fee_config_authority: Option<&'a Pubkey>,
    withdraw_withheld_authority: Option<&'a Pubkey>,
    transfer_fee_basis_points: u16,
    maximum_fee: u16,
) -> &'a [u8] {
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

    // -  [0]: instruction TransferFeeExtension discriminator (1 byte, u8)
    // -  [1]: instruction WithdrawWithheldTokensFromMint discriminator (1 byte, u8)
    write_bytes(
        &mut instruction_data,
        &[TRANSFER_FEE_EXTENSION, INITIALIZE_TRANSFER_FEE_CONFIG],
    );

    let mut offset = 2;

    // Set Option(transfer_fee_config_authority) = `false` [2..3]
    write_bytes(&mut instruction_data[offset..offset + 1], &[0]);
    offset += 1;
    if let Some(transfer_fee_config_authority) = transfer_fee_config_authority {
        // Set Option(transfer_fee_config_authority) = `true` [2..3]
        write_bytes(&mut instruction_data[offset - 1..offset], &[1]);
        // Set transfer_fee_config_authority at offset [3..35]
        write_bytes(
            &mut instruction_data[offset..offset + 32],
            transfer_fee_config_authority,
        );
        offset += 32;
    }

    // Set Option(withdraw_withheld_authority) = `false` [35 or 3]
    write_bytes(&mut instruction_data[offset..offset + 1], &[0]);
    offset += 1;
    if let Some(withdraw_withheld_authority) = withdraw_withheld_authority {
        // Set Option(withdraw_withheld_authority) = `true` [35 or 3]
        write_bytes(&mut instruction_data[offset - 1..offset], &[1]);
        // Set withdraw_withheld_authority at offset [36..68] or [4..36]
        write_bytes(
            &mut instruction_data[offset..offset + 32],
            withdraw_withheld_authority,
        );
        offset += 32;
    }

    // Set transfer_fee_basis_points as u16 at offset [68..70 or 36..38 or 4..6]
    write_bytes(
        &mut instruction_data[offset..offset + 2],
        transfer_fee_basis_points.to_le_bytes().as_ref(),
    );
    offset += 2;

    // Set maximum_fee as u16 at offset [70..72 or 38..40 or 6..8]
    write_bytes(
        &mut instruction_data[offset..offset + 2],
        maximum_fee.to_le_bytes().as_ref(),
    );
    offset += 2;

    unsafe { from_raw_parts(instruction_data.as_ptr() as _, offset) }
}

pub fn transfer_checked_with_fee_instruction_data<'a>(
    amount: u64,
    decimals: u8,
    fee: u64,
) -> &'a [u8] {
    // Instruction data layout:
    // -  [0]: instruction TransferFeeExtension discriminator (1 byte, u8)
    // -  [1]: instruction TransferCheckedWithFee discriminator (1 byte, u8)
    // -  [2..10]: amount (8 bytes, u64)
    // -  [10]: decimals (1 byte, u8)
    // -  [11..19]: fee (8 bytes, u64)
    let mut instruction_data = [UNINIT_BYTE; 19];

    // -  [0]: instruction TransferFeeExtension discriminator (1 byte, u8)
    // -  [1]: instruction TransferCheckedWithFee discriminator (1 byte, u8)
    write_bytes(
        &mut instruction_data,
        &[TRANSFER_FEE_EXTENSION, TRANSFER_CHECKED_WITH_FEE],
    );

    // Set amount as u64 at offset [2..10]
    write_bytes(&mut instruction_data[2..10], amount.to_le_bytes().as_ref());
    // Set amount as u8 at offset [10..11]
    write_bytes(
        &mut instruction_data[10..11],
        decimals.to_le_bytes().as_ref(),
    );
    // Set fee as u64 at offset [11..19]
    write_bytes(&mut instruction_data[11..19], fee.to_le_bytes().as_ref());

    unsafe { from_raw_parts(instruction_data.as_ptr() as _, 19) }
}

pub fn set_transfer_fee_instruction_data<'a>(
    transfer_fee_basis_points: u16,
    maximum_fee: u64,
) -> &'a [u8] {
    // Instruction data layout:
    // -  [0]: instruction TransferFeeExtension discriminator (1 byte, u8)
    // -  [1]: instruction SetTransferFee discriminator (1 byte, u8)
    // -  [2..4]: transfer_fee_basis_points (2 bytes, u16)
    // -  [4..12]: maximum_fee (8 bytes, u64)
    let mut instruction_data = [UNINIT_BYTE; 12];

    // -  [0]: instruction TransferFeeExtension discriminator (1 byte, u8)
    // -  [1]: instruction SetTransferFee discriminator (1 byte, u8)
    write_bytes(
        &mut instruction_data,
        &[TRANSFER_FEE_EXTENSION, SET_TRANSFER_FEE],
    );

    // Set amount as u64 at offset [2..4]
    write_bytes(
        &mut instruction_data[2..4],
        transfer_fee_basis_points.to_le_bytes().as_ref(),
    );
    // Set fee as u64 at offset [4..12]
    write_bytes(
        &mut instruction_data[4..12],
        maximum_fee.to_le_bytes().as_ref(),
    );

    unsafe { from_raw_parts(instruction_data.as_ptr() as _, 12) }
}
