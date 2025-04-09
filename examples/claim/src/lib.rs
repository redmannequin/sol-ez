#![no_std]
use claim_contract::{
    Claim, ClaimAccounts, ClaimConfig, ClaimContract, ClaimDispatcher, CreateConfigAccounts,
    UpdateClaimAccounts, UpdateConfigAccounts,
};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use pinocchio_log::log;
use sol_ez::{
    Contract,
    account::Account,
    account_info::{AccountRead, Empty, Signed},
};

mod claim_contract;

type EFN =
    for<'a, 'b, 'info> fn(&'a Pubkey, &'info [AccountInfo], &'b [u8]) -> Result<(), ProgramError>;

pub const FN: EFN = ClaimDispatcher::<MyClaim>::dispatch;

pub struct MyClaim;

impl ClaimContract for MyClaim {
    fn create_claim(
        program_id: &Pubkey,
        mut accounts: claim_contract::CreateClaimAccounts,
        amount: u64,
        claim_authority: [u8; 32],
    ) -> Result<(), ProgramError> {
        validate_config_manager(&accounts.claim_config, &accounts.manager_authority)?;
        accounts.claim.init(
            Claim {
                amount_acquired: amount,
                claim_authority,
                manager_authority: *accounts.manager_authority.key(),
            },
            &mut accounts.manager_authority,
            program_id,
        )?;
        log!("Claim Created");
        Ok(())
    }

    fn update_claim(
        _program_id: &Pubkey,
        mut accounts: UpdateClaimAccounts,
        amount: u64,
    ) -> Result<(), ProgramError> {
        validate_config_manager(&accounts.claim_config, &accounts.manager_authority)?;
        validate_claim_manager(&accounts.claim, &accounts.manager_authority)?;
        accounts.claim.as_ref_mut().amount_acquired += amount;
        accounts.claim.apply()?;
        log!("Claim Updated");
        Ok(())
    }

    fn claim(_program_id: &Pubkey, accounts: ClaimAccounts) -> Result<(), ProgramError> {
        validate_claim(
            &accounts.claim,
            &accounts.claim_config,
            &accounts.claim_authority,
            &accounts.manager_authority,
        )?;

        // TODO: create CPI call to token transfer to claim tokens
        // use accounts.user_authority

        log!("Claim Claimed");
        Ok(())
    }

    fn create_config(
        program_id: &Pubkey,
        mut accounts: CreateConfigAccounts,
        token_id: Pubkey,
    ) -> Result<(), ProgramError> {
        accounts.claim_config.init(
            ClaimConfig {
                manager_authority: *accounts.manager_authority.key(),
                min_amount_to_claim: 0,
                token_id,
            },
            &mut accounts.manager_authority,
            program_id,
        )?;
        log!("Claim Config Created");
        Ok(())
    }

    fn update_config(
        _program_id: &Pubkey,
        mut accounts: UpdateConfigAccounts,
        amount: u64,
    ) -> Result<(), ProgramError> {
        validate_config_manager(&accounts.claim_config, &accounts.manager_authority)?;
        accounts.claim_config.as_ref_mut().min_amount_to_claim = amount;
        accounts.claim_config.apply()?;
        log!("Claim Config Updated");
        Ok(())
    }
}

pub fn validate_config_manager<S>(
    config: &Account<ClaimConfig, impl AccountRead, S>,
    manager: &Account<Empty, impl AccountRead, Signed>,
) -> Result<(), ProgramError> {
    if config.as_ref().manager_authority != *manager.key() {
        return Err(ProgramError::IllegalOwner);
    }
    Ok(())
}

pub fn validate_claim_manager<S>(
    claim: &Account<Claim, impl AccountRead, S>,
    manager: &Account<Empty, impl AccountRead, Signed>,
) -> Result<(), ProgramError> {
    if claim.as_ref().manager_authority != *manager.key() {
        return Err(ProgramError::IllegalOwner);
    }
    Ok(())
}

pub fn validate_claim<S0, S1, S2, S3>(
    claim: &Account<Claim, impl AccountRead, S0>,
    claim_config: &Account<ClaimConfig, impl AccountRead, S1>,
    claim_auth: &Account<Empty, impl AccountRead, S2>,
    manager: &Account<Empty, impl AccountRead, S3>,
) -> Result<(), ProgramError> {
    if *claim_auth.key() != claim.as_ref().claim_authority {
        return Err(ProgramError::IllegalOwner);
    }
    if claim_config.as_ref().min_amount_to_claim > claim.as_ref().amount_acquired {
        return Err(ProgramError::Custom(0));
    }
    if claim.as_ref().manager_authority != *manager.key() {
        return Err(ProgramError::IllegalOwner);
    }
    if claim_config.as_ref().manager_authority != *manager.key() {
        return Err(ProgramError::IllegalOwner);
    }
    Ok(())
}
