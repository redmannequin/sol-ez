use pinocchio::program_error::ProgramError;

use crate::account_info::AccountInfo;

use super::Account;

pub struct Signer;

pub type SignerAccount<'info, M> = Account<'info, Signer, M>;

impl<'info, M> SignerAccount<'info, M> {
    pub fn new_singer(account_info: AccountInfo<'info, M>) -> Result<Self, ProgramError> {
        if !account_info.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Account {
            inner: Signer,
            account_info,
        })
    }
}
