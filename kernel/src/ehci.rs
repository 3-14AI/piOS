#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct EhciDriver {
        pub mmio_base: usize,
        pub initialized: bool,
    }

    impl EhciDriver {
        pub fn new(mmio_base: usize) -> (d: Self)
            ensures d.mmio_base == mmio_base
        {
            EhciDriver {
                mmio_base,
                initialized: false,
            }
        }

        #[verifier::external_body]
        pub fn read_register(&self, offset: usize) -> (val: u32) {
            unsafe {
                let ptr = (self.mmio_base + offset) as *const u32;
                core::ptr::read_volatile(ptr)
            }
        }

        #[verifier::external_body]
        pub fn write_register(&mut self, offset: usize, value: u32) {
            unsafe {
                let ptr = (self.mmio_base + offset) as *mut u32;
                core::ptr::write_volatile(ptr, value);
            }
        }

        pub fn init_device(&mut self) -> (success: bool) {
            // Write to command register to run
            self.write_register(0x20, 1);
            self.initialized = true;
            true
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug)]
pub struct EhciDriver {
    pub mmio_base: usize,
    pub initialized: bool,
}

#[cfg(not(feature = "verus"))]
impl EhciDriver {
    pub fn new(mmio_base: usize) -> Self {
        EhciDriver {
            mmio_base,
            initialized: false,
        }
    }

    pub fn read_register(&self, offset: usize) -> u32 {
        unsafe {
            let ptr = (self.mmio_base + offset) as *const u32;
            core::ptr::read_volatile(ptr)
        }
    }

    pub fn write_register(&mut self, offset: usize, value: u32) {
        unsafe {
            let ptr = (self.mmio_base + offset) as *mut u32;
            core::ptr::write_volatile(ptr, value);
        }
    }

    pub fn init_device(&mut self) -> bool {
        // Mock hardware initialization
        // Write to USBCMD register to run
        self.write_register(0x20, 1);
        self.initialized = true;
        true
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ehci_initialization() {
        // Mock hardware registers with a local array to prevent segfaults
        let mut registers: [u32; 64] = [0; 64];
        let mmio_base = registers.as_mut_ptr() as usize;

        let mut drv = EhciDriver::new(mmio_base);
        assert_eq!(drv.initialized, false);

        assert_eq!(drv.init_device(), true);
        assert_eq!(drv.initialized, true);

        // Verify that the command register was written (offset 0x20 is index 8)
        assert_eq!(registers[8], 1);
    }
}
