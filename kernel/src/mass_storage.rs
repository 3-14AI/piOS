#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub enum UsbMassStorageProtocol {
        Control,
        BulkOnly,
    }

    pub struct UsbMassStorageDriver {
        pub protocol: UsbMassStorageProtocol,
        pub initialized: bool,
    }

    impl UsbMassStorageDriver {
        pub fn new() -> (d: Self)
            ensures
                d.initialized == false
        {
            UsbMassStorageDriver {
                protocol: UsbMassStorageProtocol::BulkOnly,
                initialized: false,
            }
        }

        pub fn init(&mut self) -> (success: bool)
            ensures
                self.initialized == true,
                success == true
        {
            self.initialized = true;
            true
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug, PartialEq, Eq)]
pub enum UsbMassStorageProtocol {
    Control,
    BulkOnly,
}

#[cfg(not(feature = "verus"))]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CommandBlockWrapper {
    pub signature: u32,
    pub tag: u32,
    pub data_transfer_length: u32,
    pub flags: u8,
    pub lun: u8,
    pub cb_length: u8,
    pub cb: [u8; 16],
}

#[cfg(not(feature = "verus"))]
impl CommandBlockWrapper {
    pub fn new(tag: u32, data_len: u32, direction_in: bool, lun: u8, cb: &[u8]) -> Self {
        let mut cb_array = [0u8; 16];
        let cb_len = cb.len().min(16);
        cb_array[..cb_len].copy_from_slice(&cb[..cb_len]);

        CommandBlockWrapper {
            signature: 0x43425355, // 'USBC'
            tag,
            data_transfer_length: data_len,
            flags: if direction_in { 0x80 } else { 0x00 },
            lun: lun & 0x0F,
            cb_length: cb_len as u8,
            cb: cb_array,
        }
    }
}

#[cfg(not(feature = "verus"))]
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CommandStatusWrapper {
    pub signature: u32,
    pub tag: u32,
    pub data_residue: u32,
    pub status: u8,
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct UsbMassStorageDriver {
    pub protocol: UsbMassStorageProtocol,
    pub initialized: bool,
    pub tag_counter: u32,
}

#[cfg(not(feature = "verus"))]
impl UsbMassStorageDriver {
    pub fn new() -> Self {
        UsbMassStorageDriver {
            protocol: UsbMassStorageProtocol::BulkOnly,
            initialized: false,
            tag_counter: 1,
        }
    }

    pub fn init(&mut self) -> bool {
        self.initialized = true;
        true
    }

    pub fn read_blocks(&mut self, lba: u32, count: u16, buffer: &mut [u8]) -> bool {
        if !self.initialized || buffer.len() < (count as usize) * 512 {
            return false;
        }

        let mut cb = [0u8; 10];
        cb[0] = 0x28; // READ(10)
        cb[2] = (lba >> 24) as u8;
        cb[3] = (lba >> 16) as u8;
        cb[4] = (lba >> 8) as u8;
        cb[5] = lba as u8;
        cb[7] = (count >> 8) as u8;
        cb[8] = count as u8;

        let _cbw = CommandBlockWrapper::new(self.tag_counter, (count as u32) * 512, true, 0, &cb);
        self.tag_counter += 1;

        // Mock hardware simulation
        for b in buffer.iter_mut() {
            *b = 0xAA;
        }
        true
    }

    pub fn write_blocks(&mut self, lba: u32, count: u16, buffer: &[u8]) -> bool {
        if !self.initialized || buffer.len() < (count as usize) * 512 {
            return false;
        }

        let mut cb = [0u8; 10];
        cb[0] = 0x2A; // WRITE(10)
        cb[2] = (lba >> 24) as u8;
        cb[3] = (lba >> 16) as u8;
        cb[4] = (lba >> 8) as u8;
        cb[5] = lba as u8;
        cb[7] = (count >> 8) as u8;
        cb[8] = count as u8;

        let _cbw = CommandBlockWrapper::new(self.tag_counter, (count as u32) * 512, false, 0, &cb);
        self.tag_counter += 1;

        // Mock hardware simulation
        true
    }
}

#[cfg(not(feature = "verus"))]
impl Default for UsbMassStorageDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb_mass_storage_cbw() {
        let cbw = CommandBlockWrapper::new(1, 512, true, 0, &[0x28, 0, 0, 0, 0, 0, 0, 0, 1, 0]);
        assert_eq!(cbw.signature, 0x43425355);
        assert_eq!(cbw.tag, 1);
        assert_eq!(cbw.data_transfer_length, 512);
        assert_eq!(cbw.flags, 0x80);
        assert_eq!(cbw.cb_length, 10);
        assert_eq!(cbw.cb[0], 0x28);
    }

    #[test]
    fn test_usb_mass_storage_initialization() {
        let mut driver = UsbMassStorageDriver::new();
        assert!(!driver.initialized);
        assert!(driver.init());
        assert!(driver.initialized);
    }

    #[test]
    fn test_usb_mass_storage_read_write() {
        let mut driver = UsbMassStorageDriver::new();
        let mut buffer = [0u8; 512];
        assert!(!driver.read_blocks(0, 1, &mut buffer));
        assert!(!driver.write_blocks(0, 1, &buffer));

        driver.init();
        assert!(driver.read_blocks(0, 1, &mut buffer));
        assert_eq!(buffer[0], 0xAA);
        assert!(driver.write_blocks(0, 1, &buffer));

        let mut small_buffer = [0u8; 128];
        assert!(!driver.read_blocks(0, 1, &mut small_buffer));
        assert!(!driver.write_blocks(0, 1, &small_buffer));
    }

    #[test]
    fn test_usb_mass_storage_default() {
        let driver = UsbMassStorageDriver::default();
        assert!(!driver.initialized);
    }

    #[test]
    fn test_csw() {
        let csw = CommandStatusWrapper::default();
        assert_eq!(csw.signature, 0);
    }
}
