#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub enum UsbSpeed {
        Low,
        Full,
        High,
        Super,
    }

    pub enum UsbTransferType {
        Control,
        Isochronous,
        Bulk,
        Interrupt,
    }

    pub struct UsbEndpoint {
        pub address: u8,
        pub transfer_type: UsbTransferType,
        pub max_packet_size: u16,
        pub interval: u8,
    }

    impl UsbEndpoint {
        pub fn new(address: u8, transfer_type: UsbTransferType, max_packet_size: u16, interval: u8) -> (e: Self)
            ensures
                e.address == address,
                e.max_packet_size == max_packet_size,
                e.interval == interval
        {
            UsbEndpoint {
                address,
                transfer_type,
                max_packet_size,
                interval,
            }
        }
    }

    pub struct Urb {
        pub endpoint_addr: u8,
        pub buffer_ptr: usize,
        pub buffer_length: usize,
        pub actual_length: usize,
        pub status: i32,
    }

    impl Urb {
        pub fn new(endpoint_addr: u8, buffer_ptr: usize, buffer_length: usize) -> (u: Self)
            ensures
                u.endpoint_addr == endpoint_addr,
                u.buffer_ptr == buffer_ptr,
                u.buffer_length == buffer_length,
                u.actual_length == 0,
                u.status == 0
        {
            Urb {
                endpoint_addr,
                buffer_ptr,
                buffer_length,
                actual_length: 0,
                status: 0,
            }
        }
    }

    pub struct UsbDevice {
        pub address: u8,
        pub port: u8,
        pub speed: UsbSpeed,
        pub connected: bool,
    }

    impl UsbDevice {
        pub fn new(address: u8, port: u8, speed: UsbSpeed) -> (d: Self)
            ensures
                d.address == address,
                d.port == port,
                d.connected == true
        {
            UsbDevice {
                address,
                port,
                speed,
                connected: true,
            }
        }
    }

    pub struct UsbHub {
        pub address: u8,
        pub num_ports: u8,
        pub is_root: bool,
    }

    impl UsbHub {
        pub fn new(address: u8, num_ports: u8, is_root: bool) -> (h: Self)
            ensures
                h.address == address,
                h.num_ports == num_ports,
                h.is_root == is_root
        {
            UsbHub {
                address,
                num_ports,
                is_root,
            }
        }
    }

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
#[derive(Debug, PartialEq, Eq)]
pub enum UsbSpeed {
    Low,
    Full,
    High,
    Super,
}

#[cfg(not(feature = "verus"))]
#[derive(Debug, PartialEq, Eq)]
pub enum UsbTransferType {
    Control,
    Isochronous,
    Bulk,
    Interrupt,
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct UsbEndpoint {
    pub address: u8,
    pub transfer_type: UsbTransferType,
    pub max_packet_size: u16,
    pub interval: u8,
}

#[cfg(not(feature = "verus"))]
impl UsbEndpoint {
    pub fn new(
        address: u8,
        transfer_type: UsbTransferType,
        max_packet_size: u16,
        interval: u8,
    ) -> Self {
        UsbEndpoint {
            address,
            transfer_type,
            max_packet_size,
            interval,
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct Urb {
    pub endpoint_addr: u8,
    pub buffer_ptr: usize,
    pub buffer_length: usize,
    pub actual_length: usize,
    pub status: i32,
}

#[cfg(not(feature = "verus"))]
impl Urb {
    pub fn new(endpoint_addr: u8, buffer_ptr: usize, buffer_length: usize) -> Self {
        Urb {
            endpoint_addr,
            buffer_ptr,
            buffer_length,
            actual_length: 0,
            status: 0,
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct UsbDevice {
    pub address: u8,
    pub port: u8,
    pub speed: UsbSpeed,
    pub connected: bool,
}

#[cfg(not(feature = "verus"))]
impl UsbDevice {
    pub fn new(address: u8, port: u8, speed: UsbSpeed) -> Self {
        UsbDevice {
            address,
            port,
            speed,
            connected: true,
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct UsbHub {
    pub address: u8,
    pub num_ports: u8,
    pub is_root: bool,
}

#[cfg(not(feature = "verus"))]
impl UsbHub {
    pub fn new(address: u8, num_ports: u8, is_root: bool) -> Self {
        UsbHub {
            address,
            num_ports,
            is_root,
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
    fn test_usb_endpoint() {
        let ep = UsbEndpoint::new(1, UsbTransferType::Bulk, 512, 0);
        assert_eq!(ep.address, 1);
        assert_eq!(ep.transfer_type, UsbTransferType::Bulk);
        assert_eq!(ep.max_packet_size, 512);
        assert_eq!(ep.interval, 0);
    }

    #[test]
    fn test_urb() {
        let urb = Urb::new(1, 0x1000, 1024);
        assert_eq!(urb.endpoint_addr, 1);
        assert_eq!(urb.buffer_ptr, 0x1000);
        assert_eq!(urb.buffer_length, 1024);
        assert_eq!(urb.actual_length, 0);
        assert_eq!(urb.status, 0);
    }

    #[test]
    fn test_usb_device() {
        let dev = UsbDevice::new(2, 1, UsbSpeed::Super);
        assert_eq!(dev.address, 2);
        assert_eq!(dev.port, 1);
        assert_eq!(dev.speed, UsbSpeed::Super);
        assert_eq!(dev.connected, true);
    }

    #[test]
    fn test_usb_hub() {
        let hub = UsbHub::new(1, 4, true);
        assert_eq!(hub.address, 1);
        assert_eq!(hub.num_ports, 4);
        assert_eq!(hub.is_root, true);
    }

    #[test]
    fn test_usb_driver() {
        let drv = UsbDriver::new();
        assert_eq!(drv.initialized, true);
        let drv_def = UsbDriver::default();
        assert_eq!(drv_def.initialized, true);
    }
}
