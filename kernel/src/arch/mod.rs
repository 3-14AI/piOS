#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
pub mod aarch64;
#[cfg(feature = "verus")]
pub mod riscv64;
#[cfg(feature = "verus")]
pub mod x86_64;

#[cfg(not(feature = "verus"))]
pub mod aarch64;
#[cfg(not(feature = "verus"))]
pub mod riscv64;
#[cfg(not(feature = "verus"))]
pub mod x86_64;

#[cfg(feature = "verus")]
verus! {
    pub enum Architecture {
        X86_64,
        Aarch64,
        Riscv64,
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug, PartialEq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Riscv64,
}
