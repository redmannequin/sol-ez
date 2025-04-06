use core::marker::PhantomData;

use account_access_triat::{AccountRead, AccountWrite};

mod pinocchio {
    pub use pinocchio::{
        account_info::{AccountInfo, RefMut},
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
pub struct ReadOnly;

/// Account Mutable Marker
pub struct Mutable;

/// Account Unsigned Marker
pub struct Unsigned;

/// Account Signed Marker
pub struct Signed;

// Mark Account with no payload
pub struct Empty;

impl AccountRead for ReadOnly {}
impl AccountRead for Mutable {}
impl AccountWrite for Init {}
impl AccountWrite for Mutable {}

struct AccountGuard<'info> {
    data: Option<pinocchio::RefMut<'info, [u8]>>,
    lamports: Option<pinocchio::RefMut<'info, u64>>,
}

impl<'info> AccountGuard<'info> {
    pub fn lock(
        account_info: &'info pinocchio::AccountInfo,
    ) -> Result<Self, pinocchio::ProgramError> {
        Ok(Self {
            data: Some(account_info.try_borrow_mut_data()?),
            lamports: Some(account_info.try_borrow_mut_lamports()?),
        })
    }

    pub fn release(&mut self) {
        self.data.take();
        self.lamports.take();
    }
}

pub struct AccountInfo<'info, M, S> {
    inner: &'info pinocchio::AccountInfo,
    guard: AccountGuard<'info>,
    _mutable_marker: PhantomData<M>,
    _signed_markser: PhantomData<S>,
}

impl<'info, M, S> AccountInfo<'info, M, S> {
    pub fn new(
        account_info: &'info pinocchio::AccountInfo,
    ) -> Result<Self, pinocchio::ProgramError> {
        let guard = AccountGuard::lock(account_info)?;
        Ok(AccountInfo {
            inner: account_info,
            guard,
            _mutable_marker: PhantomData,
            _signed_markser: PhantomData,
        })
    }

    pub fn to_raw_account_info(self) -> &'info pinocchio::AccountInfo {
        self.inner
    }

    pub fn while_released(
        &mut self,
        f: impl FnOnce(&pinocchio::AccountInfo) -> Result<(), pinocchio::ProgramError>,
    ) -> Result<(), pinocchio::ProgramError> {
        self.guard.release();
        f(self.inner)?;
        self.guard = AccountGuard::lock(self.inner)?;
        Ok(())
    }

    pub fn key(&self) -> &pinocchio::Pubkey {
        self.inner.key()
    }

    pub fn is_signer(&self) -> bool {
        self.inner.is_signer()
    }

    /// Retruns the owner of the account.
    pub fn owner(&self) -> &pinocchio::Pubkey {
        // SAFETY: The borrow checker ensures that `assign` cannot be called
        // while a reference from `owner` exists.
        unsafe { self.inner.owner() }
    }

    /// Assigns new owner to the account.
    pub fn assign(&mut self, owner: &pinocchio::Pubkey)
    where
        M: AccountWrite,
    {
        // SAFETY: Because this function requires `&mut self`, Rust guarantees
        // that no `&self` references exist, making the operation safe.
        unsafe { self.inner.assign(owner) };
    }

    pub fn data(&self) -> &[u8]
    where
        M: AccountRead,
    {
        unsafe { self.inner.borrow_data_unchecked() }
    }

    pub fn data_mut(&mut self) -> &mut [u8]
    where
        M: AccountWrite,
    {
        unsafe { self.inner.borrow_mut_data_unchecked() }
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

    pub fn lamports_mut(&mut self) -> &mut u64
    where
        M: AccountWrite,
    {
        unsafe { self.inner.borrow_mut_lamports_unchecked() }
    }

    pub fn set_lamports(&mut self, lamports: u64)
    where
        M: AccountWrite,
    {
        unsafe {
            *self.inner.borrow_mut_lamports_unchecked() = lamports;
        }
    }

    pub fn add_lamports(&mut self, lamports: u64) -> Result<(), pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        Ok(self.set_lamports(
            self.inner
                .lamports()
                .checked_add(lamports)
                .ok_or(pinocchio::ProgramError::ArithmeticOverflow)?,
        ))
    }

    pub fn sub_lamports(&mut self, lamports: u64) -> Result<(), pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        Ok(self.set_lamports(
            self.inner
                .lamports()
                .checked_sub(lamports)
                .ok_or(pinocchio::ProgramError::ArithmeticOverflow)?,
        ))
    }

    pub fn zero_out_lamports(&mut self) -> Result<u64, pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        let lamports = self.inner.lamports();
        self.set_lamports(0);
        Ok(lamports)
    }

    pub fn realloc(&mut self, len: usize, zero_init: bool) -> Result<(), pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        self.inner.realloc(len, zero_init)
    }

    pub fn close(self)
    where
        M: AccountWrite,
    {
        unsafe { self.inner.close_unchecked() };
    }
}

impl<'info, S> AccountInfo<'info, Init, S> {
    pub fn new_init(
        account_info: &'info pinocchio::AccountInfo,
    ) -> Result<Self, pinocchio::ProgramError> {
        Self::new(account_info)
    }
}

impl<'info, M, S> AccountInfo<'info, M, S>
where
    M: AccountWrite,
{
    pub fn to_read(self) -> AccountInfo<'info, ReadOnly, S> {
        AccountInfo {
            inner: self.inner,
            guard: self.guard,
            _mutable_marker: PhantomData,
            _signed_markser: PhantomData,
        }
    }
}
