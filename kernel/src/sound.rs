#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct HdaSoundDriver {
        pub initialized: bool,
    }

    impl HdaSoundDriver {
        pub fn new() -> (d: Self)
            ensures d.initialized == true
        {
            HdaSoundDriver { initialized: true }
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct HdaSoundDriver {
    pub initialized: bool,
}

#[cfg(not(feature = "verus"))]
impl HdaSoundDriver {
    pub fn new() -> Self {
        HdaSoundDriver { initialized: true }
    }
}

#[cfg(not(feature = "verus"))]
impl Default for HdaSoundDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_driver() {
        let drv = HdaSoundDriver::new();
        assert_eq!(drv.initialized, true);
    }
}
