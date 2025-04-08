use core::marker::PhantomData;

use account_access_triat::{AccountRead, AccountWrite};
pub use markers::{Empty, Immutable, Init, Mutable, Signed, Unsigned};

mod pinocchio {
    pub use pinocchio::{
        account_info::{AccountInfo, RefMut},
        program_error::ProgramError,
        pubkey::Pubkey,
    };
}
/// Traits used as markers for account access types.
///
/// These traits serve as markers for Solana account access levels in the context of
/// the [`AccountInfo`] struct. The traits allow compile-time enforcement of whether
/// an account can be **read** or **written** to, based on its associated marker.
pub(crate) mod account_access_triat {
    /// Marker trait indicating that an account can be **read** but not modified.
    pub trait AccountRead {}
    /// Marker trait indicating that an account can be **written** to (modified).
    pub trait AccountWrite {}
}

/// Account markers.
///
/// These markers are used to provide compile-time guarantees about the type of access or state
/// of an account.
///  
/// Each of the markers is used in conjunction with the `AccountInfo` struct to provide type-level
/// guarantees about an account's access rights and constraints, helping to enforce correct access
/// patterns and prevent misuse.
pub mod markers {
    use super::account_access_triat::{AccountRead, AccountWrite};

    /// Account initialization marker.
    ///
    /// The `Init` marker is used for accounts that need to be **initialized**. This marker is used
    /// for **newly created accounts** that must be initialized  as part of the program's execution.
    /// Accounts marked with `Init` are  associated with an initialization function  to ensure the
    /// account is properly set up with valid data before being used.
    ///
    /// This marker ensures that the account is created and initialized, and it enables access
    /// to functions that are specifically designed to initialize accounts (such as setting up data
    /// or lamports).
    pub struct Init;

    /// Account read-only marker.
    ///
    /// The `Immutable` marker is used for accounts that are **read-only**. Accounts marked
    /// with `Immutable` cannot be modified and can only be accessed for reading. This marker
    /// ensures that any attempt to modify the account's data (such as using `borrow_mut_data()` or
    /// performing any write operation) will result in a compile-time error, providing strong safety
    /// guarantees.
    ///
    /// This marker is ideal for accounts that store configuration or other data that should not be
    /// altered during program execution.
    pub struct Immutable;

    /// Account mutable marker.
    ///
    /// The `Mutable` marker is used for accounts that are **mutable**. Accounts marked with `Mutable`
    /// can be both read and written to, enabling full access to the account's data and lamports.
    /// This marker ensures that the account can be modified during program execution, allowing functions
    /// that perform write operations (such as `mut_data()` or modifying lamports) to be used.
    ///
    /// This marker is ideal for accounts that need to be updated, such as accounts that store user balances,
    /// state, or other data that can change during program execution.
    pub struct Mutable;

    /// Account unsigned marker.
    ///
    /// The `Unsigned` marker is used for accounts that **do not require a signature**. Accounts marked with
    /// `Unsigned` do not need to be signed by the account holder for the operation to succeed. This marker
    /// ensures that no signature check is enforced for the account during the execution of the program.
    ///
    /// This is useful for accounts that are used purely for data storage or other operations that do not
    /// require any user authentication, such as configuration accounts or system accounts.
    pub struct Unsigned;

    /// Account signed marker.
    ///
    /// The `Signed` marker is used for accounts that **require a signature**. Accounts marked with `Signed`
    /// must be signed by the account holder in order to proceed with the operation. This marker ensures that
    /// the program will enforce the signature check for the account during execution.
    ///
    /// This is useful for accounts where user authorization is necessary, such as for transferring funds
    /// or making other changes to sensitive account data.
    pub struct Signed;

    /// Account with no data payload.
    ///
    /// The `Empty` marker is used for accounts that do **not** contain any data and are not
    /// intended to participate in any data access operations (such as reading or writing account data).
    /// This marker helps ensure that no operations are attempted on accounts that should remain empty.
    ///
    /// Accounts marked with `Empty` will not have access to the account's data, and methods
    /// like `data()` or `mut_data()` will not be available.
    ///
    /// ## Use Cases:
    /// - Placeholder accounts: Accounts that exist for system purposes but don't need to store any data.
    /// - Non-participatory accounts: Accounts that are not involved in the main program logic but need to
    ///   be managed in the program's state.
    pub struct Empty;

