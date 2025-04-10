use std::sync::Arc;

use borsh::BorshDeserialize;
use claim::{
    claim_contract::{ClaimConfig, ClaimDispatcher, CREATE_CONFIG},
    MyClaim,
};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    log::sol_log,
    pubkey::Pubkey,
    runtime::mock::{invoke, MockAccount, MockData, MOCK_RUNTIME},
    ProgramResult,
};
use sol_ez::{AccountData, Contract};

pub struct Account {
    info: AccountInfo,
    _data: Vec<u8>,
}

impl Account {
    pub fn init(
        is_signer: bool,
        is_writable: bool,
        is_program: bool,
        key: Pubkey,
        owener: Pubkey,
        lamports: u64,
        data: Vec<u8>,
    ) -> Self {
        let mut raw_data = vec![0, is_signer as u8, is_writable as u8, is_program as u8];
        raw_data.extend((data.len() as u32).to_ne_bytes());
        raw_data.extend(key);
        raw_data.extend(owener);
        raw_data.extend(lamports.to_ne_bytes());
        raw_data.extend((data.len() as u64).to_ne_bytes());
        raw_data.extend(data);

        let raw_data_ptr = raw_data.as_mut_ptr();

        #[repr(C)]
        struct Account {
            borrow_state: u8,
            is_signer: u8,
            is_writable: u8,
            executable: u8,
            original_data_len: u32,
            key: Pubkey,
            owner: Pubkey,
            lamports: u64,
            data_len: u64,
        }

        #[repr(C)]
        struct Dummy {
            _inner: *mut Account,
        }

        let account_info = Dummy {
            _inner: raw_data_ptr as *mut Account,
        };

        Self {
            info: unsafe { std::mem::transmute(account_info) },
            _data: raw_data,
        }
    }
}

trait Wrap: Sized + Fn(&Pubkey, &[AccountInfo], &[u8]) -> ProgramResult {
    fn wrap(
        self,
        before: impl Fn(&Pubkey, &[AccountInfo], &[u8]),
        after: impl Fn(&ProgramResult),
    ) -> impl Fn(&Pubkey, &[AccountInfo], &[u8]) -> ProgramResult {
        move |key, accounts, payload| {
            before(key, accounts, payload);
            let res = self(key, accounts, payload);
            after(&res);
            res
        }
    }
}

impl<T> Wrap for T where T: Fn(&Pubkey, &[AccountInfo], &[u8]) -> ProgramResult {}

#[test]
fn create_config() {
    let program_id = [250; 32];
    let token_id = [254; 32];
    let config_id = [253; 32];
    let manager_id = [252; 32];

    MOCK_RUNTIME.lock().unwrap().add_program(MockAccount {
        key: program_id,
        lamports: 0,
        data: MockData::Program(Arc::new(ClaimDispatcher::<MyClaim>::dispatch.wrap(
            |pid, _, _| {
                pinocchio::log::sol_log("Enter MyClaim");
                pinocchio::log::sol_log(&format!("pid: {:?}", &pid[0..5]));
            },
            |res| {
                let _ = res
                    .as_ref()
                    .map_err(|err| sol_log(&format!("Err: {:?}", err)));
                pinocchio::log::sol_log("Exit MyClaim");
            },
        ))),
    });

    // Add system program to mock -- don't verify anything just make it work!
    MOCK_RUNTIME.lock().unwrap().add_program(MockAccount {
        key: pinocchio_system::ID,
        lamports: 0,
        data: MockData::Program(Arc::new(|_, accounts, data| {
            let space = u64::from_le_bytes([
                data[12], data[13], data[14], data[15], data[16], data[17], data[18], data[19],
            ]);
            accounts[1].realloc(space as usize, false)?;
            Ok(())
        })),
    });

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

    let manager_account = Account::init(true, true, false, manager_id, manager_id, 0, vec![]);
    let config_account = Account::init(false, true, false, config_id, program_id, 0, vec![]);
    let account_infos = [&manager_account.info, &config_account.info];

    invoke(
        &Instruction {
            program_id: &program_id,
            data: &data,
            accounts: &accounts,
        },
        &account_infos,
    );

    let config_data = unsafe { config_account.info.borrow_data_unchecked() };

    let (discriminator, mut data) = config_data.split_at(ClaimConfig::DISCRIMINATOR.len());
    assert_eq!(discriminator, ClaimConfig::DISCRIMINATOR);

    let config = <ClaimConfig as BorshDeserialize>::deserialize(&mut data)
        .expect("failed to deserialize config");

    assert_eq!(config.manager_authority, manager_id);
    assert_eq!(config.min_amount_to_claim, 0);
    assert_eq!(config.token_id, token_id);
}
