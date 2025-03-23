use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

pub use account::{AccountData, DataSize};
pub use context::Context;

pub mod account;
pub mod context;

pub trait Contract {
    fn dispatch<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        payload: &[u8],
    ) -> ProgramResult;
}

pub trait Payload: Sized {
    fn load(bytes: &[u8]) -> Result<Self, ProgramError>;
}

pub trait Accounts: Sized {
    fn load(account_infos: &[AccountInfo]) -> Result<Self, ProgramError>;
}

pub use sol_derive::AccountData;
