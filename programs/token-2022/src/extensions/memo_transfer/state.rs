use crate::{write_bytes, UNINIT_BYTE};
use core::mem::MaybeUninit;

/// Sub-instruction for the Token-2022 Memo Transfer extension.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RequiredMemoTransfersInstruction {
    /// Require memos for transfers into this Account.
    Enable = 0,
    /// Stop requiring memos for transfers into this Account.
    Disable = 1,
}

/// Discriminator for `TokenInstruction::MemoTransferExtension`.
/// Matches the Token-2022 core program instruction discriminator.
/// See: https://solana-labs.github.io/solana-program-library/token/js/enums/TokenInstruction.html
const MEMO_TRANSFER_EXTENSION: u8 = 30;

/// Packs instruction data for a `RequiredMemoTransfersInstruction`.
///
/// Layout (little-endian bytes):
/// - [0]: extension discriminator (1 byte, u8)
/// - [1]: instruction kind        (1 byte, u8)
#[inline(always)]
pub fn encode_instruction_data(
    instruction_type: RequiredMemoTransfersInstruction,
) -> [MaybeUninit<u8>; 2] {
    let mut data = [UNINIT_BYTE; 2];

    // Set extension discriminator at offset [0]
    write_bytes(&mut data, &[MEMO_TRANSFER_EXTENSION]);

    // Set sub-instruction at offset [1]
    write_bytes(&mut data[1..2], &[instruction_type as u8]);

    data
}
