//! A Solana Rust Framework

#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::{mem::MaybeUninit, ptr};

use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

pub use account::{AccountData, AccountDataConfig, DataSize};
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

pub use sol_derive::AccountDataConfig;

/// Initializes a `[T; N]` array from a byte slice without bounds checks.
///
/// # Safety
/// - `src.len()` **must** be at least `N`.
/// - If `src.len() < N`, this results in undefined behavior.
///
/// This function performs a raw, unchecked memory copy from the slice into a new `[T; N]` array.
/// It will assume that the caller has ensured the size and validity of the src.
///
/// # Panics
///
/// In debug builds, this will panic if `src.len() != N`.
/// ```
#[inline(always)]
pub unsafe fn init_from_slice_unchecked<const N: usize, T>(src: &[T]) -> [T; N]
where
    T: Copy,
{
    debug_assert!(src.len() >= N, "Slice length must be atleast N");
    let mut buf = MaybeUninit::<[T; N]>::uninit();
    ptr::copy_nonoverlapping(src.as_ptr(), buf.as_mut_ptr() as *mut T, N);
    buf.assume_init()
}

#[inline(always)]
pub unsafe fn split_at_fixed_unchecked<'a, const N: usize, T>(
    src: &'a [T],
) -> (&'a [T; N], &'a [T]) {
    debug_assert!(src.len() >= N, "Slice length must be atleast N");
    let (a, b) = src.split_at_unchecked(N);
    (slice_as_array_unchecked(a), b)
}

#[inline(always)]
pub unsafe fn slice_as_array_unchecked<'a, T, const N: usize>(slice: &'a [T]) -> &'a [T; N] {
    debug_assert_eq!(slice.len(), N, "Slice length must be exactly N");
    &*(slice.as_ptr() as *const [T; N])
}

#[inline(always)]
pub unsafe fn split_at_mut_fixed_unchecked<'a, const N: usize, T>(
    src: &'a mut [T],
) -> (&'a mut [T; N], &'a mut [T]) {
    debug_assert!(src.len() >= N, "Slice length must be at least N");
    let (a, b) = src.split_at_mut_unchecked(N);
    (slice_as_mut_array_unchecked(a), b)
}

#[inline(always)]
pub unsafe fn slice_as_mut_array_unchecked<'a, T, const N: usize>(
    slice: &'a mut [T],
) -> &'a mut [T; N] {
    debug_assert_eq!(slice.len(), N, "Slice length must be exactly N");
    &mut *(slice.as_mut_ptr() as *mut [T; N])
}
