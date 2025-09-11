use core::slice::from_raw_parts;

use pinocchio::pubkey::Pubkey;

use crate::{write_bytes, UNINIT_BYTE};

/// Discriminator for the InitializePermanentDelegate.
pub const INITIALIZE_PERMANENT_DELEGATE: u8 = 35;

pub fn permanent_delegate_instruction_data<'a>(delegate: Pubkey) -> &'a [u8] {
    // Instruction data Layout:
    // -  [0]: instruction discriminator (1 byte, u8)
    // -  [1..33]: permanent delegate (32 bytes, Pubkey)
    let mut instruction_data = [UNINIT_BYTE; 33];
    // Set discriminator as u8 at offset [0]
    write_bytes(
        &mut instruction_data[0..1],
        &[INITIALIZE_PERMANENT_DELEGATE],
    );
    // Set permanent delegate as Pubkey at offset [1..33]
    write_bytes(&mut instruction_data[1..33], &delegate);

    unsafe { from_raw_parts(instruction_data.as_ptr() as _, 33) }
}
