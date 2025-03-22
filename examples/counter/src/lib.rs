use counter_contract::{
    Count, Initialize, Update,
    counter_contract::{CounterContract, CounterDispatcher},
};
use sol_ez::{Context, Contract};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

// generated code
mod counter_contract;

type EFN = for<'a, 'b, 'info> fn(
    &'a Pubkey,
    &'info [AccountInfo<'info>],
    &'b [u8],
) -> Result<(), ProgramError>;

pub const FN: EFN = CounterDispatcher::<Counter>::dispatch;

pub struct Counter;

impl CounterContract for Counter {
    fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        let payer = ctx.accounts.payer;
        let sys_program = ctx.accounts.program;
        let owner = ctx.program_id;

        let account = Count { data: 0 };
        let counter = ctx
            .accounts
            .counter
            .init(account, &payer, &sys_program, owner)?;

        msg!("Counter initialized with value: {}", counter.as_ref().data);

        Ok(())
    }

    fn update(mut ctx: Context<Update>) -> ProgramResult {
        ctx.accounts.counter.as_ref_mut().data += 1;
        let counter = ctx.accounts.counter.apply()?;

        msg!("Counter incremented to: {}", counter.as_ref().data);

        Ok(())
    }
}
