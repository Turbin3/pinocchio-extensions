#![allow(unexpected_cfgs)]

use {
    crate::instructions::initialize_permanent_delegate,
    pinocchio::{
        account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
        ProgramResult,
    },
    pinocchio_pubkey::declare_id,
    spl_token_2022_interface::{
        extension::{
            group_member_pointer::instruction::GroupMemberPointerInstruction,
            group_pointer::instruction::GroupPointerInstruction,
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

                TokenInstruction::GroupPointerExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: GroupPointerInstruction = decode_instruction_type(instruction_data)
                        .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        GroupPointerInstruction::Initialize => {
                            i::group_pointer::initialize(accounts, instruction_data)
                        }
                        GroupPointerInstruction::Update => {
                            i::group_pointer::update(accounts, instruction_data)
                        }
                    }
                }

                TokenInstruction::GroupMemberPointerExtension => {
                    let instruction_data = &instruction_data[1..]; // Remove extension discriminator
                    let ix: GroupMemberPointerInstruction =
                        decode_instruction_type(instruction_data)
                            .map_err(|_| ProgramError::InvalidInstructionData)?;

                    match ix {
                        GroupMemberPointerInstruction::Initialize => {
                            i::group_member_pointer::initialize(accounts, instruction_data)
                        }
                        GroupMemberPointerInstruction::Update => {
                            i::group_member_pointer::update(accounts, instruction_data)
                        }
                    }
                }

                TokenInstruction::InitializePermanentDelegate { delegate } => {
                    initialize_permanent_delegate(accounts, delegate)
                }

                _ => Err(ProgramError::InvalidInstructionData)?,
            }
        }
        Err(_) => {
            // try to match TokenGroupInstruction
            match TokenGroupInstruction::unpack(instruction_data) {
                Ok(token_instruction) => match token_instruction {
                    TokenGroupInstruction::InitializeGroup(InitializeGroup {
                        update_authority,
                        max_size,
                    }) => i::token_group::initialize_group(accounts, update_authority, max_size),
                    TokenGroupInstruction::UpdateGroupMaxSize(UpdateGroupMaxSize { max_size }) => {
                        i::token_group::update_max_size(accounts, max_size)
                    }
                    TokenGroupInstruction::UpdateGroupAuthority(UpdateGroupAuthority {
                        new_authority,
                    }) => i::token_group::update_group_authority(accounts, new_authority),
                    TokenGroupInstruction::InitializeMember(InitializeMember) => {
                        i::token_group::initialize_member(accounts)
                    }
                },
                _ => Err(ProgramError::InvalidInstructionData)?,
            }
        }
    }
}
