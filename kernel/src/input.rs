#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct HidInputDriver {
        pub initialized: bool,
    }

    impl HidInputDriver {
        pub fn new() -> (d: Self)
            ensures d.initialized == true
        {
            HidInputDriver { initialized: true }
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct HidInputDriver {
    pub initialized: bool,
}

#[cfg(not(feature = "verus"))]
impl HidInputDriver {
    pub fn new() -> Self {
        HidInputDriver { initialized: true }
    }
}

#[cfg(not(feature = "verus"))]
impl Default for HidInputDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_driver() {
        let drv = HidInputDriver::new();
        assert_eq!(drv.initialized, true);
    }
}
