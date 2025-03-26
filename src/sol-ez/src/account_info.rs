use core::marker::PhantomData;

use account_access_triat::{AccountRead, AccountWrite};

mod pinocchio {
    pub use pinocchio::{
        account_info::{AccountInfo, Ref, RefMut},
        program_error::ProgramError,
        pubkey::Pubkey,
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
    inner: &'info pinocchio::AccountInfo,
    _marker: PhantomData<M>,
}

impl<'info, M> AccountInfo<'info, M> {
    fn new(account_info: &'info pinocchio::AccountInfo) -> Self {
        AccountInfo {
            inner: account_info,
            _marker: PhantomData,
        }
    }

    pub fn raw_account_info(&self) -> &pinocchio::AccountInfo {
        self.inner
    }

    pub fn key(&self) -> &pinocchio::Pubkey {
        self.inner.key()
    }

    pub fn owner(&self, key: &pinocchio::Pubkey) -> bool {
        self.inner.is_owned_by(key)
    }

    pub fn is_signer(&self) -> bool {
        self.inner.is_signer()
    }

    pub fn data(&self) -> Result<pinocchio::Ref<[u8]>, pinocchio::ProgramError>
    where
        M: AccountRead,
    {
        self.inner.try_borrow_data()
    }

    pub fn data_mut(&self) -> Result<pinocchio::RefMut<[u8]>, pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        self.inner.try_borrow_mut_data()
    }

    pub fn data_is_empty(&self) -> bool {
        self.inner.data_is_empty()
    }

    pub fn lamports(&self) -> u64
    where
        M: AccountRead,
    {
        self.inner.lamports()
    }

    pub fn lamports_mut(&mut self) -> Result<pinocchio::RefMut<u64>, pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        self.inner.try_borrow_mut_lamports()
    }

    pub fn set_lamports(&mut self, lamports: u64) -> Result<(), pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        *self.inner.try_borrow_mut_lamports()? = lamports;
        Ok(())
    }

    pub fn add_lamports(&mut self, lamports: u64) -> Result<(), pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        self.set_lamports(
            self.inner
                .lamports()
                .checked_add(lamports)
                .ok_or(pinocchio::ProgramError::ArithmeticOverflow)?,
        )
    }

    pub fn sub_lamports(&mut self, lamports: u64) -> Result<(), pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        self.set_lamports(
            self.inner
                .lamports()
                .checked_sub(lamports)
                .ok_or(pinocchio::ProgramError::ArithmeticOverflow)?,
        )
    }

    pub fn zero_out_lamports(&mut self) -> Result<u64, pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        let lamports = self.inner.lamports();
        self.set_lamports(0)?;
        Ok(lamports)
    }

    pub unsafe fn assign(&self, owner: &pinocchio::Pubkey)
    where
        M: AccountWrite,
    {
        unsafe { self.inner.assign(owner) };
    }

    pub fn realloc(&self, len: usize, zero_init: bool) -> Result<(), pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        self.inner.realloc(len, zero_init)
    }
}

impl<'info> AccountInfo<'info, Init> {
    pub fn new_init(account_info: &'info pinocchio::AccountInfo) -> Self {
        Self::new(account_info)
    }
}

impl<'info> AccountInfo<'info, Read> {
    pub fn new_read(account_info: &'info pinocchio::AccountInfo) -> Self {
        Self::new(account_info)
    }
}

impl<'info> AccountInfo<'info, Mutable> {
    pub fn new_mut(
        account_info: &'info pinocchio::AccountInfo,
    ) -> Result<Self, pinocchio::ProgramError> {
        if !account_info.is_writable() {
            return Err(pinocchio::ProgramError::Immutable);
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