    impl AccountRead for Immutable {}
    impl AccountRead for Mutable {}
    impl AccountWrite for Init {}
    impl AccountWrite for Mutable {}
}

/// Internal guard that ensures a Solana account is only wrapped once.
///
/// `AccountGuard` is used by `AccountInfo` to ensure that a account is only
/// accessed once at a time, by locking both its `data` and `lamports`. The lock
/// is acquired via the `lock` method, and it ensures that no other `AccountInfo`
/// wrapper can access the same account, preventing accidental or unsafe aliasing.
///
/// ## Locking Behavior
///
/// - `AccountGuard::lock` borrows both the account's data and lamports mutably.
/// - If another borrow already exists for either field, it will return an error.
/// - This guard prevents further borrows until it is explicitly released via the `release` method.
///
/// ## Safety
///
/// This guard is created during [`AccountInfo::new`], which is called once per
/// account during the program's dispatch phase. If multiple `AccountInfo` wrappers
/// are constructed for the same underlying account, the second `new()` call will fail.
struct AccountGuard<'info> {
    data: Option<pinocchio::RefMut<'info, [u8]>>,
    lamports: Option<pinocchio::RefMut<'info, u64>>,
}

impl<'info> AccountGuard<'info> {
    /// Locks the account, acquiring mutable references to both its data and lamports.
    /// This prevents further mutable borrows of the account until the lock is released.
    ///
    /// # Errors
    ///
    /// If the account has already been borrowed (data or lamports), this will
    /// return an error.
    pub fn lock(
        account_info: &'info pinocchio::AccountInfo,
    ) -> Result<Self, pinocchio::ProgramError> {
        Ok(Self {
            data: Some(account_info.try_borrow_mut_data()?),
            lamports: Some(account_info.try_borrow_mut_lamports()?),
        })
    }

    /// Releases the lock on the account's data and lamports, allowing other parts
    /// of the program to borrow the account again.
    pub fn release(&mut self) {
        self.data.take();
        self.lamports.take();
    }
}

/// A typed wrapper around [`pinocchio::AccountInfo`] with guarantees on access
/// level and signer constraints.
///
/// `AccountInfo<'info, M, S>` encapsulates a account reference and encodes:
///
/// - `'info`: The lifetime of the underlying account
/// - `M`: The account's **access type**, such as [`Init`], [`Mutable`], or [`Immutable`]
/// - `S`: The account's **signer status**, such as [`Signed`] or [`Unsigned`]
///
/// These type parameters are **zero-cost phantom types**. They are not used at runtime,
/// but enable method-level enforcement of capabilities via type and traits like
/// [`AccountRead`] and [`AccountWrite`].
///
/// ## Capabilities
///
/// - Only types `M: AccountRead` may call methods that take a `&self`
/// - Only types `M: AccountWrite` may call method take take a `&self` and `&mut self`
/// - Only types `S = Singer` may be used as signed accounts
///
/// This system ensures at compile time that accounts are used correctly,
/// eliminating many classes of runtime errors.
///
/// ## Fields
///
/// These fields are intentionally private to preserve safety invariants.
/// Use framework-provided traits and methods to interact with the account.
///
/// - `inner`: A reference to the raw [`pinocchio::AccountInfo`].
/// - `guard`: Internal guard to enforce runtime safety (e.g. borrow rules).
/// - `_mutable_marker`: Encodes access type at the type level
/// - `_signed_markser`: Encodes signer constraint at the type level
pub struct AccountInfo<'info, M, S> {
    inner: &'info pinocchio::AccountInfo,
    guard: AccountGuard<'info>,
    _mutable_marker: PhantomData<M>,
    _signed_markser: PhantomData<S>,
}

