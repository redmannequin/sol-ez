use core::marker::PhantomData;
use borsh::{BorshDeserialize, BorshSerialize};
use sol_ez::{account::*, account_info::*, AccountData, AccountDataConfig, DataSize};
use pinocchio::{program_error::ProgramError, pubkey::Pubkey};
#[derive(BorshSerialize, BorshDeserialize, AccountDataConfig)]
#[account_data(hash(seed = "claim|account|claim", size = 4usize))]
pub struct Claim {
    pub amount_acquired: u64,
    pub claim_authority: [u8; 32],
    pub manager_authority: [u8; 32],
    pub bump: u8,
}
#[derive(BorshSerialize, BorshDeserialize, AccountDataConfig)]
#[account_data(hash(seed = "claim|account|claim_config", size = 4usize))]
pub struct ClaimConfig {
    pub manager_authority: [u8; 32],
    pub min_amount_to_claim: u64,
    pub token_id: [u8; 32],
    pub bump: u8,
}
pub struct CreateClaimAccounts<'info> {
    pub manager_authority: AccountWritableSigned<'info, Empty>,
    pub claim_config: AccountReadOnly<'info, AccountData<4usize, ClaimConfig>>,
    pub claim: Account<'info, PhantomData<AccountData<4usize, Claim>>, Init, Unsigned>,
}
impl<'info> CreateClaimAccounts<'info> {
    pub fn load(
        accounts: &'info [pinocchio::account_info::AccountInfo],
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            manager_authority: AccountBuilder::new(
                    accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .mutable()?
                .signed()?
                .build()?,
            claim_config: AccountBuilder::new(
                    accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .set_payload()
                .build()?,
            claim: Account::new_init(
                AccountInfo::new_init(
                    accounts.get(2usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )?,
            ),
        })
    }
}
pub struct UpdateClaimAccounts<'info> {
    pub manager_authority: AccountReadOnlySigned<'info, Empty>,
    pub claim_config: AccountReadOnly<'info, AccountData<4usize, ClaimConfig>>,
    pub claim: AccountWritable<'info, AccountData<4usize, Claim>>,
}
impl<'info> UpdateClaimAccounts<'info> {
    pub fn load(
        accounts: &'info [pinocchio::account_info::AccountInfo],
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            manager_authority: AccountBuilder::new(
                    accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .signed()?
                .build()?,
            claim_config: AccountBuilder::new(
                    accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .set_payload()
                .build()?,
            claim: AccountBuilder::new(
                    accounts.get(2usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .set_payload()
                .mutable()?
                .build()?,
        })
    }
}
pub struct ClaimAccounts<'info> {
    pub claim_authority: AccountWritable<'info, Empty>,
    pub claim_config: AccountReadOnly<'info, AccountData<4usize, ClaimConfig>>,
    pub claim: AccountWritable<'info, AccountData<4usize, Claim>>,
    pub manager_authority: AccountReadOnly<'info, Empty>,
    pub user_authority: AccountReadOnlySigned<'info, Empty>,
}
impl<'info> ClaimAccounts<'info> {
    pub fn load(
        accounts: &'info [pinocchio::account_info::AccountInfo],
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            claim_authority: AccountBuilder::new(
                    accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .mutable()?
                .build()?,
            claim_config: AccountBuilder::new(
                    accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .set_payload()
                .build()?,
            claim: AccountBuilder::new(
                    accounts.get(2usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .set_payload()
                .mutable()?
                .build()?,
            manager_authority: AccountBuilder::new(
                    accounts.get(3usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .build()?,
            user_authority: AccountBuilder::new(
                    accounts.get(4usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .signed()?
                .build()?,
        })
    }
}
pub struct CreateConfigAccounts<'info> {
    pub manager_authority: AccountWritableSigned<'info, Empty>,
    pub claim_config: Account<
        'info,
        PhantomData<AccountData<4usize, ClaimConfig>>,
        Init,
        Unsigned,
    >,
}
impl<'info> CreateConfigAccounts<'info> {
    pub fn load(
        accounts: &'info [pinocchio::account_info::AccountInfo],
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            manager_authority: AccountBuilder::new(
                    accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .mutable()?
                .signed()?
                .build()?,
            claim_config: Account::new_init(
                AccountInfo::new_init(
                    accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )?,
            ),
        })
    }
}
pub struct UpdateConfigAccounts<'info> {
    pub manager_authority: AccountReadOnlySigned<'info, Empty>,
    pub claim_config: AccountWritable<'info, AccountData<4usize, ClaimConfig>>,
}
impl<'info> UpdateConfigAccounts<'info> {
    pub fn load(
        accounts: &'info [pinocchio::account_info::AccountInfo],
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            manager_authority: AccountBuilder::new(
                    accounts.get(0usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .signed()?
                .build()?,
            claim_config: AccountBuilder::new(
                    accounts.get(1usize).ok_or(ProgramError::NotEnoughAccountKeys)?,
                )
                .set_payload()
                .mutable()?
                .build()?,
        })
    }
}
pub trait ClaimContract {
    fn create_claim(
        program_id: &Pubkey,
        accounts: CreateClaimAccounts,
        amount: u64,
        claim_authority: [u8; 32],
    ) -> Result<(), ProgramError>;
    fn update_claim(
        program_id: &Pubkey,
        accounts: UpdateClaimAccounts,
        amount_to_add: u64,
    ) -> Result<(), ProgramError>;
    fn claim(program_id: &Pubkey, accounts: ClaimAccounts) -> Result<(), ProgramError>;
    fn create_config(
        program_id: &Pubkey,
        accounts: CreateConfigAccounts,
        config_bump: u8,
        token_id: [u8; 32],
    ) -> Result<(), ProgramError>;
    fn update_config(
        program_id: &Pubkey,
        accounts: UpdateConfigAccounts,
        min_amount_to_claim: u64,
    ) -> Result<(), ProgramError>;
}
pub struct ClaimDispatcher<T> {
    inner: PhantomData<T>,
}
pub const CREATE_CLAIM: [u8; 4usize] = [109u8, 226u8, 7u8, 61u8];
pub const UPDATE_CLAIM: [u8; 4usize] = [3u8, 255u8, 115u8, 253u8];
pub const CLAIM: [u8; 4usize] = [29u8, 37u8, 118u8, 180u8];
pub const CREATE_CONFIG: [u8; 4usize] = [78u8, 77u8, 163u8, 125u8];
pub const UPDATE_CONFIG: [u8; 4usize] = [88u8, 6u8, 10u8, 242u8];
impl<T> sol_ez::Contract for ClaimDispatcher<T>
where
    T: ClaimContract,
{
    fn dispatch<'info>(
        program_id: &Pubkey,
        accounts: &'info [pinocchio::account_info::AccountInfo],
        payload: &[u8],
    ) -> Result<(), ProgramError> {
        let ix_data = sol_ez::InstructionData::new(payload)?;
        match ix_data.ix {
            &CREATE_CLAIM => {
                let accounts = CreateClaimAccounts::load(accounts)?;
                let (amount, claim_authority) = ix_data.deserialize_data()?;
                T::create_claim(program_id, accounts, amount, claim_authority)
            }
            &UPDATE_CLAIM => {
                let accounts = UpdateClaimAccounts::load(accounts)?;
                let amount_to_add = ix_data.deserialize_data()?;
                T::update_claim(program_id, accounts, amount_to_add)
            }
            &CLAIM => {
                let accounts = ClaimAccounts::load(accounts)?;
                T::claim(program_id, accounts)
            }
            &CREATE_CONFIG => {
                let accounts = CreateConfigAccounts::load(accounts)?;
                let (config_bump, token_id) = ix_data.deserialize_data()?;
                T::create_config(program_id, accounts, config_bump, token_id)
            }
            &UPDATE_CONFIG => {
                let accounts = UpdateConfigAccounts::load(accounts)?;
                let min_amount_to_claim = ix_data.deserialize_data()?;
                T::update_config(program_id, accounts, min_amount_to_claim)
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
