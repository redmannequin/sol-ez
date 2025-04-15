use core::{marker::PhantomData, ptr};

use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{rent::Rent, Sysvar},
};
use pinocchio_system::instructions::CreateAccount;

use crate::{
    account_info::{AccountInfo, AccountRead, AccountWrite, Immutable, Init, Signed, Unsigned},
    split_at_fixed_unchecked,
};

use super::Account;

pub trait AccountDataConfig<const DISCRIMINATOR_SIZE: usize> {
    const DATA_SIZE: usize;
    const DISCRIMINATOR: [u8; DISCRIMINATOR_SIZE];
}

pub struct AccountData<const DISCRIMINATOR_SIZE: usize, T> {
    inner: T,
}

impl<const DISCRIMINATOR_SIZE: usize, T> AccountData<DISCRIMINATOR_SIZE, T>
where
    T: AccountDataConfig<DISCRIMINATOR_SIZE>,
{
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    #[cfg(feature = "std")]
    pub fn to_bytes(self) -> Result<std::vec::Vec<u8>, ProgramError>
    where
        T: BorshSerialize,
    {
        use std::vec;
        let mut bytes = vec![0; DISCRIMINATOR_SIZE + T::DATA_SIZE];
        let (discriminator, mut data) = bytes.split_at_mut(DISCRIMINATOR_SIZE);
        discriminator.copy_from_slice(&T::DISCRIMINATOR);
        BorshSerialize::serialize(&self.inner, &mut data)
            .map_err(|_err| ProgramError::BorshIoError)?;
        Ok(bytes)
    }

    fn deserialize<'info, M, S>(
        account_info: &AccountInfo<'info, M, S>,
    ) -> Result<Self, ProgramError>
    where
        T: BorshDeserialize,
        M: AccountRead,
    {
        let bytes = account_info.data();
        if bytes.len() != T::DATA_SIZE + DISCRIMINATOR_SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }
        // SAFETY: the account data size is already checked
        let (discriminator, data) = unsafe { split_at_fixed_unchecked(bytes) };
        if discriminator != &T::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self {
            inner: T::try_from_slice(data).map_err(|_err| ProgramError::BorshIoError)?,
        })
    }

    fn serialize<'info, M, S>(
        &self,
        account_info: &mut AccountInfo<'info, M, S>,
    ) -> Result<(), ProgramError>
    where
        T: BorshSerialize,
        M: AccountWrite,
    {
        let account_data = &mut account_info.data_mut()[..];
        if account_data.len() < T::DATA_SIZE + DISCRIMINATOR_SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        // SAFETY: the account data size is already checked
        let mut data = unsafe {
            let (discriminator, data) = account_data.split_at_mut_unchecked(DISCRIMINATOR_SIZE);
            ptr::copy_nonoverlapping(
                T::DISCRIMINATOR.as_ptr(),
                discriminator.as_mut_ptr(),
                DISCRIMINATOR_SIZE,
            );
            data
        };
        BorshSerialize::serialize(&self.inner, &mut data)
            .map_err(|_err| ProgramError::BorshIoError)?;
        Ok(())
    }
}

impl<'info, const DISCRIMINATOR_SIZE: usize, T, P, S>
    Account<'info, AccountData<DISCRIMINATOR_SIZE, T>, P, S>
where
    T: AccountDataConfig<DISCRIMINATOR_SIZE>,
{
    pub(crate) fn new(account_info: AccountInfo<'info, P, S>) -> Result<Self, ProgramError>
    where
        T: BorshDeserialize,
        P: AccountRead,
    {
        let inner = AccountData::deserialize(&account_info)?;
        Ok(Account {
            inner,
            account_info,
        })
    }

    pub fn as_ref(&self) -> &T
    where
        P: AccountRead,
    {
        &self.inner.inner
    }

    pub fn as_ref_mut(&mut self) -> &mut T
    where
        P: AccountWrite,
    {
        &mut self.inner.inner
    }

    pub fn apply(
        mut self,
    ) -> Result<Account<'info, AccountData<DISCRIMINATOR_SIZE, T>, Immutable, S>, ProgramError>
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

    pub fn close<D, DP>(
        self,
        signer: &mut Account<'info, D, DP, Signed>,
    ) -> Result<(), ProgramError>
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

impl<'info, const DISCRIMINATOR_SIZE: usize, T>
    Account<'info, PhantomData<AccountData<DISCRIMINATOR_SIZE, T>>, Init, Unsigned>
where
    T: AccountDataConfig<DISCRIMINATOR_SIZE>,
{
    pub fn new_init(account_info: AccountInfo<'info, Init, Unsigned>) -> Self {
        Self {
            inner: PhantomData,
            account_info,
        }
    }

    pub fn init<P, PA>(
        mut self,
        account: T,
        bump: u8,
        payer: &mut Account<'info, P, PA, Signed>,
        owner: &Pubkey,
    ) -> Result<Account<'info, AccountData<DISCRIMINATOR_SIZE, T>, Immutable, Unsigned>, ProgramError>
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
            let seeds = [seed.as_slice(), &[bump]];
            let pda = pubkey::create_program_address(&seeds, owner)?;

            if *account_info.key() != pda {
                return Err(ProgramError::IllegalOwner);
            }

            let rent = Rent::get()?;
            let required_lamports = rent.minimum_balance(T::DATA_SIZE + DISCRIMINATOR_SIZE);

            payer.account_info.while_released(|payer| {
                CreateAccount {
                    from: payer,
                    to: account_info,
                    lamports: required_lamports,
                    space: (T::DATA_SIZE + DISCRIMINATOR_SIZE) as u64,
                    owner,
                }
                .invoke_signed(&[Signer::from(&[Seed::from(seed), Seed::from(&[bump])])])?;
                Ok(())
            })
        })?;

        let account = AccountData { inner: account };
        AccountData::serialize(&account, &mut self.account_info)?;

        Ok(Account {
            inner: account,
            account_info: self.account_info.to_read(),
        })
    }
}
