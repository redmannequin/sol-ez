use core::marker::PhantomData;
use borsh::{BorshDeserialize, BorshSerialize};
use sol_ez::{account::*, account_info::*, AccountData, DataSize};
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};
pub struct CounterDispatcher<T> {
    inner: PhantomData<T>,
}
impl<T> sol_ez::Contract for CounterDispatcher<T>
where
    T: CounterContract,
{
    fn dispatch<'info>(
        program_id: &Pubkey,
        accounts: &'info [pinocchio::account_info::AccountInfo],
        payload: &[u8],
    ) -> Result<(), ProgramError> {
        let ix_data = sol_ez::InstructionData::new(payload)?;
        match ix_data.ix {
            [154u8, 238u8, 251u8, 74u8] => {
                T::close(program_id, CloseAccounts::load(accounts)?)
            }
            [45u8, 80u8, 125u8, 159u8] => {
                T::increment(program_id, IncrementAccounts::load(accounts)?)
            }
            [150u8, 198u8, 209u8, 78u8] => {
                let amount = ix_data.deserialize_data()?;
                T::initalize(program_id, InitalizeAccounts::load(accounts)?, amount)
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
pub trait CounterContract {
    fn close(owner: &Pubkey, accounts: CloseAccounts) -> Result<(), ProgramError>;
    fn increment(
        owner: &Pubkey,
        accounts: IncrementAccounts,
    ) -> Result<(), ProgramError>;
    fn initalize(
        owner: &Pubkey,
        accounts: InitalizeAccounts,
        amount: u8,
    ) -> Result<(), ProgramError>;
}
#[derive(Debug, BorshSerialize, BorshDeserialize, AccountData)]
pub struct Count {
    pub value: u8,
}
pub struct CloseAccounts<'info> {
    pub count: Account<'info, Count, Mutable, Unsigned>,
    pub user: Account<'info, Empty, Mutable, Signed>,
}
impl<'info> CloseAccounts<'info> {
    pub fn load(
        accounts: &'info [pinocchio::account_info::AccountInfo],
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            count: AccountBuilder::new(
                    accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .set_payload()
                .mutable()?
                .build()?,
            user: AccountBuilder::new(
                    accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .mutable()?
                .signed()?
                .build()?,
        })
    }
}
pub struct IncrementAccounts<'info> {
    pub count: Account<'info, Count, Mutable, Unsigned>,
    pub user: Account<'info, Empty, Mutable, Signed>,
}
impl<'info> IncrementAccounts<'info> {
    pub fn load(
        accounts: &'info [pinocchio::account_info::AccountInfo],
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            count: AccountBuilder::new(
                    accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .set_payload()
                .mutable()?
                .build()?,
            user: AccountBuilder::new(
                    accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .mutable()?
                .signed()?
                .build()?,
        })
    }
}
pub struct InitalizeAccounts<'info> {
    pub count: Account<'info, PhantomData<Count>, Init, Unsigned>,
    pub user: Account<'info, Empty, Mutable, Signed>,
}
impl<'info> InitalizeAccounts<'info> {
    pub fn load(
        accounts: &'info [pinocchio::account_info::AccountInfo],
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            count: Account::new_init(
                AccountInfo::new_init(
                    accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )?,
            ),
            user: AccountBuilder::new(
                    accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .mutable()?
                .signed()?
                .build()?,
        })
    }
}
