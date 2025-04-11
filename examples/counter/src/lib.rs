#![no_std]

#[cfg(not(feature = "bpf"))]
pub use crate::{counter::MyCounter, counter::FN, counter_contract::*};

#[cfg(feature = "bpf")]
mod entrypoint {
    use crate::counter::FN;
    use pinocchio::entrypoint;
    entrypoint!(FN);
}

// generated code
mod counter_contract;
// program impl
mod counter;
