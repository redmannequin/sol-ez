use account_access_triat::{AccountRead, AccountWrite};
pub use pda::{Account, AccountData};
pub use signer::Signer;

mod pda;
mod signer;

pub(crate) mod account_access_triat {
    pub trait AccountRead {}
    pub trait AccountWrite {}
}

/// Account Initalization Marker
pub struct Init;

/// Account Read Marker
pub struct Read;

/// Account Mutable Marker
pub struct Mutable;

impl AccountRead for Read {}
impl AccountRead for Mutable {}
impl AccountWrite for Mutable {}

pub trait DataSize {
    const SIZE: usize;
}

impl<T> DataSize for Box<T>
where
    T: DataSize,
{
    const SIZE: usize = T::SIZE;
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
