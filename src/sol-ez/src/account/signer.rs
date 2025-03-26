use crate::account_info::AccountInfo;

use super::Account;

pub struct Signer;

pub type SignerAccount<'info, M> = Account<'info, Signer, M>;

impl<'info, M> SignerAccount<'info, M> {
    pub fn new_singer(account_info: AccountInfo<'info, M>) -> Self {
        Account {
            inner: Signer,
            account_info,
        }
    }
}
