//! A Solana Rust Framework

#![no_std]
use core::{mem::MaybeUninit, ptr};

use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

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
    const SEEDS: &'static [&'static [u8]; D];
    type Accounts;
    fn seeds(keys: &Self::Accounts) -> [&[u8]; N];
}

pub use sol_derive::AccountData;

pub(crate) trait ArrayExt<const N: usize, T> {
    /// Initializes a `[T; N]` array from a byte slice without bounds checks.
    ///
    /// # Safety
    /// - `src.len()` **must** be exactly `N`.
    /// - If `src.len() != N`, this results in undefined behavior.
    ///
    /// This function performs a raw, unchecked memory copy from the slice into a new `[T; N]` array.
    /// It will assume that the caller has ensured the size and validity of the src.
    ///
    /// # Panics
    ///
    /// In debug builds, this will panic if `src.len() != N`.
    /// ```
    unsafe fn init_from_slice_unchecked(src: &[T]) -> Self
    where
        T: Copy;
}

impl<const N: usize, T> ArrayExt<N, T> for [T; N] {
    unsafe fn init_from_slice_unchecked(src: &[T]) -> Self
    where
        T: Copy,
    {
        debug_assert!(src.len() == N);
        let mut buf = MaybeUninit::<[T; N]>::uninit();
        ptr::copy_nonoverlapping(src.as_ptr(), buf.as_mut_ptr() as *mut T, N);
        buf.assume_init()
    }
}
