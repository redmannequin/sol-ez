pub use account::{Account, AccountField, Type};
pub use accounts::{Accounts, AccountsField};
pub use contract::{Contract, Instruction};

use crate::parse::token::Span;

mod account;
mod accounts;
mod contract;

pub struct Identifer<'a> {
    pub span: Span,
    pub value: &'a str,
}
