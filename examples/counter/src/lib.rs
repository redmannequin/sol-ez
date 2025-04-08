#![no_std]
use counter_contract::{
    CloseAccounts, Count, CounterContract, CounterDispatcher, IncrementAccounts, InitalizeAccounts,
};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use pinocchio_log::log;
use sol_ez::Contract;

// generated code
pub mod counter_contract;

type EFN =
    for<'a, 'b, 'info> fn(&'a Pubkey, &'info [AccountInfo], &'b [u8]) -> Result<(), ProgramError>;

pub const FN: EFN = CounterDispatcher::<Counter>::dispatch;

pub struct Counter;

impl CounterContract for Counter {
    fn initalize(
        owner: &Pubkey,
        mut accounts: InitalizeAccounts,
        amount: u8,
    ) -> Result<(), pinocchio::program_error::ProgramError> {
        let account = Count {
            authority: *accounts.user.key(),
            value: amount,
        };
        let counter = accounts.count.init(account, &mut accounts.user, owner)?;
        log!("Counter initialized with value: {}", counter.as_ref().value);
        Ok(())
    }

    fn increment(
        _owner: &Pubkey,
        mut accounts: IncrementAccounts,
    ) -> Result<(), pinocchio::program_error::ProgramError> {
        if accounts.count.as_ref().authority != *accounts.user.key() {
            return Err(ProgramError::IllegalOwner);
        }
        accounts.count.as_ref_mut().value += 1;
        let counter = accounts.count.apply()?;
        log!("Counter incremented to: {}", counter.as_ref().value);
        Ok(())
    }

    fn close(
        _owner: &Pubkey,
        mut accounts: CloseAccounts,
    ) -> Result<(), pinocchio::program_error::ProgramError> {
        if accounts.count.as_ref().authority != *accounts.user.key() {
            return Err(ProgramError::IllegalOwner);
        }
        accounts.count.close(&mut accounts.user)?;
        log!("Counter closed");
        Ok(())
    }
}
