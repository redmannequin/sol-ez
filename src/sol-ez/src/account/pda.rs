use core::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{Sysvar, rent::Rent},
};
use pinocchio_system::instructions::CreateAccount;

use crate::account_info::{
    AccountInfo, Init, Read,
    account_access_triat::{AccountRead, AccountWrite},
};

pub trait AccountData {
    const SIZE: usize;
    const DISCRIMINATOR: [u8; 8];

    fn deserialize<'info, M>(account_info: &AccountInfo<'info, M>) -> Result<Self, ProgramError>
    where
        Self: BorshDeserialize,
        M: AccountRead,
    {
        let bytes = account_info.data()?;
        let (discriminator, data) = bytes.split_at(8);
        if discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self::try_from_slice(data).map_err(|_err| ProgramError::BorshIoError)?)
    }

    fn serialize<'info, M>(&self, account_info: &AccountInfo<'info, M>) -> Result<(), ProgramError>
    where
        Self: BorshSerialize,
        M: AccountWrite,
    {
        let account_data = &mut account_info.data_mut()?[..];
        let (discriminator, mut data) = account_data.split_at_mut(8);
        discriminator.copy_from_slice(&Self::DISCRIMINATOR);
        BorshSerialize::serialize(&self, &mut data).map_err(|_err| ProgramError::BorshIoError)?;
        Ok(())
    }
}

pub struct Account<'info, T, P> {
    pub(crate) inner: T,
    pub(crate) account_info: AccountInfo<'info, P>,
}

impl<'info, T, P> Account<'info, T, P> {
    pub fn new(account_info: AccountInfo<'info, P>) -> Result<Self, ProgramError>
    where
        T: AccountData + BorshDeserialize,
        P: AccountRead + 'info,
    {
        let inner = { <T as AccountData>::deserialize(&account_info)? };
        Ok(Account {
            inner,
            account_info,
        })
    }

    pub fn key(&self) -> &Pubkey {
        self.account_info.key()
    }

    pub fn lamports(&self) -> u64
    where
        P: AccountRead,
    {
        self.account_info.lamports()
    }

    pub fn set_lamports(&mut self, lamports: u64) -> Result<(), ProgramError>
    where
        P: AccountWrite,
    {
        self.account_info.set_lamports(lamports)
    }

    pub fn as_ref(&self) -> &T
    where
        T: AccountData,
        P: AccountRead,
    {
        &self.inner
    }

    pub fn as_ref_mut(&mut self) -> &mut T
    where
        T: AccountData,
        P: AccountWrite,
    {
        &mut self.inner
    }

    pub fn account_info(&self) -> &AccountInfo<'info, P>
    where
        P: AccountWrite,
    {
        &self.account_info
    }

    pub fn account_info_mut(&mut self) -> &AccountInfo<'info, P>
    where
        P: AccountWrite,
    {
        &self.account_info
    }

    pub fn verify_signer(&self) -> Result<(), ProgramError> {
        if !self.account_info.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(())
    }

    pub fn apply(self) -> Result<Account<'info, T, Read>, ProgramError>
    where
        T: AccountData + BorshSerialize,
        P: AccountWrite,
    {
        AccountData::serialize(&self.inner, &self.account_info)?;
        Ok(Account {
            inner: self.inner,
            account_info: self.account_info.to_read(),
        })
    }

    pub fn close<D, DP>(mut self, signer: &mut Account<'info, D, DP>) -> Result<(), ProgramError>
    where
        P: AccountWrite + AccountRead,
        DP: AccountWrite + AccountRead,
    {
        let lamports = self.account_info.zero_out_lamports()?;
        signer.account_info.add_lamports(lamports)?;
        self.account_info.raw_account_info().realloc(0, true)?;
        self.account_info.raw_account_info().close()?;
        Ok(())
    }
}

impl<'info, T> Account<'info, PhantomData<T>, Init> {
    pub fn new_init(account_info: AccountInfo<'info, Init>) -> Self
    where
        T: AccountData,
    {
        Self {
            inner: PhantomData,
            account_info,
        }
    }

    pub fn init<P, PA>(
        self,
        account: T,
        payer: &Account<'info, P, PA>,
        owner: &Pubkey,
    ) -> Result<Account<'info, T, Read>, ProgramError>
    where
        T: AccountData + BorshSerialize,
        PA: AccountWrite,
    {
        if !self.account_info.data_is_empty() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // TODO: set seed
        let seed = b"todo";
        let bump_seepd = &[255];
        let pda = pubkey::create_program_address(&[seed, bump_seepd], owner)?;

        if *self.account_info.key() != pda {
            return Err(ProgramError::InvalidAccountData);
        }

        let rent = Rent::get()?;
        let required_lamports = rent.minimum_balance(T::SIZE);

        CreateAccount {
            from: payer.account_info.raw_account_info(),
            to: self.account_info.raw_account_info(),
            lamports: required_lamports,
            space: T::SIZE as u64,
            owner,
        }
        .invoke()?;

        AccountData::serialize(&account, &self.account_info)?;

        Ok(Account {
            inner: account,
            account_info: self.account_info.to_read(),
        })
    }
}
