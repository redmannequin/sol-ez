#![no_std]
use pinocchio::{
    ProgramResult, account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey,
};

pub use account::{AccountData, DataSize};
pub use context::Context;

pub mod account;
pub mod account_info;
pub mod context;

pub trait Contract {
    fn dispatch<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo],
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
