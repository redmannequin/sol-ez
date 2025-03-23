use std::{cell::RefMut, marker::PhantomData};

use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::account::{AccountRead, AccountWrite, Mutable, Read};

pub struct Signer<'info, M> {
    key: &'info Pubkey,
    account_info: AccountInfo<'info>,
    _mode: PhantomData<M>,
}

impl<'info, M> Signer<'info, M> {
    fn new(account_info: AccountInfo<'info>) -> Result<Self, ProgramError> {
        if !account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(Signer {
            key: account_info.key,
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

    pub fn lamports_mut(&self) -> RefMut<'info, &mut u64>
    where
        M: AccountWrite,
    {
        self.account_info.lamports.borrow_mut()
    }
}

impl<'info> Signer<'info, Read> {
    pub fn new_read(account_info: AccountInfo<'info>) -> Result<Self, ProgramError> {
        Self::new(account_info)
    }
}

impl<'info> Signer<'info, Mutable> {
    pub fn new_mut(account_info: AccountInfo<'info>) -> Result<Self, ProgramError> {
        if !account_info.is_writable {
            return Err(ProgramError::Immutable);
        }
        Self::new(account_info)
    }
}
