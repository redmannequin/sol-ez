use core::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{rent::Rent, Sysvar},
};
use pinocchio_system::instructions::CreateAccount;

use crate::account_info::{
    AccountInfo, AccountRead, AccountWrite, Immutable, Init, Signed, Unsigned,
};

use super::Account;

pub trait Discriminator {
    const SIZE: usize;
    fn as_bytes(&self) -> &[u8];
}

impl<const N: usize> Discriminator for [u8; N] {
    const SIZE: usize = N;

    fn as_bytes(&self) -> &[u8] {
        self
    }
}

pub trait AccountData {
    const SIZE: usize;
    const DISCRIMINATOR: Self::DiscriminatorKind;
    type DiscriminatorKind: Discriminator;

    fn deserialize<'info, M, S>(
        account_info: &AccountInfo<'info, M, S>,
    ) -> Result<Self, ProgramError>
    where
        Self: BorshDeserialize,
        M: AccountRead,
    {
        let bytes = account_info.data();
        if bytes.len() < Self::SIZE {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // SAFETY: the account data size is already checked
        let (discriminator, data) =
            unsafe { bytes.split_at_unchecked(Self::DiscriminatorKind::SIZE) };

        if discriminator != Self::DISCRIMINATOR.as_bytes() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self::try_from_slice(data).map_err(|_err| ProgramError::BorshIoError)?)
    }

    fn serialize<'info, M, S>(
        &self,
        account_info: &mut AccountInfo<'info, M, S>,
    ) -> Result<(), ProgramError>
    where
        Self: BorshSerialize,
        M: AccountWrite,
    {
        let account_data = &mut account_info.data_mut()[..];
        if account_data.len() < Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }

        // SAFETY: the account data size is already checked
        let (discriminator, mut data) =
            unsafe { account_data.split_at_mut_unchecked(Self::DiscriminatorKind::SIZE) };

        discriminator.copy_from_slice(Self::DISCRIMINATOR.as_bytes());
        BorshSerialize::serialize(&self, &mut data).map_err(|_err| ProgramError::BorshIoError)?;
        Ok(())
    }
}

impl<'info, T, P, S> Account<'info, T, P, S>
where
    T: AccountData,
{
    pub(crate) fn new(account_info: AccountInfo<'info, P, S>) -> Result<Self, ProgramError>
    where
        T: BorshDeserialize,
        P: AccountRead,
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

    pub fn apply(mut self) -> Result<Account<'info, T, Immutable, S>, ProgramError>
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

impl<'info, T> Account<'info, PhantomData<T>, Init, Unsigned>
where
    T: AccountData,
{
    pub fn new_init(account_info: AccountInfo<'info, Init, Unsigned>) -> Self
    where
        T: AccountData,
    {
        Self {
            inner: PhantomData,
            account_info,
        }
    }

    pub fn init<P, PA, PS>(
        mut self,
        account: T,
        bump: u8,
        payer: &mut Account<'info, P, PA, PS>,
        owner: &Pubkey,
    ) -> Result<Account<'info, T, Immutable, Unsigned>, ProgramError>
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
            let pda = pubkey::create_program_address(&[seed, &[bump]], owner)?;

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
                .invoke()?;
                Ok(())
            })
        })?;

        AccountData::serialize(&account, &mut self.account_info)?;

        Ok(Account {
            inner: account,
            account_info: self.account_info.to_read(),
        })
    }
}
