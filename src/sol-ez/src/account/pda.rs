use std::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program::invoke_signed, program_error::ProgramError, pubkey::Pubkey, rent::Rent,
    system_instruction, system_program, sysvar::Sysvar,
};

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
        let bytes = account_info.data();
        let (discriminator, data) = bytes.split_at(8);
        if discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self::try_from_slice(data)?)
    }

    fn serialize<'info, M>(&self, account_info: &AccountInfo<'info, M>) -> Result<(), ProgramError>
    where
        Self: BorshSerialize,
        M: AccountWrite,
    {
        let account_data = &mut account_info.data_mut()[..];
        let (discriminator, mut data) = account_data.split_at_mut(8);
        discriminator.copy_from_slice(&Self::DISCRIMINATOR);
        BorshSerialize::serialize(&self, &mut data)?;
        Ok(())
    }
}

pub struct Account<'info, T, P> {
    inner: T,
    account_info: AccountInfo<'info, P>,
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

    pub fn as_ref(&self) -> &T
    where
        P: AccountRead,
    {
        &self.inner
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

    pub fn as_ref_mut(&mut self) -> &mut T
    where
        P: AccountWrite,
    {
        &mut self.inner
    }

    pub fn verify_signer(&self) -> Result<(), ProgramError> {
        if !self.account_info.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(())
    }

    pub fn verify_seed(&self, seed: &[u8], owner: &Pubkey) -> Result<(), ProgramError> {
        let (pda, _) = Pubkey::find_program_address(&[seed], owner);
        if *self.account_info.key() != pda {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    pub fn apply(self) -> Result<Account<'info, T, Read>, ProgramError>
    where
        T: BorshSerialize,
        P: AccountWrite,
    {
        {
            let mut account_data = &mut self.account_info.data_mut()[..];
            self.inner.serialize(&mut account_data)?;
        }
        Ok(Account {
            inner: self.inner,
            account_info: self.account_info.to_read(),
        })
    }

    pub fn close<D, DP>(self, signer: &Account<'info, D, DP>) -> Result<(), ProgramError>
    where
        P: AccountWrite + AccountRead,
        DP: AccountWrite + AccountRead,
    {
        let dest_starting_lamports = signer.account_info.lamports();
        **signer.account_info.lamports_mut() = dest_starting_lamports
            .checked_add(self.account_info.lamports())
            .unwrap();
        **self.account_info.lamports_mut() = 0;

        self.account_info.assign(&system_program::ID);
        self.account_info.realloc(0, true)?;
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

    pub fn init<P, PA, S, SA>(
        self,
        account: T,
        payer: &Account<'info, P, PA>,
        system_program: &Account<'info, S, SA>,
        owner: &Pubkey,
    ) -> Result<Account<'info, T, Read>, ProgramError>
    where
        T: AccountData + BorshSerialize,
        PA: AccountWrite,
        SA: AccountRead,
    {
        if !self.account_info.data_is_empty() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // TODO: set seed
        let seed = b"todo";
        let (pda, bump_seed) = Pubkey::find_program_address(&[seed], owner);

        if *self.account_info.key() != pda {
            return Err(ProgramError::InvalidAccountData);
        }

        let rent = Rent::get()?;
        let required_lamports = rent.minimum_balance(T::SIZE);

        let ix = system_instruction::create_account(
            payer.account_info.key(),
            self.account_info.key(),
            required_lamports,
            T::SIZE as u64,
            owner,
        );

        invoke_signed(
            &ix,
            &[
                payer.account_info.raw_account_info().clone(),
                self.account_info.raw_account_info().clone(),
                system_program.account_info.raw_account_info().clone(),
            ],
            &[&[seed, &[bump_seed]]],
        )?;

        AccountData::serialize(&account, &self.account_info)?;

        Ok(Account {
            inner: account,
            account_info: self.account_info.to_read(),
        })
    }
}
