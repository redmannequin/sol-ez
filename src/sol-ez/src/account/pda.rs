use std::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, program::invoke_signed, program_error::ProgramError, pubkey::Pubkey,
    rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use super::{AccountRead, AccountWrite, Init, Mutable, Read};

pub trait AccountData {
    const SIZE: usize;
    const DISCRIMINATOR: [u8; 8];

    fn deserialize(account_info: &AccountInfo) -> Result<Self, ProgramError>
    where
        Self: BorshDeserialize,
    {
        let bytes = account_info.data.borrow();
        let (discriminator, data) = bytes.split_at(8);
        if discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self::try_from_slice(data)?)
    }

    fn serialize(&self, account_info: &AccountInfo) -> Result<(), ProgramError>
    where
        Self: BorshSerialize,
    {
        let account_data = &mut account_info.data.borrow_mut()[..];
        let (discriminator, mut data) = account_data.split_at_mut(8);
        discriminator.copy_from_slice(&Self::DISCRIMINATOR);
        BorshSerialize::serialize(&self, &mut data)?;
        Ok(())
    }
}

pub struct Account<'info, T, P> {
    inner: T,
    account_info: &'info AccountInfo<'info>,
    _mode: PhantomData<P>,
}

impl<'info, T, P> Account<'info, T, P> {
    fn new(account_info: &'info AccountInfo<'info>) -> Result<Self, ProgramError>
    where
        T: AccountData + BorshDeserialize,
    {
        let inner = <T as AccountData>::deserialize(account_info)?;
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

    pub fn verify_signer(&self) -> Result<(), ProgramError> {
        if !self.account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(())
    }

    pub fn verify_seed(&self, seed: &[u8], owner: &Pubkey) -> Result<(), ProgramError> {
        let (pda, _) = Pubkey::find_program_address(&[seed], owner);
        if *self.account_info.key != pda {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
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

    pub fn close<D, DP>(self, signer: &Account<'info, D, DP>) -> Result<(), ProgramError>
    where
        P: AccountWrite,
        DP: AccountWrite,
    {
        let dest_starting_lamports = signer.account_info.lamports();
        **signer.account_info.lamports.borrow_mut() = dest_starting_lamports
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
        T: AccountData,
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

        if *self.account_info.key != pda {
            return Err(ProgramError::InvalidAccountData);
        }

        let rent = Rent::get()?;
        let required_lamports = rent.minimum_balance(T::SIZE);

        let ix = system_instruction::create_account(
            payer.account_info.key,
            self.account_info.key,
            required_lamports,
            T::SIZE as u64,
            owner,
        );

        invoke_signed(
            &ix,
            &[
                payer.account_info.clone(),
                self.account_info.clone(),
                system_program.account_info.clone(),
            ],
            &[&[seed, &[bump_seed]]],
        )?;

        AccountData::serialize(&account, &self.account_info)?;

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
        T: AccountData + BorshDeserialize,
    {
        Account::new(account_info)
    }
}

impl<'info, T> Account<'info, T, Mutable> {
    pub fn new_mut(account_info: &'info AccountInfo<'info>) -> Result<Self, ProgramError>
    where
        T: AccountData + BorshDeserialize,
    {
        if !account_info.is_writable {
            return Err(ProgramError::Immutable);
        }
        Account::new(account_info)
    }
}
