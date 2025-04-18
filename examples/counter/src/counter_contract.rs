use core::marker::PhantomData;
use borsh::{BorshDeserialize, BorshSerialize};
use sol_ez::{account::*, account_info::*, AccountData, AccountDataConfig, DataSize};
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};
#[derive(BorshSerialize, BorshDeserialize, AccountDataConfig)]
#[account_data(hash(seed = "counter|account|count", size = 4usize))]
pub struct Count {
    pub authority: [u8; 32],
    pub value: u8,
    pub bump: u8,
}
pub struct InitalizeAccounts<'info> {
    pub user: AccountWritableSigned<'info, Empty>,
    pub count: Account<'info, PhantomData<AccountData<4usize, Count>>, Init, Unsigned>,
}
impl<'info> InitalizeAccounts<'info> {
    pub fn load(
        accounts: &'info [pinocchio::account_info::AccountInfo],
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            user: AccountBuilder::new(
                    accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .mutable()?
                .signed()?
                .build()?,
            count: Account::new_init(
                AccountInfo::new_init(
                    accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )?,
            ),
        })
    }
}
pub struct IncrementAccounts<'info> {
    pub user: AccountWritableSigned<'info, Empty>,
    pub count: AccountWritable<'info, AccountData<4usize, Count>>,
}
impl<'info> IncrementAccounts<'info> {
    pub fn load(
        accounts: &'info [pinocchio::account_info::AccountInfo],
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            user: AccountBuilder::new(
                    accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .mutable()?
                .signed()?
                .build()?,
            count: AccountBuilder::new(
                    accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .set_payload()
                .mutable()?
                .build()?,
        })
    }
}
pub struct CloseAccounts<'info> {
    pub user: AccountWritableSigned<'info, Empty>,
    pub count: AccountWritable<'info, AccountData<4usize, Count>>,
}
impl<'info> CloseAccounts<'info> {
    pub fn load(
        accounts: &'info [pinocchio::account_info::AccountInfo],
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            user: AccountBuilder::new(
                    accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .mutable()?
                .signed()?
                .build()?,
            count: AccountBuilder::new(
                    accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .set_payload()
                .mutable()?
                .build()?,
        })
    }
}
pub trait CounterContract {
    fn initalize(
        program_id: &Pubkey,
        accounts: InitalizeAccounts,
        amount: u8,
    ) -> Result<(), ProgramError>;
    fn increment(
        program_id: &Pubkey,
        accounts: IncrementAccounts,
    ) -> Result<(), ProgramError>;
    fn close(program_id: &Pubkey, accounts: CloseAccounts) -> Result<(), ProgramError>;
}
pub struct CounterDispatcher<T> {
    inner: PhantomData<T>,
}
pub const INITALIZE: [u8; 4usize] = [165u8, 109u8, 64u8, 236u8];
pub const INCREMENT: [u8; 4usize] = [139u8, 113u8, 235u8, 106u8];
pub const CLOSE: [u8; 4usize] = [9u8, 199u8, 35u8, 185u8];
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
            &INITALIZE => {
                let accounts = InitalizeAccounts::load(accounts)?;
                let amount = ix_data.deserialize_data()?;
                T::initalize(program_id, accounts, amount)
            }
            &INCREMENT => {
                let accounts = IncrementAccounts::load(accounts)?;
                T::increment(program_id, accounts)
            }
            &CLOSE => {
                let accounts = CloseAccounts::load(accounts)?;
                T::close(program_id, accounts)
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
