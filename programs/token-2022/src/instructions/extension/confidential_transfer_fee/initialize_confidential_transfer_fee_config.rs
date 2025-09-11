use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

use crate::instructions::{
    initialize_confidential_transfer_fee_config_instruction_data, ELGAMAL_PUBKEY_LEN,
};

/// Initializes confidential transfer fees for a mint.
///
/// The `ConfidentialTransferFeeInstruction::InitializeConfidentialTransferFeeConfig`
/// instruction requires no signers and MUST be included within the same
/// Transaction as `TokenInstruction::InitializeMint`. Otherwise another
/// party can initialize the configuration.
///
/// The instruction fails if the `TokenInstruction::InitializeMint`
/// instruction has already executed for the mint.
///
/// Accounts expected by this instruction:
///
///   0. `[writable]` The SPL Token mint.
///
/// Data expected by this instruction:
pub struct InitializeConfidentialTransferFeeConfig<'a, 'b> {
    /// The mint to initialize the confidential transfer fee config
    pub mint: &'a AccountInfo,
    /// The authority to set the withdraw withheld authority ElGamal key
    pub authority: Option<Pubkey>,
    /// The ElGamal public key for the withdraw withheld authority
    pub withdraw_withheld_authority_elgamal_pubkey: [u8; ELGAMAL_PUBKEY_LEN],
    /// Token Program
    pub token_program: &'b Pubkey,
}

impl InitializeConfidentialTransferFeeConfig<'_, '_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    #[inline(always)]
    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly_signer(self.authority.as_ref().unwrap()),
        ];

        let data = initialize_confidential_transfer_fee_config_instruction_data(
            self.authority,
            self.withdraw_withheld_authority_elgamal_pubkey,
        );

        let instruction = Instruction {
            program_id: &self.token_program,
            accounts: &account_metas,
            data,
        };

        invoke_signed(&instruction, &[self.mint], signers)?;

        Ok(())
    }
}
