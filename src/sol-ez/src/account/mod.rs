pub use pda::AccountData;
use pinocchio::pubkey::Pubkey;

use crate::account_info::{
    AccountInfo,
    account_access_triat::{AccountRead, AccountWrite},
};

pub use builder::AccountBuilder;

mod builder;
mod pda;

pub trait DataSize {
    const SIZE: usize;
}

/// A high-level wrapper around `AccountInfo` that provides data deserialization,
/// serialization, and additional convenience methods for interacting with accounts.
///
/// The `Account` struct builds on the `AccountInfo` wrapper by adding support for
/// deserializing and serializing account data, providing easier access to mutable
/// or immutable account data, and handling high-level account operations like validation
/// or initialization.
///
/// ## Type Parameters
///
/// - `'info`: The lifetime of the underlying account.
/// - `T`: The type of the account's data (typically a struct that can be deserialized).
/// - `P`: The account's access level:
///     - [`Init`]: The account is uninitialized and will be created.
///     - [`Mutable`]: The account is already initialized and writable.
///     - [`Immutable`]: The account is read-only.
/// - `S`: The signer constraint:
///     - [`Signed`]: The account must be signed.
///     - [`Unsigned`]: No signature constraint.
pub struct Account<'info, T, P, S> {
    pub(crate) inner: T,
    pub(crate) account_info: AccountInfo<'info, P, S>,
}

impl<'info, T, P, S> Account<'info, T, P, S> {
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

    pub fn account_info(&self) -> &AccountInfo<'info, P, S>
    where
        P: AccountWrite,
    {
        &self.account_info
    }

    pub fn account_info_mut(&mut self) -> &mut AccountInfo<'info, P, S>
    where
        P: AccountWrite,
    {
        &mut self.account_info
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
