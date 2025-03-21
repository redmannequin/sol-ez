use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub use context::Context;

pub mod context;

pub trait Contract {
    fn dispatch<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        payload: &[u8],
    ) -> ProgramResult;
}
