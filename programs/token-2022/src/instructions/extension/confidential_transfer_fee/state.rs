use core::slice::from_raw_parts;

use pinocchio::pubkey::Pubkey;

use crate::{
    instructions::{
        CONFIDENTIAL_TRANSFER_FEE_EXTENSION, ELGAMAL_PUBKEY_LEN,
        INITIALIZE_CONFIDENTIAL_TRANSFER_FEE_CONFIG,
    },
    write_bytes, UNINIT_BYTE,
};

pub fn initialize_confidential_transfer_fee_config_instruction_data<'a>(
    authority: Option<Pubkey>,
    withdraw_withheld_authority_elgamal_pubkey: [u8; ELGAMAL_PUBKEY_LEN],
) -> &'a [u8] {
    // Instruction data Layout:
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1]: extension instruction discriminator (1 byte, u8)
    // -  [2..38]: authority (32 bytes, Pubkey)
    // -  [38..50]: withdraw withheld authority ElGamal public key (32 bytes, ElGamalPubkey)
    let mut instruction_data = [UNINIT_BYTE; 70];

    // Set the instruction discriminator
    write_bytes(
        &mut instruction_data[0..2],
        &[
            CONFIDENTIAL_TRANSFER_FEE_EXTENSION,
            INITIALIZE_CONFIDENTIAL_TRANSFER_FEE_CONFIG,
        ],
    );

    // Set the authority
    if let Some(authority) = authority {
        write_bytes(&mut instruction_data[2..38], &authority);
    } else {
        write_bytes(&mut instruction_data[2..38], &Pubkey::default());
    }

    // Set the withdraw withheld authority ElGamal public key
    write_bytes(
        &mut instruction_data[38..70],
        &withdraw_withheld_authority_elgamal_pubkey,
    );

    unsafe { from_raw_parts(instruction_data.as_ptr() as _, 70) }
}
