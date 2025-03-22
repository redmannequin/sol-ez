use std::marker::PhantomData;

use _account_access_triat::{AccountRead, AccountWrite};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, program::invoke, program_error::ProgramError, pubkey::Pubkey,
    rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use crate::AccountRent;

mod _account_access_triat {
    pub trait AccountRead {}
    pub trait AccountWrite {}
}

pub struct Init;
pub struct Read;

impl AccountRead for Read {}

pub struct Mutable;

impl AccountRead for Mutable {}
impl AccountWrite for Mutable {}

pub struct Account<'info, T, P> {
    inner: T,
    account_info: &'info AccountInfo<'info>,
    _mode: PhantomData<P>,
}

impl<'info, T, P> Account<'info, T, P> {
    fn new(account_info: &'info AccountInfo<'info>) -> Result<Self, ProgramError>
    where
        T: BorshDeserialize,
    {
        let inner = {
            let data = account_info.data.borrow_mut();
            T::try_from_slice(&data)?
        };

        Ok(Account {
            inner,
            account_info,
            _mode: PhantomData,
        })
    }

    pub fn as_ref(&self) -> &T
    where
        P: AccountRead,
    {
        &self.inner
    }

    pub fn as_ref_mut(&mut self) -> &mut T
    where
        P: AccountWrite,
    {
        &mut self.inner
    }

    pub fn apply(self) -> Result<Account<'info, T, Read>, ProgramError>
    where
        T: BorshSerialize,
        P: AccountWrite,
    {
        let mut account_data = &mut self.account_info.data.borrow_mut()[..];
        self.inner.serialize(&mut account_data)?;
        Ok(Account {
            inner: self.inner,
            account_info: self.account_info,
            _mode: PhantomData,
        })
    }

    pub fn close<D, DP>(self, sol_dest: &Account<'info, D, DP>) -> Result<(), ProgramError>
    where
        P: AccountWrite,
        DP: AccountWrite,
    {
        let dest_starting_lamports = sol_dest.account_info.lamports();
        **sol_dest.account_info.lamports.borrow_mut() = dest_starting_lamports
            .checked_add(self.account_info.lamports())
            .unwrap();
        **self.account_info.lamports.borrow_mut() = 0;

        self.account_info.assign(&system_program::ID);
        self.account_info.realloc(0, true)?;
        Ok(())
    }
}

impl<'info, T> Account<'info, PhantomData<T>, Init> {
    pub fn new_init(account_info: &'info AccountInfo<'info>) -> Self
    where
        T: AccountRent,
    {
        Self {
            inner: PhantomData,
            account_info,
            _mode: PhantomData,
        }
    }

    pub fn init<P, PA, S, SA>(
        self,
        account: T,
        payer: &Account<'info, P, PA>,
        system_program: &Account<'info, S, SA>,
        owner: &Pubkey,
    ) -> Result<Account<'info, T, Read>, ProgramError>
    where
        T: BorshSerialize + AccountRent,
        PA: AccountWrite,
        SA: AccountRead,
    {
        let rent = Rent::get()?;
        let required_lamports = rent.minimum_balance(T::SIZE);

        let ix = system_instruction::create_account(
            payer.account_info.key,
            self.account_info.key,
            required_lamports,
            T::SIZE as u64,
            owner,
        );

        invoke(
            &ix,
            &[
                payer.account_info.clone(),
                self.account_info.clone(),
                system_program.account_info.clone(),
            ],
        )?;

        {
            let mut account_data = &mut self.account_info.data.borrow_mut()[..];
            account.serialize(&mut account_data)?;
        }

        Ok(Account {
            inner: account,
            account_info: self.account_info,
            _mode: PhantomData,
        })
    }
}

impl<'info, T> Account<'info, T, Read> {
    pub fn new_read(account_info: &'info AccountInfo<'info>) -> Result<Self, ProgramError>
    where
        T: BorshDeserialize,
    {
        Account::new(account_info)
    }
}

impl<'info, T> Account<'info, T, Mutable> {
    pub fn new_mut(account_info: &'info AccountInfo<'info>) -> Result<Self, ProgramError>
    where
        T: BorshDeserialize,
    {
        if !account_info.is_writable {
            return Err(ProgramError::Immutable);
        }
        Account::new(account_info)
    }
}
