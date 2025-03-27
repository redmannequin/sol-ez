pub use account::{Account, AccountField, Type};
pub use accounts::{Accounts, AccountsField};
pub use contract::{Contract, Instruction};
pub use message::{Message, MessageField};

use crate::parse::token::Span;

mod account;
mod accounts;
mod contract;
mod message;

pub struct Definitions<'src> {
    pub message: Vec<Message<'src>>,
    pub account: Vec<Account<'src>>,
    pub accounts: Vec<Accounts<'src>>,
    pub contract: Contract<'src>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Identifer<'a> {
    pub span: Span,
    pub value: &'a str,
}