impl<'info, M, S> AccountInfo<'info, M, S> {
    pub fn new(
        account_info: &'info pinocchio::AccountInfo,
    ) -> Result<Self, pinocchio::ProgramError> {
        let guard = AccountGuard::lock(account_info)?;
        Ok(AccountInfo {
            inner: account_info,
            guard,
            _mutable_marker: PhantomData,
            _signed_markser: PhantomData,
        })
    }

    pub fn to_raw_account_info(self) -> &'info pinocchio::AccountInfo {
        self.inner
    }

    pub fn while_released(
        &mut self,
        f: impl FnOnce(&pinocchio::AccountInfo) -> Result<(), pinocchio::ProgramError>,
    ) -> Result<(), pinocchio::ProgramError> {
        self.guard.release();
        f(self.inner)?;
        self.guard = AccountGuard::lock(self.inner)?;
        Ok(())
    }

    pub fn key(&self) -> &pinocchio::Pubkey {
        self.inner.key()
    }

    pub fn is_signer(&self) -> bool {
        self.inner.is_signer()
    }

    /// Retruns the owner of the account.
    pub fn owner(&self) -> &pinocchio::Pubkey {
        // SAFETY: The borrow checker ensures that `assign` cannot be called
        // while a reference from `owner` exists.
        unsafe { self.inner.owner() }
    }

    /// Assigns new owner to the account.
    pub fn assign(&mut self, owner: &pinocchio::Pubkey)
    where
        M: AccountWrite,
    {
        // SAFETY: Because this function requires `&mut self`, Rust guarantees
        // that no `&self` references exist, making the operation safe.
        unsafe { self.inner.assign(owner) };
    }

    pub fn data(&self) -> &[u8]
    where
        M: AccountRead,
    {
        unsafe { self.inner.borrow_data_unchecked() }
    }

    pub fn data_mut(&mut self) -> &mut [u8]
    where
        M: AccountWrite,
    {
        unsafe { self.inner.borrow_mut_data_unchecked() }
    }

    pub fn data_is_empty(&self) -> bool {
        self.inner.data_is_empty()
    }

    pub fn lamports(&self) -> u64
    where
        M: AccountRead,
    {
        self.inner.lamports()
    }

    pub fn lamports_mut(&mut self) -> &mut u64
    where
        M: AccountWrite,
    {
        unsafe { self.inner.borrow_mut_lamports_unchecked() }
    }

    pub fn set_lamports(&mut self, lamports: u64)
    where
        M: AccountWrite,
    {
        unsafe {
            *self.inner.borrow_mut_lamports_unchecked() = lamports;
        }
    }

    pub fn add_lamports(&mut self, lamports: u64) -> Result<(), pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        Ok(self.set_lamports(
            self.inner
                .lamports()
                .checked_add(lamports)
                .ok_or(pinocchio::ProgramError::ArithmeticOverflow)?,
        ))
    }

    pub fn sub_lamports(&mut self, lamports: u64) -> Result<(), pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        Ok(self.set_lamports(
            self.inner
                .lamports()
                .checked_sub(lamports)
                .ok_or(pinocchio::ProgramError::ArithmeticOverflow)?,
        ))
    }

    pub fn zero_out_lamports(&mut self) -> Result<u64, pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        let lamports = self.inner.lamports();
        self.set_lamports(0);
        Ok(lamports)
    }

    pub fn realloc(&mut self, len: usize, zero_init: bool) -> Result<(), pinocchio::ProgramError>
    where
        M: AccountWrite,
    {
        self.inner.realloc(len, zero_init)
    }

    pub fn close(self)
    where
        M: AccountWrite,
    {
        unsafe { self.inner.close_unchecked() };
    }
}

impl<'info, S> AccountInfo<'info, Init, S> {
    pub fn new_init(
        account_info: &'info pinocchio::AccountInfo,
    ) -> Result<Self, pinocchio::ProgramError> {
        Self::new(account_info)
    }
}

impl<'info, M, S> AccountInfo<'info, M, S>
where
    M: AccountWrite,
{
    pub fn to_read(self) -> AccountInfo<'info, Immutable, S> {
        AccountInfo {
            inner: self.inner,
            guard: self.guard,
            _mutable_marker: PhantomData,
            _signed_markser: PhantomData,
        }
    }
}
