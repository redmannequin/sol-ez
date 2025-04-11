#![no_std]

#[cfg(not(feature = "bpf"))]
pub use crate::{claim::MyClaim, claim::FN, claim_contract::*};

#[cfg(feature = "bpf")]
mod entrypoint {
    use crate::claim::FN;
    use pinocchio::entrypoint;
    entrypoint!(FN);
}

mod claim;
mod claim_contract;
