#![no_std]
use pinocchio::{ProgramResult, account_info::AccountInfo, pubkey::Pubkey};

pub use account::{AccountData, DataSize};
pub use instruction_data::InstructionData;

pub mod account;
pub mod account_info;
pub mod instruction_data;

pub trait Contract {
    fn dispatch<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo],
        payload: &[u8],
    ) -> ProgramResult;
}

pub trait Seed<const D: usize, const N: usize> {
    const SEEDS: &[&[u8]; D];
    type Accounts;
    fn seeds(keys: &Self::Accounts) -> [&[u8]; N];
}

pub use sol_derive::AccountData;
