use core::marker::PhantomData;

use pinocchio::{
    account_info::{AccountInfo, RefMut},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::account_info::{
    Mutable, Read,
    account_access_triat::{AccountRead, AccountWrite},
};

pub struct Signer<'info, M> {
    key: &'info Pubkey,
    account_info: &'info AccountInfo,
    _mode: PhantomData<M>,
}

impl<'info, M> Signer<'info, M> {
    fn new(account_info: &'info AccountInfo) -> Result<Self, ProgramError> {
        if !account_info.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(Signer {
            key: account_info.key(),
            account_info,
            _mode: PhantomData,
        })
    }

    pub fn key(&self) -> &Pubkey
    where
        M: AccountRead,
    {
        self.key
    }

    pub fn lamports(&self) -> u64
    where
        M: AccountRead,
    {
        self.account_info.lamports()
    }

    pub fn lamports_mut(&self) -> Result<RefMut<u64>, ProgramError>
    where
        M: AccountWrite,
    {
        self.account_info.try_borrow_mut_lamports()
    }
}

impl<'info> Signer<'info, Read> {
    pub fn new_read(account_info: &'info AccountInfo) -> Result<Self, ProgramError> {
        Self::new(account_info)
    }
}

impl<'info> Signer<'info, Mutable> {
    pub fn new_mut(account_info: &'info AccountInfo) -> Result<Self, ProgramError> {
        if !account_info.is_writable() {
            return Err(ProgramError::Immutable);
        }
        Self::new(account_info)
    }
}
