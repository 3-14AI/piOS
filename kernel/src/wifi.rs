#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
use crate::pci::PciConfig;

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

    pub struct IntelWifiDriver {
        pub mmio_base: u64,
        pub tx_ring_base: u64,
        pub rx_ring_base: u64,
        pub initialized: bool,
    }

    impl IntelWifiDriver {
        pub fn new() -> (d: Self)
            ensures
                d.mmio_base == 0,
                d.tx_ring_base == 0,
                d.rx_ring_base == 0,
                d.initialized == false
        {
            IntelWifiDriver {
                mmio_base: 0,
                tx_ring_base: 0,
                rx_ring_base: 0,
                initialized: false,
            }
        }

        pub fn init(&mut self, config: &PciConfig)
            requires
                old(self).initialized == false
            ensures
                self.initialized == true
        {
            self.mmio_base = config.device_id as u64; // mock
            self.tx_ring_base = 0x1000;
            self.rx_ring_base = 0x2000;
            self.initialized = true;
        }

        pub fn send_packet(&mut self, _data: &[u8]) -> (success: bool)
            requires
                old(self).initialized == true
            ensures
                self.tx_ring_base != 0,
                success == true,
                self.initialized == true,
                self.mmio_base == old(self).mmio_base,
                self.rx_ring_base == old(self).rx_ring_base
        {
            self.tx_ring_base = 0x1001;
            true
        }

        pub fn receive_packet(&mut self, buffer: &mut [u8]) -> (res: usize)
            requires
                old(self).initialized == true
            ensures
                self.initialized == true,
                self.mmio_base == old(self).mmio_base,
                self.tx_ring_base == old(self).tx_ring_base,
                self.rx_ring_base == old(self).rx_ring_base
        {
            if self.rx_ring_base != 0 {
                if buffer.len() >= 14 {
                    14
                } else {
                    buffer.len()
                }
            } else {
                0
            }
        }
    }
}

#[cfg(not(feature = "verus"))]
use crate::pci::PciConfig;

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
#[derive(Debug)]
pub struct IntelWifiDriver {
    pub mmio_base: u64,
    pub tx_ring_base: u64,
    pub rx_ring_base: u64,
    pub initialized: bool,
}

#[cfg(not(feature = "verus"))]
impl IntelWifiDriver {
    pub fn new() -> Self {
        IntelWifiDriver {
            mmio_base: 0,
            tx_ring_base: 0,
            rx_ring_base: 0,
            initialized: false,
        }
    }

    pub fn init(&mut self, config: &PciConfig) {
        self.mmio_base = config.device_id as u64; // mock
        self.tx_ring_base = 0x1000;
        self.rx_ring_base = 0x2000;
        self.initialized = true;
    }

    pub fn send_packet(&mut self, _data: &[u8]) -> bool {
        if !self.initialized {
            return false;
        }
        self.tx_ring_base = 0x1001;
        true
    }

    pub fn receive_packet(&mut self, buffer: &mut [u8]) -> usize {
        if !self.initialized || self.rx_ring_base == 0 {
            return 0;
        }
        let mock_data = b"wifi_mock_data";
        let len = mock_data.len().min(buffer.len());
        buffer[..len].copy_from_slice(&mock_data[..len]);
        len
    }
}

#[cfg(not(feature = "verus"))]
impl Default for IntelWifiDriver {
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
        assert!(drv.initialized);
        let drv_def = WifiBluetoothDriver::default();
        assert!(drv_def.initialized);
    }

    #[test]
    fn test_intel_wifi_driver() {
        let mut drv = IntelWifiDriver::new();
        assert!(!drv.initialized);

        let config = PciConfig {
            vendor_id: 0x8086,
            device_id: 0x0001,
            class_code: 0x02,
            subclass: 0x80,
            prog_if: 0x00,
            bus: 0,
            device: 0,
            function: 0,
        };

        drv.init(&config);
        assert!(drv.initialized);
        assert_eq!(drv.mmio_base, 0x0001);

        assert!(drv.send_packet(b"hello"));
        assert_eq!(drv.tx_ring_base, 0x1001);

        let mut buf = [0u8; 32];
        let len = drv.receive_packet(&mut buf);
        assert_eq!(len, 14);
        assert_eq!(&buf[..14], b"wifi_mock_data");

        let drv_def = IntelWifiDriver::default();
        assert!(!drv_def.initialized);
    }
}
