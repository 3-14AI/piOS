#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct Aarch64Arch {
        pub initialized: bool,
    }

    impl Aarch64Arch {
        pub fn new() -> (d: Self)
            ensures d.initialized == true
        {
            Aarch64Arch { initialized: true }
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct Aarch64Arch {
    pub initialized: bool,
}

#[cfg(not(feature = "verus"))]
impl Aarch64Arch {
    pub fn new() -> Self {
        Aarch64Arch { initialized: true }
    }
}

#[cfg(not(feature = "verus"))]
impl Default for Aarch64Arch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aarch64_arch() {
        let arch = Aarch64Arch::new();
        assert_eq!(arch.initialized, true);
    }
}
