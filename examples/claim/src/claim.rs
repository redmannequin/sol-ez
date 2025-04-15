use crate::claim_contract::{
    Claim, ClaimAccounts, ClaimConfig, ClaimContract, ClaimDispatcher, CreateClaimAccounts,
    CreateConfigAccounts, UpdateClaimAccounts, UpdateConfigAccounts,
};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use pinocchio_log::log;
use sol_ez::{
    account::{AccountReadOnly, AccountSigned, AccountWritable},
    account_info::{AccountRead, Empty},
    Contract,
};

pub const FN: fn(&Pubkey, &[AccountInfo], &[u8]) -> Result<(), ProgramError> =
    ClaimDispatcher::<MyClaim>::dispatch;

pub struct MyClaim;

impl ClaimContract for MyClaim {
    fn create_claim(
        program_id: &Pubkey,
        mut accounts: CreateClaimAccounts,
        amount: u64,
        claim_authority: [u8; 32],
    ) -> Result<(), ProgramError> {
        validate_config_manager(accounts.claim_config.as_ref(), &accounts.manager_authority)?;
        accounts.claim.init(
            Claim {
                amount_acquired: amount,
                claim_authority,
                manager_authority: *accounts.manager_authority.key(),
                bump: 0,
            },
            0,
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
        validate_config_manager(accounts.claim_config.as_ref(), &accounts.manager_authority)?;
        validate_claim_manager(accounts.claim.as_ref(), &accounts.manager_authority)?;
        accounts.claim.as_ref_mut().amount_acquired += amount;
        accounts.claim.apply()?;
        log!("Claim Updated");
        Ok(())
    }

    fn claim(_program_id: &Pubkey, accounts: ClaimAccounts) -> Result<(), ProgramError> {
        validate_claim(
            accounts.claim.as_ref(),
            accounts.claim_config.as_ref(),
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
        config_bump: u8,
        token_id: Pubkey,
    ) -> Result<(), ProgramError> {
        log!("In Config Created");
        accounts.claim_config.init(
            ClaimConfig {
                manager_authority: *accounts.manager_authority.key(),
                min_amount_to_claim: 0,
                token_id,
                bump: config_bump,
            },
            config_bump,
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
        validate_config_manager(accounts.claim_config.as_ref(), &accounts.manager_authority)?;
        accounts.claim_config.as_ref_mut().min_amount_to_claim = amount;
        accounts.claim_config.apply()?;
        Ok(())
    }
}

fn validate_config_manager(
    config: &ClaimConfig,
    manager: &AccountSigned<Empty, impl AccountRead>,
) -> Result<(), ProgramError> {
    if config.manager_authority != *manager.key() {
        return Err(ProgramError::IllegalOwner);
    }
    Ok(())
}

fn validate_claim_manager(
    claim: &Claim,
    manager: &AccountSigned<Empty, impl AccountRead>,
) -> Result<(), ProgramError> {
    if claim.manager_authority != *manager.key() {
        return Err(ProgramError::IllegalOwner);
    }
    Ok(())
}

fn validate_claim(
    claim: &Claim,
    claim_config: &ClaimConfig,
    claim_auth: &AccountWritable<Empty>,
    manager: &AccountReadOnly<Empty>,
) -> Result<(), ProgramError> {
    if *claim_auth.key() != claim.claim_authority {
        return Err(ProgramError::IllegalOwner);
    }
    if claim_config.min_amount_to_claim > claim.amount_acquired {
        return Err(ProgramError::Custom(0));
    }
    if claim.manager_authority != *manager.key() {
        return Err(ProgramError::IllegalOwner);
    }
    if claim_config.manager_authority != *manager.key() {
        return Err(ProgramError::IllegalOwner);
    }
    Ok(())
}
