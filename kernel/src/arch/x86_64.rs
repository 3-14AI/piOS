#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct X86_64Arch {
        pub initialized: bool,
    }

    impl X86_64Arch {
        pub fn new() -> (d: Self)
            ensures d.initialized == true
        {
            X86_64Arch { initialized: true }
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct X86_64Arch {
    pub initialized: bool,
}

#[cfg(not(feature = "verus"))]
impl X86_64Arch {
    pub fn new() -> Self {
        X86_64Arch { initialized: true }
    }
}

#[cfg(not(feature = "verus"))]
impl Default for X86_64Arch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x86_64_arch() {
        let arch = X86_64Arch::new();
        assert_eq!(arch.initialized, true);
    }
}
