/// Initialize instruction for the Transfer Hook extension
pub mod initialize;
/// Update instruction for the Transfer Hook extension
pub mod update;

pub use initialize::*;
pub use update::*;

/// Transfer Hook Extension discriminator
pub const TRANSFER_HOOK_EXTENSION_DISCRIMINATOR: u8 = 36;

/// Initialize sub-instruction discriminator
pub const INITIALIZE_DISCRIMINATOR: u8 = 0;

/// Update sub-instruction discriminator  
pub const UPDATE_DISCRIMINATOR: u8 = 1;

use crate::write_bytes;
use core::mem::MaybeUninit;
use pinocchio::pubkey::Pubkey;

/// Write an optional pubkey to a destination array. Compatible with the `OptionalNonZeroPubkey` type.
/// Returns the length of the written data.
/// 
/// `destination` - The destination array to write the pubkey to.
/// `pubkey` - The pubkey to write.
/// 
/// Returns the length of the written data.
/// 
/// Serialization format:
/// Some(pubkey): [1](presence flag) + [32-byte pubkey]= 33 bytes
/// None: [0](presence flag) = 1 byte

#[inline(always)]
pub(crate) fn write_optional_pubkey(
    destination: &mut [MaybeUninit<u8>],
    pubkey: Option<&Pubkey>,
) -> usize {
    match pubkey {
        Some(pk) => {
            //Write presence flag (1 = Some)
            destination[0].write(1);
            //Write pubkey bytes
            write_bytes(&mut destination[1..33], pk);
            1 + 32
        }
        None => {
            //Write presence flag (0 = None)
            destination[0].write(0);
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pinocchio::pubkey::Pubkey;
    use crate::UNINIT_BYTE;

    #[test]
    fn test_write_optional_pubkey_some() {
        let mut destination = [UNINIT_BYTE; 33];

        // Create a known pubkey for testing
        let test_pubkey = Pubkey::try_from([42u8; 32]).unwrap();
        let pubkey = Some(&test_pubkey);

        let len = write_optional_pubkey(&mut destination, pubkey);

        // Check length
        assert_eq!(len, 33);

        // Check presence flag
        assert_eq!(unsafe { destination[0].assume_init() }, 1);

        // Check pubkey bytes
        let mut pubkey_bytes = [0u8; 32];
        for (i, b) in pubkey_bytes.iter_mut().enumerate() {
            *b = unsafe { destination[1 + i].assume_init() };
        }
        assert_eq!(pubkey_bytes, test_pubkey.as_ref());
    }

    #[test]
    fn test_write_optional_pubkey_none() {
        let mut destination = [UNINIT_BYTE; 33];
        let pubkey = None;

        let len = write_optional_pubkey(&mut destination, pubkey);

        // Check length
        assert_eq!(len, 1);

        // Check presence flag
        assert_eq!(unsafe { destination[0].assume_init() }, 0);
    }

    #[test]
    fn test_transfer_hook_initialize_instruction_data() {
        // Test with both authority and program_id
        let authority = Pubkey::try_from([1u8; 32]).unwrap();
        let hook_program = Pubkey::try_from([2u8; 32]).unwrap();

        let mut instruction_data = [UNINIT_BYTE; 68];
        let mut offset = 0;

        // Main discriminator
        write_bytes(
            &mut instruction_data[offset..offset + 1],
            &[TRANSFER_HOOK_EXTENSION_DISCRIMINATOR],
        );
        offset += 1;

        // Sub discriminator
        write_bytes(
            &mut instruction_data[offset..offset + 1],
            &[INITIALIZE_DISCRIMINATOR],
        );
        offset += 1;

        // Authority (Some)
        let auth_len = write_optional_pubkey(&mut instruction_data[offset..], Some(&authority));
        offset += auth_len;

        // Program ID (Some)
        let program_len =
            write_optional_pubkey(&mut instruction_data[offset..], Some(&hook_program));
        offset += program_len;

        // Verify the instruction data
        let expected_len = 2 + 33 + 33; // discriminators + authority + program_id
        assert_eq!(offset, expected_len);

        // Check discriminators
        assert_eq!(
            unsafe { instruction_data[0].assume_init() },
            TRANSFER_HOOK_EXTENSION_DISCRIMINATOR
        );
        assert_eq!(
            unsafe { instruction_data[1].assume_init() },
            INITIALIZE_DISCRIMINATOR
        );

        // Check authority presence and data
        assert_eq!(unsafe { instruction_data[2].assume_init() }, 1); // presence flag
        let mut auth_bytes = [0u8; 32];
        for i in 0..32 {
            auth_bytes[i] = unsafe { instruction_data[3 + i].assume_init() };
        }
        assert_eq!(auth_bytes, authority.as_ref());

        // Check program_id presence and data
        assert_eq!(unsafe { instruction_data[35].assume_init() }, 1); // presence flag
        let mut program_bytes = [0u8; 32];
        for i in 0..32 {
            program_bytes[i] = unsafe { instruction_data[36 + i].assume_init() };
        }
        assert_eq!(program_bytes, hook_program.as_ref());
    }

    #[test]
    fn test_transfer_hook_initialize_minimal() {
        // Test with only authority (no program_id)
        let authority = Pubkey::try_from([1u8; 32]).unwrap();

        let mut instruction_data = [UNINIT_BYTE; 68];
        let mut offset = 0;

        // Main discriminator
        write_bytes(
            &mut instruction_data[offset..offset + 1],
            &[TRANSFER_HOOK_EXTENSION_DISCRIMINATOR],
        );
        offset += 1;

        // Sub discriminator
        write_bytes(
            &mut instruction_data[offset..offset + 1],
            &[INITIALIZE_DISCRIMINATOR],
        );
        offset += 1;

        // Authority (Some)
        let auth_len = write_optional_pubkey(&mut instruction_data[offset..], Some(&authority));
        offset += auth_len;

        // Program ID (None)
        let program_len = write_optional_pubkey(&mut instruction_data[offset..], None);
        offset += program_len;

        // Verify the instruction data
        let expected_len = 2 + 33 + 1; // discriminators + authority + none program_id
        assert_eq!(offset, expected_len);

        // Check program_id is None
        assert_eq!(unsafe { instruction_data[35].assume_init() }, 0); // presence flag = false
    }

    #[test]
    fn test_transfer_hook_update_instruction_data() {
        let new_program = Pubkey::try_from([3u8; 32]).unwrap();

        let mut instruction_data = [UNINIT_BYTE; 35];
        let mut offset = 0;

        // Main discriminator
        write_bytes(
            &mut instruction_data[offset..offset + 1],
            &[TRANSFER_HOOK_EXTENSION_DISCRIMINATOR],
        );
        offset += 1;

        // Sub discriminator
        write_bytes(
            &mut instruction_data[offset..offset + 1],
            &[UPDATE_DISCRIMINATOR],
        );
        offset += 1;

        // New program ID (Some)
        let program_len =
            write_optional_pubkey(&mut instruction_data[offset..], Some(&new_program));
        offset += program_len;

        // Verify the instruction data
        let expected_len = 2 + 33; // discriminators + program_id
        assert_eq!(offset, expected_len);

        // Check discriminators
        assert_eq!(
            unsafe { instruction_data[0].assume_init() },
            TRANSFER_HOOK_EXTENSION_DISCRIMINATOR
        );
        assert_eq!(
            unsafe { instruction_data[1].assume_init() },
            UPDATE_DISCRIMINATOR
        );

        // Check program_id presence and data
        assert_eq!(unsafe { instruction_data[2].assume_init() }, 1); // presence flag
        let mut program_bytes = [0u8; 32];
        for i in 0..32 {
            program_bytes[i] = unsafe { instruction_data[3 + i].assume_init() };
        }
        assert_eq!(program_bytes, new_program.as_ref());
    }

    #[test]
    fn test_transfer_hook_update_disable() {
        // Test disabling transfer hook (None program_id)
        let mut instruction_data = [UNINIT_BYTE; 35];
        let mut offset = 0;

        // Main discriminator
        write_bytes(
            &mut instruction_data[offset..offset + 1],
            &[TRANSFER_HOOK_EXTENSION_DISCRIMINATOR],
        );
        offset += 1;

        // Sub discriminator
        write_bytes(
            &mut instruction_data[offset..offset + 1],
            &[UPDATE_DISCRIMINATOR],
        );
        offset += 1;

        // New program ID (None - disabling)
        let program_len = write_optional_pubkey(&mut instruction_data[offset..], None);
        offset += program_len;

        // Verify the instruction data
        let expected_len = 2 + 1; // discriminators + none program_id
        assert_eq!(offset, expected_len);

        // Check program_id is None
        assert_eq!(unsafe { instruction_data[2].assume_init() }, 0); // presence flag = false
    } 
}
