pub mod group_member_pointer;
pub mod group_pointer;
pub mod token_group;

#[repr(u8)]
pub enum ExtensionDiscriminator {
    GroupPointer = 40,
    GroupMemberPointer = 41,
}
