use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

use crate::instructions::initialize_transfer_fee_config_instruction_data;

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
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        let data = initialize_transfer_fee_config_instruction_data(
            self.transfer_fee_config_authority,
            self.withdraw_withheld_authority,
            self.transfer_fee_basis_points,
            self.maximum_fee,
        );

        let instruction = Instruction {
            program_id: self.token_program,
            accounts: &account_metas,
            data,
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
