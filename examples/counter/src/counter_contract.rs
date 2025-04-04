use core::marker::PhantomData;
use borsh::{BorshDeserialize, BorshSerialize};
use sol_ez::{account::*, account_info::*, AccountData, DataSize};
mod pinocchio {
    pub use pinocchio::{program_error::ProgramError, account_info::AccountInfo};
}
#[derive(Debug, BorshDeserialize)]
pub struct InitialValue {
    pub value: u8,
}
#[derive(Debug, BorshSerialize, BorshDeserialize, AccountData)]
pub struct Count {
    pub data: u8,
}
pub struct Initialize<'info> {
    pub counter: Account<'info, PhantomData<Count>, Init>,
    pub signer: Account<'info, Signer, Mutable>,
}
impl<'info> Initialize<'info> {
    pub fn load(
        accounts: &'info [pinocchio::AccountInfo],
    ) -> Result<Self, pinocchio::ProgramError> {
        Ok(Self {
            counter: Account::new_init(
                AccountInfo::new_init(
                    accounts
                        .get(0usize)
                        .ok_or(pinocchio::ProgramError::NotEnoughAccountKeys)?,
                )?,
            ),
            signer: Account::new_singer(
                AccountInfo::new_mut(
                    accounts
                        .get(1usize)
                        .ok_or(pinocchio::ProgramError::NotEnoughAccountKeys)?,
                )?,
            )?,
        })
    }
}
pub struct Update<'info> {
    pub counter: Account<'info, Count, Mutable>,
    pub signer: Account<'info, Signer, Read>,
}
impl<'info> Update<'info> {
    pub fn load(
        accounts: &'info [pinocchio::AccountInfo],
    ) -> Result<Self, pinocchio::ProgramError> {
        Ok(Self {
            counter: Account::new(
                AccountInfo::new_mut(
                    accounts
                        .get(0usize)
                        .ok_or(pinocchio::ProgramError::NotEnoughAccountKeys)?,
                )?,
            )?,
            signer: Account::new_singer(
                AccountInfo::new_read(
                    accounts
                        .get(1usize)
                        .ok_or(pinocchio::ProgramError::NotEnoughAccountKeys)?,
                )?,
            )?,
        })
    }
}
pub struct Close<'info> {
    pub counter: Account<'info, Count, Mutable>,
    pub signer: Account<'info, Signer, Mutable>,
}
impl<'info> Close<'info> {
    pub fn load(
        accounts: &'info [pinocchio::AccountInfo],
    ) -> Result<Self, pinocchio::ProgramError> {
        Ok(Self {
            counter: Account::new(
                AccountInfo::new_mut(
                    accounts
                        .get(0usize)
                        .ok_or(pinocchio::ProgramError::NotEnoughAccountKeys)?,
                )?,
            )?,
            signer: Account::new_singer(
                AccountInfo::new_mut(
                    accounts
                        .get(1usize)
                        .ok_or(pinocchio::ProgramError::NotEnoughAccountKeys)?,
                )?,
            )?,
        })
    }
}
pub mod counter_contract {
    use core::marker::PhantomData;
    use pinocchio::{
        account_info::AccountInfo, ProgramResult, program_error::ProgramError,
        pubkey::Pubkey,
    };
    pub struct CounterDispatcher<T> {
        inner: PhantomData<T>,
    }
    impl<T> sol_ez::Contract for CounterDispatcher<T>
    where
        T: CounterContract,
    {
        fn dispatch<'info>(
            program_id: &Pubkey,
            accounts: &'info [AccountInfo],
            payload: &[u8],
        ) -> ProgramResult {
            let instruction_data = sol_ez::InstructionData::new(payload)?;
            match instruction_data.ix {
                [189u8, 191u8, 195u8, 215u8] => {
                    let ctx = sol_ez::Context {
                        program_id,
                        accounts: super::Initialize::load(accounts)?,
                    };
                    T::initialize(ctx, instruction_data.deserialize_data()?)
                }
                [67u8, 81u8, 28u8, 1u8] => {
                    let ctx = sol_ez::Context {
                        program_id,
                        accounts: super::Update::load(accounts)?,
                    };
                    T::update(ctx)
                }
                [102u8, 112u8, 45u8, 252u8] => {
                    let ctx = sol_ez::Context {
                        program_id,
                        accounts: super::Close::load(accounts)?,
                    };
                    T::close(ctx)
                }
                _ => Err(ProgramError::InvalidInstructionData),
            }
        }
    }
    pub trait CounterContract {
        fn initialize(
            accounts: sol_ez::Context<super::Initialize>,
            payload: super::InitialValue,
        ) -> ProgramResult;
        fn update(accounts: sol_ez::Context<super::Update>) -> ProgramResult;
        fn close(accounts: sol_ez::Context<super::Close>) -> ProgramResult;
    }
}
