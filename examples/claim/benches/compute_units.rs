#![feature(test)]

extern crate test;

use std::{cell::RefCell, rc::Rc};

use claim::{Claim, ClaimConfig, CREATE_CONFIG, UPDATE_CLAIM, UPDATE_CONFIG};
use mollusk::{write_results, MyBenchResult};
use mollusk_svm::{
    program::{create_program_account_loader_v3, loader_keys::LOADER_V3},
    Mollusk,
};
use mollusk_svm_bencher::MolluskComputeUnitBencher;
use sol_ez::AccountDataConfig;
use sol_log_parser::{ParsedLog, ParsedStructuredLog, RawLog};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use test::Bencher;

mod mollusk;

#[bench]
fn cu(_b: &mut Bencher) {
    let program_id = Pubkey::new_from_array(claim::ID);
    let manager_id = Pubkey::new_unique();
    let token_id = Pubkey::new_unique();

    let (config_id, config_bump) = Pubkey::find_program_address(&[b"todo"], &program_id);
    let (claim_id, claim_bump) = Pubkey::find_program_address(&[b"claim"], &program_id);

    let system_program_id = Pubkey::new_from_array(pinocchio_system::ID);

    let mut mollusk = Mollusk::new(&program_id, "claim");
    mollusk.add_program(&system_program_id, "solana_system_program", &LOADER_V3);
    mollusk.logger = Some(Rc::new(RefCell::new(Default::default())));

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
        ClaimConfig::DISCRIMINATOR, // discriminator
        *manager_id.as_array(),     // manager auth
        0u64,                       // min amount to claim
        *token_id.as_array(),       // token to claim
        config_bump,                // pda bump
    );

    let update_config_accounts = [
        manager_account.clone(),
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
     *  UPDATE CLAIM
     * *********************************************************************** */

    let claim_data = (
        Claim::DISCRIMINATOR,   // discriminator
        0u64,                   // amount acquired
        *claim_id.as_array(),   // cliam auth
        *manager_id.as_array(), // manager auth
        claim_bump,             // pda bump
    );

    let update_claim_accounts = [
        manager_account,
        (
            config_id,
            Account::new_data(10000, &config_data, &program_id).unwrap(),
        ),
        (
            claim_id,
            Account::new_data(10000, &claim_data, &program_id).unwrap(),
        ),
    ];

    let claim_update = {
        let amount_to_add: u64 = 100000;
        let mut data = UPDATE_CLAIM.to_vec();
        data.extend(amount_to_add.to_le_bytes());

        let account_metas = vec![
            AccountMeta::new(manager_id, true),
            AccountMeta::new(config_id, false),
            AccountMeta::new(claim_id, true),
        ];

        (
            "update_claim",
            &Instruction {
                program_id,
                accounts: account_metas,
                data,
            },
            update_claim_accounts.as_slice(),
        )
    };

    /* *********************************************************************** *
     *  BENCH
     * *********************************************************************** */

    let mut bench = MolluskComputeUnitBencher::new(mollusk)
        .bench(config_create)
        .bench(config_update)
        .bench(claim_update)
        .must_pass(true);

    let results = bench
        .execute_without_write()
        .into_iter()
        .map(|res| {
            eprintln!("{:?}", res.logs);
            let cus_breakdown = res.logs.and_then(|logs| {
                let parsed_logs = logs
                    .iter()
                    .map(|s| ParsedLog::from_raw(&RawLog::parse(s.as_str())))
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?;

                let structured_log = &ParsedStructuredLog::from_parsed_logs(parsed_logs)[0];

                eprintln!("{:?}", structured_log);

                let cpi_cus = structured_log.cpi_logs.iter().fold(0, |acc, log| {
                    acc + log.compute_log.as_ref().map(|cu| cu.consumed).unwrap_or(0)
                });

                let root_cus = structured_log
                    .compute_log
                    .as_ref()
                    .map(|log| log.consumed)
                    .unwrap_or(0)
                    - cpi_cus;

                Some((root_cus, cpi_cus))
            });

            MyBenchResult {
                name: res.name,
                cus_consumed: res.cus_consumed,
                root_cus_consumed: cus_breakdown.as_ref().map(|(cu, _)| *cu),
                cpi_cus_consumed: cus_breakdown.as_ref().map(|(_, cu)| *cu),
            }
        })
        .collect();

    write_results(results);
}
