use counter_contract::{
    Close, Count, Initialize, Update,
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
        let signer = ctx.accounts.signer;
        let sys_program = ctx.accounts.program;
        let owner = ctx.program_id;

        signer.verify_signer()?;

        let account = Count { data: 0 };
        let counter = ctx
            .accounts
            .counter
            .init(account, &signer, &sys_program, owner)?;

        msg!("Counter initialized with value: {}", counter.as_ref().data);

        Ok(())
    }

    fn update(mut ctx: Context<Update>) -> ProgramResult {
        ctx.accounts.signer.verify_signer()?;
        ctx.accounts.counter.as_ref_mut().data += 1;
        let counter = ctx.accounts.counter.apply()?;

        msg!("Counter incremented to: {}", counter.as_ref().data);

        Ok(())
    }

    fn close(ctx: Context<Close>) -> ProgramResult {
        ctx.accounts.signer.verify_signer()?;
        ctx.accounts.counter.close(&ctx.accounts.signer)?;
        Ok(())
    }
}
