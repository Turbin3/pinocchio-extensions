#![allow(unexpected_cfgs)]

use {
    pinocchio::{
        account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
        ProgramResult,
    },
    pinocchio_pubkey::declare_id,
    spl_token_2022_interface::{
        extension::{
            cpi_guard::instruction::CpiGuardInstruction,
            group_member_pointer::instruction::GroupMemberPointerInstruction,
            group_pointer::instruction::GroupPointerInstruction,
            scaled_ui_amount::instruction::ScaledUiAmountMintInstruction,
        },
        instruction::{decode_instruction_type, TokenInstruction},
    },
    spl_token_group_interface::instruction::{
        InitializeGroup, InitializeMember, TokenGroupInstruction, UpdateGroupAuthority,
        UpdateGroupMaxSize,
    },
};

pub mod helpers;
mod instructions;

use instructions as i;

entrypoint!(process_instruction);
declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match TokenInstruction::unpack(instruction_data) {
        // try to match TokenInstruction
        Ok(token_instruction) => {
            match token_instruction {
                TokenInstruction::InitializeMint {
                    decimals,
                    mint_authority,
                    freeze_authority,
                } => i::initialize_mint(accounts, decimals, mint_authority, freeze_authority),                

                TokenInstruction::CpiGuardExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: CpiGuardInstruction = decode_instruction_type(instruction_data)
                        .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        CpiGuardInstruction::Enable => {
                            i::cpi_guard::enable_guard(accounts, instruction_data)
                        }
                        CpiGuardInstruction::Disable => {
                            i::cpi_guard::disable_guard(accounts, instruction_data)
                        }
                    }
                }

                TokenInstruction::ScaledUiAmountExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: ScaledUiAmountMintInstruction = decode_instruction_type(instruction_data)
                        .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        ScaledUiAmountMintInstruction::Initialize => {
                            i::scaled_ui_amount::initialize_scaled_ui_amount(accounts, instruction_data)
                        }
                        ScaledUiAmountMintInstruction::UpdateMultiplier => {
                            i::scaled_ui_amount::update_multiplier(accounts, instruction_data)
                        }
                    }
                }

                _ => Err(ProgramError::InvalidInstructionData)?,
            }
        }
        Err(_) => {
            Err(ProgramError::InvalidInstructionData)?
        }
    }
}
