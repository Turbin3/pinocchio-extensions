#[repr(u8)]
pub enum ExtensionDiscriminator {
    PermanentDelegate = 35,
    GroupPointer = 40,
    GroupMemberPointer = 41,
}
