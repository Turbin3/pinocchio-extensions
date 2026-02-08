pub mod memo_transfer;
pub mod mint_close_authority;

#[repr(u8)]
#[non_exhaustive]
pub enum ExtensionDiscriminator {
    MemoTransfer = 30,
    MintCloseAuthority = 25,
}
