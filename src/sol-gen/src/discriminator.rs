use sha2::{Digest, Sha256};

use crate::my_idl::{Account, Instruction};

pub type Discriminator = Vec<u8>;

pub trait DiscriminatorGen {
    fn from_instruction(program_name: &str, ix: &Instruction, size: usize) -> Discriminator;
    fn from_account(program_name: &str, account: &Account, size: usize) -> Discriminator;
}

pub struct HashDiscriminatorGen;

impl DiscriminatorGen for HashDiscriminatorGen {
    fn from_instruction(program_name: &str, ix: &Instruction, size: usize) -> Discriminator {
        assert_ne!(size, 0);
        let hash = Sha256::digest(format!("{}|ix|{}", program_name, ix.name));
        let (bytes, _) = hash.split_at(size);
        bytes.to_vec()
    }

    fn from_account(program_name: &str, account: &Account, size: usize) -> Discriminator {
        if size == 0 {
            return vec![];
        }
        let hash = Sha256::digest(format!("{}|account|{}", program_name, account.name));
        let (bytes, _) = hash.split_at(size);
        bytes.to_vec()
    }
}

pub struct IndexDiscriminatorGen;

impl DiscriminatorGen for IndexDiscriminatorGen {
    fn from_instruction(_program_name: &str, ix: &Instruction, _size: usize) -> Discriminator {
        vec![ix.id]
    }

    fn from_account(_program_name: &str, account: &Account, size: usize) -> Discriminator {
        if size == 0 {
            return vec![];
        }
        vec![account.id]
    }
}
