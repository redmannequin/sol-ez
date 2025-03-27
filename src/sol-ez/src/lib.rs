#![no_std]
use pinocchio::{ProgramResult, account_info::AccountInfo, pubkey::Pubkey};

pub use account::{AccountData, DataSize};
pub use context::Context;
pub use instruction_data::InstructionData;

pub mod account;
pub mod account_info;
pub mod context;
pub mod instruction_data;

pub trait Contract {
    fn dispatch<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo],
        payload: &[u8],
    ) -> ProgramResult;
}

pub use sol_derive::AccountData;
