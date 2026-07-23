#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub enum HidDeviceType {
        Keyboard,
        Mouse,
        Unknown,
    }

    pub struct HidReport {
        pub device_type: HidDeviceType,
        pub data_ptr: usize,
        pub data_len: usize,
    }

    impl HidReport {
        pub fn new(device_type: HidDeviceType, data_ptr: usize, data_len: usize) -> (r: Self)
            ensures
                r.device_type == device_type,
                r.data_ptr == data_ptr,
                r.data_len == data_len
        {
            HidReport {
                device_type,
                data_ptr,
                data_len,
            }
        }
    }

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

    pub struct UsbHidDriver {
        pub endpoint_addr: u8,
        pub initialized: bool,
    }

    impl UsbHidDriver {
        pub fn new(endpoint_addr: u8) -> (d: Self)
            ensures
                d.endpoint_addr == endpoint_addr,
                d.initialized == true
        {
            UsbHidDriver { endpoint_addr, initialized: true }
        }

        pub fn handle_urb(&self, urb: &crate::usb::Urb) -> (success: bool)
            requires self.initialized == true
            ensures success == (urb.endpoint_addr == self.endpoint_addr && urb.actual_length > 0)
        {
            urb.endpoint_addr == self.endpoint_addr && urb.actual_length > 0
        }
    }

    pub enum EventType {
        Sync,
        Key,
        Rel,
        Abs,
    }

    pub struct InputEvent {
        pub event_type: EventType,
        pub code: u16,
        pub value: i32,
    }

    impl InputEvent {
        pub fn new(event_type: EventType, code: u16, value: i32) -> (e: Self)
            ensures
                e.event_type == event_type,
                e.code == code,
                e.value == value
        {
            InputEvent {
                event_type,
                code,
                value,
            }
        }
    }

    pub struct InputSubsystem {
        pub active: bool,
    }

    impl InputSubsystem {
        pub fn new() -> (i: Self)
            ensures i.active == true
        {
            InputSubsystem { active: true }
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug, PartialEq, Eq)]
pub enum HidDeviceType {
    Keyboard,
    Mouse,
    Unknown,
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct HidReport {
    pub device_type: HidDeviceType,
    pub data_ptr: usize,
    pub data_len: usize,
}

#[cfg(not(feature = "verus"))]
impl HidReport {
    pub fn new(device_type: HidDeviceType, data_ptr: usize, data_len: usize) -> Self {
        HidReport {
            device_type,
            data_ptr,
            data_len,
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
#[derive(Debug)]
pub struct UsbHidDriver {
    pub endpoint_addr: u8,
    pub initialized: bool,
}

#[cfg(not(feature = "verus"))]
impl UsbHidDriver {
    pub fn new(endpoint_addr: u8) -> Self {
        UsbHidDriver {
            endpoint_addr,
            initialized: true,
        }
    }

    pub fn handle_urb(&self, urb: &crate::usb::Urb) -> bool {
        if !self.initialized {
            return false;
        }
        urb.endpoint_addr == self.endpoint_addr && urb.actual_length > 0
    }
}

#[cfg(not(feature = "verus"))]
impl Default for HidInputDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug, PartialEq, Eq)]
pub enum EventType {
    Sync,
    Key,
    Rel,
    Abs,
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct InputEvent {
    pub event_type: EventType,
    pub code: u16,
    pub value: i32,
}

#[cfg(not(feature = "verus"))]
impl InputEvent {
    pub fn new(event_type: EventType, code: u16, value: i32) -> Self {
        InputEvent {
            event_type,
            code,
            value,
        }
    }
}

#[cfg(not(feature = "verus"))]
pub struct InputSubsystem {
    pub active: bool,
}

#[cfg(not(feature = "verus"))]
impl InputSubsystem {
    pub fn new() -> Self {
        InputSubsystem { active: true }
    }
}

#[cfg(not(feature = "verus"))]
impl Default for InputSubsystem {
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
        let drv_def = HidInputDriver::default();
        assert_eq!(drv_def.initialized, true);
    }

    #[test]
    fn test_usb_hid_driver() {
        let drv = UsbHidDriver::new(1);
        assert_eq!(drv.endpoint_addr, 1);
        assert_eq!(drv.initialized, true);

        let mut urb = crate::usb::Urb::new(1, 0x2000, 8);
        urb.actual_length = 8;
        assert_eq!(drv.handle_urb(&urb), true);

        let mut bad_urb = crate::usb::Urb::new(2, 0x2000, 8);
        bad_urb.actual_length = 8;
        assert_eq!(drv.handle_urb(&bad_urb), false);

        urb.actual_length = 0;
        assert_eq!(drv.handle_urb(&urb), false);
    }

    #[test]
    fn test_input_subsystem() {
        let event = InputEvent::new(EventType::Key, 1, 1);
        assert_eq!(event.event_type, EventType::Key);
        assert_eq!(event.code, 1);
        assert_eq!(event.value, 1);

        let subsys = InputSubsystem::new();
        assert_eq!(subsys.active, true);

        let subsys_def = InputSubsystem::default();
        assert_eq!(subsys_def.active, true);
    }
}
