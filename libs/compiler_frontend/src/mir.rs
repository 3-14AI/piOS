extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum MirOp {
    Const(i64),
    Load(usize), // Load from local variable
    Add(usize, usize),
    Sub(usize, usize),
    Mul(usize, usize),
    Call(String, Vec<usize>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirInst {
    Assign { dest: usize, op: MirOp },
    Return { src: usize },
    Nop,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirFunction {
    pub name: String,
    pub param_count: usize,
    pub locals_count: usize, // Includes params
    pub insts: Vec<MirInst>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirModule {
    pub functions: Vec<MirFunction>,
}
