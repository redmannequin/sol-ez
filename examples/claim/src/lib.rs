#![no_std]

use pinocchio_pubkey::declare_id;

#[cfg(not(feature = "bpf"))]
pub use crate::{claim::MyClaim, claim::FN, claim_contract::*};

declare_id!("D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns");

#[cfg(feature = "bpf")]
mod entrypoint {
    use crate::claim::FN;
    use pinocchio::entrypoint;
    entrypoint!(FN);
}

mod claim;
mod claim_contract;
