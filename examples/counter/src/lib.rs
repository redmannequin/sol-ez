#![no_std]
use counter_contract::{
    Close, Count, Initialize, Update,
    counter_contract::{CounterContract, CounterDispatcher},
};
use pinocchio::{
    ProgramResult, account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey,
};
use pinocchio_log::log;
use sol_ez::{Context, Contract};

// generated code
mod counter_contract;

type EFN =
    for<'a, 'b, 'info> fn(&'a Pubkey, &'info [AccountInfo], &'b [u8]) -> Result<(), ProgramError>;

pub const FN: EFN = CounterDispatcher::<Counter>::dispatch;

pub struct Counter;

impl CounterContract for Counter {
    fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        let mut signer = ctx.accounts.signer;
        let owner = ctx.program_id;

        signer.verify_signer()?;

        let account = Count { data: 0 };
        let counter = ctx.accounts.counter.init(account, &mut signer, owner)?;

        log!("Counter initialized with value: {}", counter.as_ref().data);

        Ok(())
    }

    fn update(mut ctx: Context<Update>) -> ProgramResult {
        ctx.accounts.signer.verify_signer()?;
        ctx.accounts.counter.as_ref_mut().data += 1;
        let counter = ctx.accounts.counter.apply()?;

        log!("Counter incremented to: {}", counter.as_ref().data);

        Ok(())
    }

    fn close(mut ctx: Context<Close>) -> ProgramResult {
        ctx.accounts.signer.verify_signer()?;
        ctx.accounts.counter.close(&mut ctx.accounts.signer)?;
        Ok(())
    }
}
