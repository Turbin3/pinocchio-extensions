use {
    crate::ID,
    pinocchio::{
        account_info::{AccountInfo, Ref},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[repr(C)]
pub struct GroupMemberPointer {
    /// Authority that can set the member address
    authority: Pubkey,
    /// Account address that holds the member
    member_address: Pubkey,
}

impl GroupMemberPointer {
    /// The length of the mint with `GroupMemberPointer` extension data
    const LEN: u8 = 234;
    /// The index where authority address starts in the mint with `GroupMemberPointer` extension data
    const AUTHORITY_START: u8 = 170;

    /// The length of the `GroupMemberPointer` extension data.
    pub const BASE_LEN: usize = core::mem::size_of::<GroupMemberPointer>();

    /// Return a `GroupMemberPointer` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<Ref<GroupMemberPointer>, ProgramError> {
        // Check data length first
        if account_info.data_len() < Self::LEN as usize {
            Err(ProgramError::InvalidAccountData)?;
        }

        // Check owner
        if account_info.owner() != &ID {
            Err(ProgramError::InvalidAccountOwner)?;
        }

        // Safely borrow and map the data
        let data_ref = account_info
            .try_borrow_data()
            .map_err(|_| ProgramError::AccountBorrowFailed)?;

        Ok(Ref::map(data_ref, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    /// Return a `GroupMemberPointer` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, but does not
    /// perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account data (e.g., there are
    /// no mutable borrows of the account data).
    #[inline]
    pub unsafe fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&Self, ProgramError> {
        // Check data length first
        if account_info.data_len() < Self::LEN as usize {
            Err(ProgramError::InvalidAccountData)?;
        }

        // Check owner
        if account_info.owner() != &ID {
            Err(ProgramError::InvalidAccountOwner)?;
        }

        // Get unchecked borrow and convert
        let data = account_info.borrow_data_unchecked();
        Ok(Self::from_bytes_unchecked(data))
    }

    /// Return a `GroupMemberPointer` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// 1. `bytes` contains at least `LEN` bytes
    /// 2. `bytes` contains a valid representation of `GroupMemberPointer`
    /// 3. The data is properly aligned (though GroupMemberPointer has alignment of 1)
    /// 4. The bytes represent valid flag values and pubkey data
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes[Self::AUTHORITY_START as usize..].as_ptr() as *const GroupMemberPointer)
    }

    /// Safe version of from_bytes that performs validation
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ProgramError> {
        if bytes.len() < Self::LEN as usize {
            Err(ProgramError::InvalidAccountData)?;
        }

        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    #[inline(always)]
    pub fn has_authority(&self) -> bool {
        self.authority != Pubkey::default()
    }

    #[inline]
    pub fn authority(&self) -> Option<&Pubkey> {
        if self.has_authority() {
            Some(&self.authority)
        } else {
            None
        }
    }

    /// Return the authority.
    ///
    /// This method should be used when the caller knows that the group member pointer will have an
    /// authority set since it skips the `Option` check.
    #[inline(always)]
    pub fn authority_unchecked(&self) -> &Pubkey {
        &self.authority
    }

    #[inline(always)]
    pub fn has_member_address(&self) -> bool {
        self.member_address != Pubkey::default()
    }

    #[inline]
    pub fn member_address(&self) -> Option<&Pubkey> {
        if self.has_member_address() {
            Some(&self.member_address)
        } else {
            None
        }
    }

    /// Return the member address.
    ///
    /// This method should be used when the caller knows that the group member pointer will have a
    /// member address set since it skips the `Option` check.
    #[inline(always)]
    pub fn member_address_unchecked(&self) -> &Pubkey {
        &self.member_address
    }
}
