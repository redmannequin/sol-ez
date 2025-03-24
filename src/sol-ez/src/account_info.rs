use std::marker::PhantomData;

use account_access_triat::{AccountRead, AccountWrite};

mod solana_program {
    pub use solana_program::{
        account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey,
    };
}

pub(crate) mod account_access_triat {
    pub trait AccountRead {}
    pub trait AccountWrite {}
}

/// Account Initalization Marker
pub struct Init;

/// Account Read Marker
pub struct Read;

/// Account Mutable Marker
pub struct Mutable;

impl AccountRead for Read {}
impl AccountRead for Mutable {}
impl AccountWrite for Init {}
impl AccountWrite for Mutable {}

pub struct AccountInfo<'info, M> {
    inner: &'info solana_program::AccountInfo<'info>,
    _marker: PhantomData<M>,
}

impl<'info, M> AccountInfo<'info, M> {
    fn new(account_info: &'info solana_program::AccountInfo<'info>) -> Self {
        AccountInfo {
            inner: account_info,
            _marker: PhantomData,
        }
    }

    pub fn raw_account_info(&self) -> &solana_program::AccountInfo<'info> {
        self.inner
    }

    pub fn key(&self) -> &solana_program::Pubkey {
        self.inner.key
    }

    pub fn owner(&self) -> &solana_program::Pubkey {
        self.inner.owner
    }

    pub fn is_signer(&self) -> bool {
        self.inner.is_signer
    }

    pub fn data(&self) -> std::cell::Ref<&mut [u8]>
    where
        M: AccountRead,
    {
        self.inner.data.borrow()
    }

    pub fn data_mut(&self) -> std::cell::RefMut<'_, &'info mut [u8]>
    where
        M: AccountWrite,
    {
        self.inner.data.borrow_mut()
    }

    pub fn data_is_empty(&self) -> bool {
        self.inner.data_is_empty()
    }

    pub fn lamports(&self) -> u64
    where
        M: AccountRead,
    {
        **self.inner.lamports.borrow()
    }

    pub fn lamports_mut(&self) -> std::cell::RefMut<&'info mut u64>
    where
        M: AccountWrite,
    {
        self.inner.lamports.borrow_mut()
    }

    pub fn assign(&self, owner: &solana_program::Pubkey)
    where
        M: AccountWrite,
    {
        self.inner.assign(owner);
    }

    pub fn realloc(&self, len: usize, zero_init: bool) -> Result<(), solana_program::ProgramError>
    where
        M: AccountWrite,
    {
        self.inner.realloc(len, zero_init)
    }
}

impl<'info> AccountInfo<'info, Init> {
    pub fn new_init(account_info: &'info solana_program::AccountInfo<'info>) -> Self {
        Self::new(account_info)
    }
}

impl<'info> AccountInfo<'info, Read> {
    pub fn new_read(account_info: &'info solana_program::AccountInfo<'info>) -> Self {
        Self::new(account_info)
    }
}

impl<'info> AccountInfo<'info, Mutable> {
    pub fn new_mut(
        account_info: &'info solana_program::AccountInfo<'info>,
    ) -> Result<Self, solana_program::ProgramError> {
        if !account_info.is_writable {
            return Err(solana_program::ProgramError::Immutable);
        }
        Ok(Self::new(account_info))
    }
}

impl<'info, M> AccountInfo<'info, M>
where
    M: AccountWrite,
{
    pub fn to_read(self) -> AccountInfo<'info, Read> {
        AccountInfo::new_read(self.inner)
    }
}
