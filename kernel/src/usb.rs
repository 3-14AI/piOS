#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct UsbDriver {
        pub initialized: bool,
    }

    impl UsbDriver {
        pub fn new() -> (d: Self)
            ensures d.initialized == true
        {
            UsbDriver { initialized: true }
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct UsbDriver {
    pub initialized: bool,
}

#[cfg(not(feature = "verus"))]
impl UsbDriver {
    pub fn new() -> Self {
        UsbDriver { initialized: true }
    }
}

#[cfg(not(feature = "verus"))]
impl Default for UsbDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb_driver() {
        let drv = UsbDriver::new();
        assert_eq!(drv.initialized, true);
        let drv_def = UsbDriver::default();
        assert_eq!(drv_def.initialized, true);
    }
}
