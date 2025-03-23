use std::marker::PhantomData;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};
use sol_ez::{account::*, AccountData, DataSize};
#[derive(Debug, BorshSerialize, BorshDeserialize, AccountData)]
pub struct SysProgram {}
#[derive(Debug, BorshSerialize, BorshDeserialize, AccountData)]
pub struct Signer {}
#[derive(Debug, BorshSerialize, BorshDeserialize, AccountData)]
pub struct Count {
    pub data: u8,
}
pub struct Initialize<'a> {
    pub counter: Account<'a, PhantomData<Count>, Init>,
    pub signer: Account<'a, Signer, Mutable>,
    pub program: Account<'a, SysProgram, Read>,
}
impl<'a> Initialize<'a> {
    pub fn load(accounts: &'a [AccountInfo<'a>]) -> Result<Self, ProgramError> {
        Ok(Self {
            counter: Account::new_init(
                accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
            ),
            signer: Account::new_mut(
                accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
            )?,
            program: Account::new_read(
                accounts.get(2usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
            )?,
        })
    }
}
pub struct Update<'a> {
    pub counter: Account<'a, Count, Mutable>,
    pub signer: Account<'a, Signer, Read>,
}
impl<'a> Update<'a> {
    pub fn load(accounts: &'a [AccountInfo<'a>]) -> Result<Self, ProgramError> {
        Ok(Self {
            counter: Account::new_mut(
                accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
            )?,
            signer: Account::new_read(
                accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
            )?,
        })
    }
}
pub struct Close<'a> {
    pub counter: Account<'a, Count, Mutable>,
    pub signer: Account<'a, Signer, Mutable>,
}
impl<'a> Close<'a> {
    pub fn load(accounts: &'a [AccountInfo<'a>]) -> Result<Self, ProgramError> {
        Ok(Self {
            counter: Account::new_mut(
                accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
            )?,
            signer: Account::new_mut(
                accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
            )?,
        })
    }
}
pub mod counter_contract {
    use solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult,
        program_error::ProgramError, pubkey::Pubkey,
    };
    pub struct CounterDispatcher<T> {
        inner: std::marker::PhantomData<T>,
    }
    impl<T> sol_ez::Contract for CounterDispatcher<T>
    where
        T: CounterContract,
    {
        fn dispatch<'info>(
            program_id: &Pubkey,
            accounts: &'info [AccountInfo<'info>],
            payload: &[u8],
        ) -> ProgramResult {
            let (instruction, _rest) = payload
                .split_first()
                .ok_or(ProgramError::InvalidInstructionData)?;
            match instruction {
                1u8 => {
                    let ctx = sol_ez::Context {
                        program_id,
                        accounts: super::Initialize::load(accounts)?,
                    };
                    T::initialize(ctx)
                }
                2u8 => {
                    let ctx = sol_ez::Context {
                        program_id,
                        accounts: super::Update::load(accounts)?,
                    };
                    T::update(ctx)
                }
                3u8 => {
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
        fn initialize(accounts: sol_ez::Context<super::Initialize>) -> ProgramResult;
        fn update(accounts: sol_ez::Context<super::Update>) -> ProgramResult;
        fn close(accounts: sol_ez::Context<super::Close>) -> ProgramResult;
    }
}
