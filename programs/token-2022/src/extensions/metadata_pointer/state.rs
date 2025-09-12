use crate::write_bytes;
use core::mem::MaybeUninit;
use pinocchio::pubkey::Pubkey;

/// Sub-instruction for the Token-2022 Metadata Pointer extension.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MetadataPointerInstruction {
    /// Require metadata pointers for transfers into this Account.
    Initialize = 0,
    /// Stop requiring metadata pointers for transfers into this Account.
    Update = 1,
}

/// Metadata Pointer Extension discriminator (matches Token-2022)
pub const METADATA_POINTER_EXTENSION_DISCRIMINATOR: u8 = 39;

fn write_optional_pubkey(
    destination: &mut [MaybeUninit<u8>],
    pubkey: Option<&Pubkey>,
) -> usize {
    match pubkey {
        Some(pk) => {
            write_bytes(&mut destination[0..32], pk);
        }
        None => {
            write_bytes(&mut destination[0..32], &[0u8; 32]);
        }
    }
    32
}


/// Encode data for `Initialize`.
/// Layout:
/// - [0]: extension discriminator (u8 = 39)
/// - [1]: sub-instruction (u8 = 0)
/// - [2..34]: authority (OptionalNonZeroPubkey -> 32 bytes; zeros if None)
/// - [34..66]: metadata_address (OptionalNonZeroPubkey -> 32 bytes; zeros if None)
#[inline(always)]
pub fn encode_initialize_instruction_data(
    out: &mut [MaybeUninit<u8>],
    authority: Option<&Pubkey>,
    metadata_address: Option<&Pubkey>,
) -> usize {
    debug_assert!(out.len() >= 66);

    write_bytes(&mut out[0..1], &[METADATA_POINTER_EXTENSION_DISCRIMINATOR]);
    write_bytes(&mut out[1..2], &[MetadataPointerInstruction::Initialize as u8]);
    write_optional_pubkey(&mut out[2..34], authority);
    write_optional_pubkey(&mut out[34..66], metadata_address);

    66
}

/// Encode data for `Update`.
/// Layout:
/// - [0]: extension discriminator (u8 = 39)
/// - [1]: sub-instruction (u8 = 1)
/// - [2..34]: new metadata_address (OptionalNonZeroPubkey -> 32 bytes; zeros if None)
#[inline(always)]
pub fn encode_update_instruction_data(
    out: &mut [MaybeUninit<u8>],
    new_metadata_address: Option<&Pubkey>,
) -> usize {
    debug_assert!(out.len() >= 34);

    write_bytes(&mut out[0..1], &[METADATA_POINTER_EXTENSION_DISCRIMINATOR]);
    write_bytes(&mut out[1..2], &[MetadataPointerInstruction::Update as u8]);
    write_optional_pubkey(&mut out[2..34], new_metadata_address);

    34
}
