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

use super::Account;

pub trait AccountData {
    const SIZE: usize;
    const DISCRIMINATOR: [u8; 8];

    fn deserialize<'info, M>(account_info: &AccountInfo<'info, M>) -> Result<Self, ProgramError>
    where
        Self: BorshDeserialize,
        M: AccountRead,
    {
        let bytes = account_info.data();
        let (discriminator, data) = bytes.split_at(8);
        if discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self::try_from_slice(data).map_err(|_err| ProgramError::BorshIoError)?)
    }

    fn serialize<'info, M>(
        &self,
        account_info: &mut AccountInfo<'info, M>,
    ) -> Result<(), ProgramError>
    where
        Self: BorshSerialize,
        M: AccountWrite,
    {
        let account_data = &mut account_info.data_mut()[..];
        let (discriminator, mut data) = account_data.split_at_mut(8);
        discriminator.copy_from_slice(&Self::DISCRIMINATOR);
        BorshSerialize::serialize(&self, &mut data).map_err(|_err| ProgramError::BorshIoError)?;
        Ok(())
    }
}

impl<'info, T, P> Account<'info, T, P>
where
    T: AccountData,
{
    pub fn new(account_info: AccountInfo<'info, P>) -> Result<Self, ProgramError>
    where
        T: BorshDeserialize,
        P: AccountRead + 'info,
    {
        let inner = { <T as AccountData>::deserialize(&account_info)? };
        Ok(Account {
            inner,
            account_info,
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

    pub fn apply(mut self) -> Result<Account<'info, T, Read>, ProgramError>
    where
        T: BorshSerialize,
        P: AccountWrite,
    {
        AccountData::serialize(&self.inner, &mut self.account_info)?;
        Ok(Account {
            inner: self.inner,
            account_info: self.account_info.to_read(),
        })
    }

    pub fn close<D, DP>(self, signer: &mut Account<'info, D, DP>) -> Result<(), ProgramError>
    where
        P: AccountWrite + AccountRead,
        DP: AccountWrite + AccountRead,
    {
        let lamports = self.account_info.lamports();
        signer.account_info.add_lamports(lamports)?;
        self.account_info.close();
        Ok(())
    }
}

impl<'info, T> Account<'info, PhantomData<T>, Init>
where
    T: AccountData,
{
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
        mut self,
        account: T,
        payer: &mut Account<'info, P, PA>,
        owner: &Pubkey,
    ) -> Result<Account<'info, T, Read>, ProgramError>
    where
        T: BorshSerialize,
        PA: AccountWrite,
    {
        self.account_info.while_released(|account_info| {
            if !account_info.data_is_empty() {
                return Err(ProgramError::AccountAlreadyInitialized);
            }

            // TODO: set seed
            let seed = b"todo";
            let bump_seepd = &[255];
            let pda = pubkey::create_program_address(&[seed, bump_seepd], owner)?;

            if *account_info.key() != pda {
                return Err(ProgramError::InvalidAccountData);
            }

            let rent = Rent::get()?;
            let required_lamports = rent.minimum_balance(T::SIZE);

            payer.account_info.while_released(|payer| {
                CreateAccount {
                    from: payer,
                    to: account_info,
                    lamports: required_lamports,
                    space: T::SIZE as u64,
                    owner,
                }
                .invoke()
            })
        })?;

        AccountData::serialize(&account, &mut self.account_info)?;

        Ok(Account {
            inner: account,
            account_info: self.account_info.to_read(),
        })
    }
}
