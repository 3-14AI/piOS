#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    /// Mocking out some structures to integrate with VFS and Block layer.
    pub struct AhciPort {
        pub state: u32,
    }

    impl AhciPort {
        pub fn new() -> (p: Self)
            ensures p.state == 0
        {
            AhciPort { state: 0 }
        }
    }

    /// BlockDevice trait simulation for integration
    pub trait BlockDevice {
        fn read_sector(&mut self, sector: u64, buffer_addr: usize) -> bool;
    }

    pub struct AhciDriver {
        pub capacity: u64,
        pub initialized: bool,
    }

    impl AhciDriver {
        pub fn new(capacity: u64) -> (d: Self)
            ensures
                d.initialized == true,
                d.capacity == capacity
        {
            AhciDriver { capacity, initialized: true }
        }

        #[verifier::external_body]
        pub fn init_device(&mut self, _base_addr: usize) -> (success: bool)
            ensures success == true // simplified for verus mock
        {
            // Hardware MMIO mapped logic simulation
            true
        }

        pub fn read_sector(&mut self, sector: u64, _buffer_addr: usize) -> (success: bool)
            ensures
                self.capacity == old(self).capacity,
                self.initialized == old(self).initialized,
                success ==> sector < self.capacity
        {
            if sector >= self.capacity {
                false
            } else {
                true
            }
        }
    }

    impl BlockDevice for AhciDriver {
        fn read_sector(&mut self, sector: u64, buffer_addr: usize) -> bool {
            self.read_sector(sector, buffer_addr)
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct AhciPort {
    pub state: u32,
}

#[cfg(not(feature = "verus"))]
impl AhciPort {
    pub fn new() -> Self {
        AhciPort { state: 0 }
    }
}

#[cfg(not(feature = "verus"))]
impl Default for AhciPort {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "verus"))]
pub trait BlockDevice {
    fn read_sector(&mut self, sector: u64, buffer_addr: usize) -> bool;
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct AhciDriver {
    pub capacity: u64,
    pub initialized: bool,
}

#[cfg(not(feature = "verus"))]
impl AhciDriver {
    pub fn new(capacity: u64) -> Self {
        AhciDriver {
            capacity,
            initialized: true,
        }
    }

    pub fn init_device(&mut self, base_addr: usize) -> bool {
        // Mocking hardware initialization logic with MMIO using raw pointers
        unsafe {
            let ptr = base_addr as *mut u32;
            let val = core::ptr::read_volatile(ptr);
            core::ptr::write_volatile(ptr, val | 1);
        }
        true
    }

    pub fn read_sector(&mut self, sector: u64, buffer_addr: usize) -> bool {
        if sector >= self.capacity {
            return false;
        }

        // Mocking DMA read to buffer
        unsafe {
            let ptr = buffer_addr as *mut u8;
            core::ptr::write_volatile(ptr, 0xaa);
        }

        true
    }
}

#[cfg(not(feature = "verus"))]
impl BlockDevice for AhciDriver {
    fn read_sector(&mut self, sector: u64, buffer_addr: usize) -> bool {
        self.read_sector(sector, buffer_addr)
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ahci_port() {
        let p = AhciPort::new();
        assert_eq!(p.state, 0);
        let p_def = AhciPort::default();
        assert_eq!(p_def.state, 0);
    }

    #[test]
    fn test_ahci_driver() {
        let mut d = AhciDriver::new(100);
        assert_eq!(d.initialized, true);
        assert_eq!(d.capacity, 100);

        // Use a local array as a mock for MMIO base address
        let mut mmio_mock = [0u32; 1024];
        let base_addr = mmio_mock.as_mut_ptr() as usize;

        assert_eq!(d.init_device(base_addr), true);
        assert_eq!(mmio_mock[0], 1);

        let mut buffer = [0u8; 512];
        let buffer_addr = buffer.as_mut_ptr() as usize;
        assert_eq!(d.read_sector(50, buffer_addr), true);
        assert_eq!(buffer[0], 0xaa);

        assert_eq!(d.read_sector(200, buffer_addr), false);
    }

    #[test]
    fn test_block_device_trait() {
        let mut d = AhciDriver::new(100);
        let mut buffer = [0u8; 512];
        let buffer_addr = buffer.as_mut_ptr() as usize;

        // Use through trait
        let bd: &mut dyn BlockDevice = &mut d;
        assert_eq!(bd.read_sector(50, buffer_addr), true);
        assert_eq!(buffer[0], 0xaa);
    }
}
