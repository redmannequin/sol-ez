use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

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

pub trait AccountRent {
    const SIZE: usize;
}

impl AccountRent for u8 {
    const SIZE: usize = 1;
}

impl AccountRent for u64 {
    const SIZE: usize = 8;
}

impl AccountRent for f32 {
    const SIZE: usize = 4;
}

impl AccountRent for f64 {
    const SIZE: usize = 8;
}

pub use sol_derive::AccountRent;
