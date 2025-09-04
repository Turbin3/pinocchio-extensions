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

/// Writes OptionalNonZeroPubkey in the correct spl-pod format
/// - Some(pubkey): writes the actual 32-byte pubkey
/// - None: writes 32 bytes of zeros (Pubkey::default())
/// Always returns 32 (fixed length)
#[inline(always)]
pub(crate) fn write_optional_pubkey(
    destination: &mut [MaybeUninit<u8>],
    pubkey: Option<&Pubkey>,
) -> usize {
    match pubkey {
        Some(pk) => {
            // Write the actual pubkey bytes (32 bytes)
            write_bytes(&mut destination[0..32], pk);
        }
        None => {
            // Write 32 bytes of zeros to represent None
            write_bytes(&mut destination[0..32], &[0u8; 32]);
        }
    }
    32 // Always 32 bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UNINIT_BYTE;
    use pinocchio::pubkey::Pubkey;

    #[test]
    fn test_write_optional_pubkey_some() {
        let mut destination = [UNINIT_BYTE; 32];

        // Create a known pubkey for testing
        let test_pubkey = Pubkey::try_from([42u8; 32]).unwrap();
        let pubkey = Some(&test_pubkey);

        let len = write_optional_pubkey(&mut destination, pubkey);

        // Check length (always 32 bytes)
        assert_eq!(len, 32);

        // Check pubkey bytes (no flag, just the pubkey directly)
        let mut pubkey_bytes = [0u8; 32];
        for (i, b) in pubkey_bytes.iter_mut().enumerate() {
            *b = unsafe { destination[i].assume_init() };
        }
        assert_eq!(pubkey_bytes, test_pubkey.as_ref());
    }

    #[test]
    fn test_write_optional_pubkey_none() {
        let mut destination = [UNINIT_BYTE; 32];
        let pubkey = None;

        let len = write_optional_pubkey(&mut destination, pubkey);

        // Check length (always 32 bytes)
        assert_eq!(len, 32);

        // Check all bytes are zeros (Pubkey::default())
        for i in 0..32 {
            assert_eq!(unsafe { destination[i].assume_init() }, 0);
        }
    }

    #[test]
    fn test_transfer_hook_initialize_instruction_data() {
        // Test with both authority and program_id
        let authority = Pubkey::try_from([1u8; 32]).unwrap();
        let hook_program = Pubkey::try_from([2u8; 32]).unwrap();

        let mut instruction_data = [UNINIT_BYTE; 66];

        // Set discriminators
        write_bytes(
            &mut instruction_data[0..2],
            &[
                TRANSFER_HOOK_EXTENSION_DISCRIMINATOR,
                INITIALIZE_DISCRIMINATOR,
            ],
        );

        // Authority at [2..34]
        write_optional_pubkey(&mut instruction_data[2..], Some(&authority));

        // Program ID at [34..66]
        write_optional_pubkey(&mut instruction_data[34..], Some(&hook_program));

        // Check discriminators
        assert_eq!(
            unsafe { instruction_data[0].assume_init() },
            TRANSFER_HOOK_EXTENSION_DISCRIMINATOR
        );
        assert_eq!(
            unsafe { instruction_data[1].assume_init() },
            INITIALIZE_DISCRIMINATOR
        );

        // Check authority data at [2..34]
        let mut auth_bytes = [0u8; 32];
        for i in 0..32 {
            auth_bytes[i] = unsafe { instruction_data[2 + i].assume_init() };
        }
        assert_eq!(auth_bytes, authority.as_ref());

        // Check program_id data at [34..66]
        let mut program_bytes = [0u8; 32];
        for i in 0..32 {
            program_bytes[i] = unsafe { instruction_data[34 + i].assume_init() };
        }
        assert_eq!(program_bytes, hook_program.as_ref());
    }

    #[test]
    fn test_transfer_hook_initialize_minimal() {
        // Test with only authority (no program_id)
        let authority = Pubkey::try_from([1u8; 32]).unwrap();

        let mut instruction_data = [UNINIT_BYTE; 66];

        // Set discriminators
        write_bytes(
            &mut instruction_data[0..2],
            &[
                TRANSFER_HOOK_EXTENSION_DISCRIMINATOR,
                INITIALIZE_DISCRIMINATOR,
            ],
        );

        // Authority at [2..34]
        write_optional_pubkey(&mut instruction_data[2..], Some(&authority));

        // Program ID (None) at [34..66] - will be all zeros
        write_optional_pubkey(&mut instruction_data[34..], None);

        // Check authority data at [2..34]
        let mut auth_bytes = [0u8; 32];
        for i in 0..32 {
            auth_bytes[i] = unsafe { instruction_data[2 + i].assume_init() };
        }
        assert_eq!(auth_bytes, authority.as_ref());

        // Check program_id is all zeros (None) at [34..66]
        for i in 34..66 {
            assert_eq!(unsafe { instruction_data[i].assume_init() }, 0);
        }
    }

    #[test]
    fn test_transfer_hook_update_instruction_data() {
        let new_program = Pubkey::try_from([3u8; 32]).unwrap();

        let mut instruction_data = [UNINIT_BYTE; 34];

        // Set discriminators
        write_bytes(
            &mut instruction_data[0..2],
            &[TRANSFER_HOOK_EXTENSION_DISCRIMINATOR, UPDATE_DISCRIMINATOR],
        );

        // New program ID at [2..34]
        write_optional_pubkey(&mut instruction_data[2..], Some(&new_program));

        // Check discriminators
        assert_eq!(
            unsafe { instruction_data[0].assume_init() },
            TRANSFER_HOOK_EXTENSION_DISCRIMINATOR
        );
        assert_eq!(
            unsafe { instruction_data[1].assume_init() },
            UPDATE_DISCRIMINATOR
        );

        // Check program_id data at [2..34]
        let mut program_bytes = [0u8; 32];
        for i in 0..32 {
            program_bytes[i] = unsafe { instruction_data[2 + i].assume_init() };
        }
        assert_eq!(program_bytes, new_program.as_ref());
    }

    #[test]
    fn test_transfer_hook_update_disable() {
        // Test disabling transfer hook (None program_id)
        let mut instruction_data = [UNINIT_BYTE; 34];

        // Set discriminators
        write_bytes(
            &mut instruction_data[0..2],
            &[TRANSFER_HOOK_EXTENSION_DISCRIMINATOR, UPDATE_DISCRIMINATOR],
        );

        // New program ID (None - disabling) at [2..34] - will be all zeros
        write_optional_pubkey(&mut instruction_data[2..], None);

        // Check discriminators
        assert_eq!(
            unsafe { instruction_data[0].assume_init() },
            TRANSFER_HOOK_EXTENSION_DISCRIMINATOR
        );
        assert_eq!(
            unsafe { instruction_data[1].assume_init() },
            UPDATE_DISCRIMINATOR
        );

        // Check program_id is all zeros (None) at [2..34]
        for i in 2..34 {
            assert_eq!(unsafe { instruction_data[i].assume_init() }, 0);
        }
    }
}
