#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct WifiBluetoothDriver {
        pub initialized: bool,
    }

    impl WifiBluetoothDriver {
        pub fn new() -> (d: Self)
            ensures d.initialized == true
        {
            WifiBluetoothDriver { initialized: true }
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct WifiBluetoothDriver {
    pub initialized: bool,
}

#[cfg(not(feature = "verus"))]
impl WifiBluetoothDriver {
    pub fn new() -> Self {
        WifiBluetoothDriver { initialized: true }
    }
}

#[cfg(not(feature = "verus"))]
impl Default for WifiBluetoothDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wifi_driver() {
        let drv = WifiBluetoothDriver::new();
        assert_eq!(drv.initialized, true);
        let drv_def = WifiBluetoothDriver::default();
        assert_eq!(drv_def.initialized, true);
    }
}
