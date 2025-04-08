use core::marker::PhantomData;

use borsh::BorshDeserialize;
use pinocchio::program_error::ProgramError;

use crate::account_info::{AccountInfo, AccountRead, Empty, Immutable, Mutable, Signed, Unsigned};

use super::{Account, AccountData};

pub struct Set<T>(PhantomData<T>);

pub struct AccountBuilder<'info, T, M, S> {
    account_info: &'info pinocchio::account_info::AccountInfo,
    payload: PhantomData<T>,
    mutable: PhantomData<M>,
    signed: PhantomData<S>,
}

impl<'info> AccountBuilder<'info, Empty, Immutable, Unsigned> {
    pub fn new(account_info: &'info pinocchio::account_info::AccountInfo) -> Self {
        Self {
            account_info,
            payload: PhantomData,
            mutable: PhantomData,
            signed: PhantomData,
        }
    }
}

impl<'info, M, S> AccountBuilder<'info, Empty, M, S> {
    pub fn build(self) -> Result<Account<'info, Empty, M, S>, ProgramError> {
        Ok(Account {
            inner: Empty,
            account_info: AccountInfo::new(self.account_info)?,
        })
    }
}

impl<'info, T, M, S> AccountBuilder<'info, Set<T>, M, S> {
    pub fn build(self) -> Result<Account<'info, T, M, S>, ProgramError>
    where
        T: AccountData + BorshDeserialize,
        M: AccountRead,
    {
        Account::new(AccountInfo::new(self.account_info)?)
    }
}

impl<'info, M, S> AccountBuilder<'info, Empty, M, S> {
    pub fn set_payload<T>(self) -> AccountBuilder<'info, Set<T>, M, S>
    where
        T: AccountData + BorshDeserialize,
    {
        AccountBuilder {
            account_info: self.account_info,
            payload: PhantomData,
            mutable: PhantomData,
            signed: PhantomData,
        }
    }
}

impl<'info, T, S> AccountBuilder<'info, T, Immutable, S> {
    pub fn mutable(self) -> Result<AccountBuilder<'info, T, Mutable, S>, ProgramError> {
        if self.account_info.is_writable() {
            return Err(ProgramError::Immutable);
        }
        Ok(AccountBuilder {
            account_info: self.account_info,
            payload: PhantomData,
            mutable: PhantomData,
            signed: PhantomData,
        })
    }
}

impl<'info, T, M> AccountBuilder<'info, T, M, Unsigned> {
    pub fn signed(self) -> Result<AccountBuilder<'info, T, M, Signed>, ProgramError> {
        if !self.account_info.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(AccountBuilder {
            account_info: self.account_info,
            payload: PhantomData,
            mutable: PhantomData,
            signed: PhantomData,
        })
    }
}
