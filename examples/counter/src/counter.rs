use crate::counter_contract::{
    CloseAccounts, Count, CounterContract, CounterDispatcher, IncrementAccounts, InitalizeAccounts,
};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use pinocchio_log::log;
use sol_ez::Contract;

pub const FN: fn(&Pubkey, &[AccountInfo], &[u8]) -> Result<(), ProgramError> =
    CounterDispatcher::<MyCounter>::dispatch;

pub struct MyCounter;

impl CounterContract for MyCounter {
    fn initalize(
        owner: &Pubkey,
        mut accounts: InitalizeAccounts,
        amount: u8,
    ) -> Result<(), ProgramError> {
        let account = Count {
            authority: *accounts.user.key(),
            value: amount,
            bump: 0,
        };
        let counter = accounts.count.init(account, 0, &mut accounts.user, owner)?;
        log!("Counter initialized with value: {}", counter.as_ref().value);
        Ok(())
    }

    fn increment(_owner: &Pubkey, mut accounts: IncrementAccounts) -> Result<(), ProgramError> {
        validate(accounts.user.key(), accounts.count.as_ref())?;
        accounts.count.as_ref_mut().value += 1;
        let counter = accounts.count.apply()?;
        log!("Counter incremented to: {}", counter.as_ref().value);
        Ok(())
    }

    fn close(_owner: &Pubkey, mut accounts: CloseAccounts) -> Result<(), ProgramError> {
        validate(accounts.user.key(), accounts.count.as_ref())?;
        accounts.count.close(&mut accounts.user)?;
        log!("Counter closed");
        Ok(())
    }
}

fn validate(user_key: &Pubkey, count: &Count) -> Result<(), ProgramError> {
    if count.authority != *user_key {
        return Err(ProgramError::IllegalOwner);
    }
    Ok(())
}
