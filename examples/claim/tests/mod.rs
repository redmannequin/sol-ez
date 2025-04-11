use borsh::BorshDeserialize;
use claim::{ClaimConfig, CREATE_CONFIG};
use pinocchio::{
    instruction::{AccountMeta, Instruction},
    runtime::mock::{invoke, MockAccount, MockProgramAccount, MOCK_RUNTIME},
};
use sol_ez::AccountData;

#[test]
fn create_config() {
    let program_id = [250; 32];
    let token_id = [254; 32];
    let config_id = [253; 32];
    let manager_id = [252; 32];

    let (manager, claim_config) = {
        let mut rt = MOCK_RUNTIME.lock().unwrap();

        rt.register_program_account(
            "my_claim",
            MockProgramAccount::new_program(
                false,
                false,
                program_id,
                pinocchio_system::ID,
                0,
                claim::FN,
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
                |_, accounts, data| {
                    let space = u64::from_le_bytes([
                        data[12], data[13], data[14], data[15], data[16], data[17], data[18],
                        data[19],
                    ]);
                    accounts[1].realloc(space as usize, false)?;
                    Ok(())
                },
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
    };

    let mut data = CREATE_CONFIG.to_vec();
    data.extend(token_id);

    let accounts = [
        AccountMeta {
            pubkey: &manager_id,
            is_writable: true,
            is_signer: true,
        },
        AccountMeta {
            pubkey: &config_id,
            is_writable: true,
            is_signer: false,
        },
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

    let (discriminator, mut data) = config_data.split_at(ClaimConfig::DISCRIMINATOR.len());
    assert_eq!(discriminator, ClaimConfig::DISCRIMINATOR);

    let config = <ClaimConfig as BorshDeserialize>::deserialize(&mut data)
        .expect("failed to deserialize config");

    assert_eq!(config.manager_authority, manager_id);
    assert_eq!(config.min_amount_to_claim, 0);
    assert_eq!(config.token_id, token_id);
}
