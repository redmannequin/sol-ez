pub use pda::AccountData;
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};
pub use signer::Signer;

use crate::account_info::{
    AccountInfo,
    account_access_triat::{AccountRead, AccountWrite},
};

mod pda;
mod signer;

pub trait DataSize {
    const SIZE: usize;
}

pub struct Account<'info, T, P> {
    pub(crate) inner: T,
    pub(crate) account_info: AccountInfo<'info, P>,
}

impl<'info, T, P> Account<'info, T, P> {
    pub fn key(&self) -> &Pubkey {
        self.account_info.key()
    }

    pub fn owner(&self) -> &Pubkey {
        self.account_info.owner()
    }

    pub fn lamports(&self) -> u64
    where
        P: AccountRead,
    {
        self.account_info.lamports()
    }

    pub fn set_lamports(&mut self, lamports: u64)
    where
        P: AccountWrite,
    {
        self.account_info.set_lamports(lamports)
    }

    pub fn account_info(&self) -> &AccountInfo<'info, P>
    where
        P: AccountWrite,
    {
        &self.account_info
    }

    pub fn account_info_mut(&mut self) -> &mut AccountInfo<'info, P>
    where
        P: AccountWrite,
    {
        &mut self.account_info
    }

    pub fn verify_signer(&self) -> Result<(), ProgramError> {
        if !self.account_info.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(())
    }
}

macro_rules! impl_data_size {
    ($ty:ty => $size:literal) => {
        impl DataSize for $ty {
            const SIZE: usize = $size;
        }
    };
    ($($ty:ty => $size:literal,)+) => {
        $(impl_data_size!($ty => $size);)+
    }
}

impl_data_size!(
    u8   => 1,
    u16  => 2,
    u32  => 4,
    u64  => 8,
    u128 => 16,
    i8   => 1,
    i16  => 2,
    i32  => 4,
    i64  => 8,
    i128 => 16,
    f32  => 4,
    f64  => 8,
    bool => 1,
    char => 4,
);
