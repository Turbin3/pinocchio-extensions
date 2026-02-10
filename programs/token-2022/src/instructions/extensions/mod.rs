pub mod default_account_state;
pub mod memo_transfer;
pub mod non_transferrable;
pub mod transfer_hook;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    DefaultAccountState = 28,
    MemoTransfer = 30,
    InitializeNonTransferableMint = 32,
    TransferHook = 36,
}
