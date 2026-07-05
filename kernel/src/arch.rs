#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub enum Architecture {
        X86_64,
        AArch64,
        RiscV,
    }

    pub struct ArchSupport {
        pub arch: Architecture,
    }

    impl ArchSupport {
        pub fn new(arch: Architecture) -> (s: Self)
            ensures s.arch == arch
        {
            ArchSupport { arch }
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Architecture {
    X86_64,
    AArch64,
    RiscV,
}

#[cfg(not(feature = "verus"))]
pub struct ArchSupport {
    pub arch: Architecture,
}

#[cfg(not(feature = "verus"))]
impl ArchSupport {
    pub fn new(arch: Architecture) -> Self {
        ArchSupport { arch }
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_support() {
        let arch_x86 = ArchSupport::new(Architecture::X86_64);
        assert_eq!(arch_x86.arch, Architecture::X86_64);

        let arch_arm = ArchSupport::new(Architecture::AArch64);
        assert_eq!(arch_arm.arch, Architecture::AArch64);

        let arch_riscv = ArchSupport::new(Architecture::RiscV);
        assert_eq!(arch_riscv.arch, Architecture::RiscV);
    }
}
