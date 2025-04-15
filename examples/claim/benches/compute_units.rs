#![feature(test)]

extern crate test;

use claim::{ClaimConfig, CREATE_CONFIG, UPDATE_CONFIG};
use mollusk_svm::{
    program::{create_program_account_loader_v3, loader_keys::LOADER_V3},
    Mollusk,
};
use mollusk_svm_bencher::MolluskComputeUnitBencher;
use sol_ez::AccountDataConfig;
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use test::Bencher;

#[bench]
fn cu(_b: &mut Bencher) {
    let program_id = Pubkey::new_from_array(claim::ID);
    let manager_id = Pubkey::new_unique();
    let token_id = Pubkey::new_unique();

    let (config_id, config_bump) = Pubkey::find_program_address(&[b"todo"], &program_id);

    let mut mollusk = Mollusk::new(&program_id, "claim");
    let system_program_id = Pubkey::new_from_array(pinocchio_system::ID);
    mollusk.add_program(&system_program_id, "solana_system_program", &LOADER_V3);

    let manager_account = (manager_id, Account::new(10000000, 0, &system_program_id));

    /* *********************************************************************** *
     *  CREATE CONFIG
     * *********************************************************************** */

    let create_config_accounts = [
        manager_account.clone(),
        (config_id, Account::default()),
        (
            system_program_id,
            create_program_account_loader_v3(&system_program_id),
        ),
    ];
    let config_create = {
        let mut data = CREATE_CONFIG.to_vec();
        data.push(config_bump);
        data.extend(token_id.as_array());

        let account_metas = vec![
            AccountMeta::new(manager_id, true),
            AccountMeta::new(config_id, false),
            AccountMeta::new_readonly(system_program_id, false),
        ];

        (
            "create_config",
            &Instruction {
                program_id,
                accounts: account_metas,
                data,
            },
            create_config_accounts.as_slice(),
        )
    };

    /* *********************************************************************** *
     *  UPDATE CONFIG
     * *********************************************************************** */

    let config_data = (
        ClaimConfig::DISCRIMINATOR,
        *manager_id.as_array(),
        0u64,
        *token_id.as_array(),
        config_bump,
    );

    let update_config_accounts = [
        manager_account,
        (
            config_id,
            Account::new_data(10000, &config_data, &program_id).unwrap(),
        ),
    ];
    let config_update = {
        let min_acount_to_claim: u64 = 100000;
        let mut data = UPDATE_CONFIG.to_vec();
        data.extend(min_acount_to_claim.to_le_bytes());

        let account_metas = vec![
            AccountMeta::new(manager_id, true),
            AccountMeta::new(config_id, false),
        ];

        (
            "update_config",
            &Instruction {
                program_id,
                accounts: account_metas,
                data,
            },
            update_config_accounts.as_slice(),
        )
    };

    /* *********************************************************************** *
     *  BENCH
     * *********************************************************************** */

    MolluskComputeUnitBencher::new(mollusk)
        .bench(config_create)
        .bench(config_update)
        .must_pass(true)
        .execute();
}
