#[repr(u8)]
pub enum ExtensionDiscriminator {
    DefaultAccountState = 28,
    PermanentDelegate = 35,
    GroupPointer = 40,
    GroupMemberPointer = 41,
}
