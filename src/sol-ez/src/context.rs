use std::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, program::invoke, program_error::ProgramError, pubkey::Pubkey,
    rent::Rent, system_instruction, sysvar::Sysvar,
};

pub struct Context<'a, T> {
    pub program_id: &'a Pubkey,
    pub accounts: T,
}

pub trait AccountRead {}
pub trait AccountWrite {}

pub struct Init;
pub struct Read;

impl AccountRead for Read {}

pub struct Mutable;

impl AccountRead for Mutable {}
impl AccountWrite for Mutable {}

pub struct Account<'a, T, P> {
    inner: T,
    account_info: &'a AccountInfo<'a>,
    _mode: PhantomData<P>,
}

impl<'a, T> Account<'a, PhantomData<T>, Init> {
    pub fn new_init(account_info: &'a AccountInfo<'a>) -> Self {
        Self {
            inner: PhantomData,
            account_info,
            _mode: PhantomData,
        }
    }

    pub fn init<'info, P, PA, S, SA>(
        self,
        account: T,
        payer: &Account<'info, P, PA>,
        system_program: &Account<'info, S, SA>,
        owner: &Pubkey,
    ) -> Result<Account<'a, T, Read>, ProgramError>
    where
        'a: 'info,
        'info: 'a,
        T: BorshSerialize,
        PA: AccountRead,
        SA: AccountRead,
    {
        // TODO: add space calcuation
        let account_space = 8;
        let rent = Rent::get()?;
        let required_lamports = rent.minimum_balance(account_space);

        let ix = system_instruction::create_account(
            payer.account_info.key,
            payer.account_info.key,
            required_lamports,
            account_space as u64,
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

impl<'a, T> Account<'a, T, Read> {
    pub fn new_read(account_info: &'a AccountInfo<'a>) -> Result<Account<'a, T, Read>, ProgramError>
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

    pub fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<'a, T> Account<'a, T, Mutable> {
    pub fn new_mut(
        account_info: &'a AccountInfo<'a>,
    ) -> Result<Account<'a, T, Mutable>, ProgramError>
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

    pub fn as_ref(&self) -> &T {
        &self.inner
    }

    pub fn as_ref_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn apply(self) -> Result<(), ProgramError>
    where
        T: BorshSerialize,
    {
        let mut account_data = &mut self.account_info.data.borrow_mut()[..];
        self.inner.serialize(&mut account_data)?;
        Ok(())
    }
}
