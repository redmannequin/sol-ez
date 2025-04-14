use borsh::BorshDeserialize;
use claim::{ClaimConfig, ClaimDispatcher, MyClaim, CREATE_CONFIG, UPDATE_CONFIG};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    runtime::mock::{invoke, MockAccount, MockProgramAccount, MOCK_RUNTIME},
    ProgramResult,
};
use sol_ez::{AccountData, AccountDataConfig, Contract, InstructionData};

fn system_program(
    _program_id: &Pubkey,
    account_infos: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let ix_data = InstructionData::new(data)?;
    match ix_data.ix {
        // create account
        [0, 0, 0, 0] => {
            let (lamports, space, owner) = ix_data.deserialize_data()?;
            let lamports = u64::from_le_bytes(lamports);
            let space = u64::from_le_bytes(space);
            unsafe {
                *account_infos[1].borrow_mut_lamports_unchecked() = lamports;
                account_infos[1].realloc(space as usize, false)?;
                account_infos[1].assign(&owner);
            }
            Ok(())
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

#[test]
fn create_config() {
    let program_id = [250; 32];
    let token_id = [150; 32];
    let manager_id = [50; 32];

    let (config_id, config_id_bump) = pubkey::find_program_address(&[b"todo"], &program_id);

    let (manager, claim_config) = MOCK_RUNTIME.with_borrow_mut(|rt| {
        rt.register_program_account(
            "my_claim",
            MockProgramAccount::new_program(
                false,
                false,
                program_id,
                pinocchio_system::ID,
                0,
                ClaimDispatcher::<MyClaim>::dispatch,
            ),
        );

        rt.register_program_account(
            "system_program",
            MockProgramAccount::new_program(
                false,
                false,
                pinocchio_system::ID,
                pinocchio_system::ID,
                0,
                system_program,
            ),
        );

        rt.register_data_account(
            "manager_account",
            MockAccount::new_data_account(true, true, manager_id, manager_id, 0, vec![]),
        );
        rt.register_data_account(
            "claim_config",
            MockAccount::new_data_account(false, true, config_id, program_id, 0, vec![]),
        );

        (
            rt.get_data_account(&manager_id).unwrap(),
            rt.get_data_account(&config_id).unwrap(),
        )
    });

    let mut data = CREATE_CONFIG.to_vec();
    data.push(config_id_bump);
    data.extend(token_id);

    let accounts = [
        AccountMeta::writable_signer(&manager_id),
        AccountMeta::writable(&config_id),
    ];

    invoke(
        &Instruction {
            program_id: &program_id,
            data: &data,
            accounts: &accounts,
        },
        &[&manager, &claim_config],
    );

    let config_data = unsafe { claim_config.borrow_data_unchecked() };

    let (discriminator, data) = config_data.split_at(ClaimConfig::DISCRIMINATOR.len());
    println!("{:?}", config_data);
    assert_eq!(discriminator, ClaimConfig::DISCRIMINATOR);
    assert_eq!(data.len(), ClaimConfig::DATA_SIZE);

    let config = <ClaimConfig as BorshDeserialize>::try_from_slice(data)
        .expect("failed to deserialize config");

    assert_eq!(config.manager_authority, manager_id);
    assert_eq!(config.min_amount_to_claim, 0);
    assert_eq!(config.token_id, token_id);
    assert_eq!(config.bump, config_id_bump);
}

#[test]
fn update_config() {
    let program_id = [250; 32];
    let token_id = [150; 32];
    let manager_id = [50; 32];

    let (config_id, config_id_bump) = pubkey::find_program_address(&[b"todo"], &program_id);

    let (manager, claim_config) = MOCK_RUNTIME.with_borrow_mut(|rt| {
        rt.register_program_account(
            "my_claim",
            MockProgramAccount::new_program(
                false,
                false,
                program_id,
                pinocchio_system::ID,
                0,
                ClaimDispatcher::<MyClaim>::dispatch,
            ),
        );

        rt.register_program_account(
            "system_program",
            MockProgramAccount::new_program(
                false,
                false,
                pinocchio_system::ID,
                pinocchio_system::ID,
                0,
                system_program,
            ),
        );

        rt.register_data_account(
            "manager_account",
            MockAccount::new_data_account(true, true, manager_id, manager_id, 0, vec![]),
        );

        rt.register_data_account(
            "claim_config",
            MockAccount::new_data_account(
                false,
                true,
                config_id,
                program_id,
                0,
                AccountData::new(ClaimConfig {
                    manager_authority: manager_id,
                    min_amount_to_claim: 0,
                    token_id: token_id,
                    bump: config_id_bump,
                })
                .to_bytes()
                .unwrap(),
            ),
        );

        (
            rt.get_data_account(&manager_id).unwrap(),
            rt.get_data_account(&config_id).unwrap(),
        )
    });

    {
        let config_data = unsafe { claim_config.borrow_data_unchecked() };
        println!("{:?}", config_data.len());
        let (discriminator, data) = config_data.split_at(ClaimConfig::DISCRIMINATOR.len());
        println!("{:?}", discriminator);
        // println!("{:?}", data);
        assert_eq!(discriminator, ClaimConfig::DISCRIMINATOR);
        assert_eq!(data.len(), ClaimConfig::DATA_SIZE);

        let config = <ClaimConfig as BorshDeserialize>::try_from_slice(data)
            .expect("failed to deserialize config");

        assert_eq!(config.manager_authority, manager_id);
        assert_eq!(config.min_amount_to_claim, 0);
        assert_eq!(config.token_id, token_id);
        assert_eq!(config.bump, config_id_bump);
    }

    let min_acount_to_claim: u64 = 100000;

    let mut data = UPDATE_CONFIG.to_vec();
    data.extend(min_acount_to_claim.to_le_bytes());

    let accounts = [
        AccountMeta::readonly_signer(&manager_id),
        AccountMeta::writable(&config_id),
    ];

    invoke(
        &Instruction {
            program_id: &program_id,
            data: &data,
            accounts: &accounts,
        },
        &[&manager, &claim_config],
    );

    let config_data = unsafe { claim_config.borrow_data_unchecked() };

    let (discriminator, data) = config_data.split_at(ClaimConfig::DISCRIMINATOR.len());
    assert_eq!(discriminator, ClaimConfig::DISCRIMINATOR);
    assert_eq!(data.len(), ClaimConfig::DATA_SIZE);

    let config = <ClaimConfig as BorshDeserialize>::try_from_slice(data)
        .expect("failed to deserialize config");

    assert_eq!(config.manager_authority, manager_id);
    assert_eq!(config.min_amount_to_claim, min_acount_to_claim);
    assert_eq!(config.token_id, token_id);
    assert_eq!(config.bump, config_id_bump);
}
