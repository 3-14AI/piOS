#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct Riscv64Arch {
        pub initialized: bool,
    }

    impl Riscv64Arch {
        pub fn new() -> (d: Self)
            ensures d.initialized == true
        {
            Riscv64Arch { initialized: true }
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct Riscv64Arch {
    pub initialized: bool,
}

#[cfg(not(feature = "verus"))]
impl Riscv64Arch {
    pub fn new() -> Self {
        Riscv64Arch { initialized: true }
    }
}

#[cfg(not(feature = "verus"))]
impl Default for Riscv64Arch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_riscv64_arch() {
        let arch = Riscv64Arch::new();
        assert_eq!(arch.initialized, true);
    }
}
