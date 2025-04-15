#![feature(test)]

extern crate test;

use claim::{ClaimConfig, UPDATE_CONFIG};
use mollusk_svm::Mollusk;
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

    let mollusk = Mollusk::new(&program_id, "claim");

    let accounts = vec![
        (manager_id, Account::new(10000000, 0, &Pubkey::new_unique())),
        (
            config_id,
            Account::new_data(
                100000,
                &(
                    ClaimConfig::DISCRIMINATOR,
                    *manager_id.as_array(),
                    0u64,
                    *token_id.as_array(),
                    config_bump,
                ),
                &program_id,
            )
            .unwrap(),
        ),
    ];

    let min_acount_to_claim: u64 = 100000;

    let mut data = UPDATE_CONFIG.to_vec();
    data.extend(min_acount_to_claim.to_le_bytes());

    let account_metas = vec![
        AccountMeta::new(manager_id, true),
        AccountMeta::new(config_id, true),
    ];

    MolluskComputeUnitBencher::new(mollusk)
        .bench((
            "update_config",
            &Instruction {
                program_id,
                accounts: account_metas,
                data,
            },
            &accounts,
        ))
        .must_pass(true)
        .execute();
}
