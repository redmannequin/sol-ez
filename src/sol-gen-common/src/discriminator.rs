use sha2::{Digest, Sha256};

use crate::my_idl::{Account, Instruction};

pub type Discriminator = Vec<u8>;

pub trait DiscriminatorGen {
    type Seed;

    fn instruction_seed(program_name: &str, ix: &Instruction) -> Self::Seed;
    fn account_seed(program_name: &str, account: &Account) -> Self::Seed;
    fn discriminator(seed: Self::Seed, size: usize) -> Discriminator;

    fn from_instruction(program_name: &str, ix: &Instruction, size: usize) -> Discriminator {
        let seed = Self::instruction_seed(program_name, ix);
        Self::discriminator(seed, size)
    }

    fn from_account(program_name: &str, account: &Account, size: usize) -> Discriminator {
        let seed = Self::account_seed(program_name, account);
        Self::discriminator(seed, size)
    }
}

pub struct HashDiscriminatorGen;

impl DiscriminatorGen for HashDiscriminatorGen {
    type Seed = String;

    fn instruction_seed(program_name: &str, ix: &Instruction) -> String {
        format!("{}|ix|{}", program_name, ix.name)
    }

    fn account_seed(program_name: &str, account: &Account) -> String {
        format!("{}|account|{}", program_name, account.name)
    }

    fn discriminator(seed: String, size: usize) -> Discriminator {
        let hash = Sha256::digest(seed);
        let (bytes, _) = hash.split_at(size);
        bytes.to_vec()
    }
}

pub struct IndexDiscriminatorGen;

impl DiscriminatorGen for IndexDiscriminatorGen {
    type Seed = u8;

    fn instruction_seed(_program_name: &str, ix: &Instruction) -> Self::Seed {
        ix.id
    }

    fn account_seed(_program_name: &str, account: &Account) -> Self::Seed {
        account.id
    }

    fn discriminator(seed: Self::Seed, _size: usize) -> Discriminator {
        vec![seed]
    }
}
