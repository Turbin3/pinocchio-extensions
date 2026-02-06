pub mod memo_transfer;
pub mod non_transferrable;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    MemoTransfer = 30,
    InitializeNonTransferableMint = 32,
}
