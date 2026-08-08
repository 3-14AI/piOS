#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub struct EhciTransferDescriptor {
        pub next_qh: u32,
        pub token: u32,
        pub buffer_ptr: u32,
    }

    impl EhciTransferDescriptor {
        pub fn new(next_qh: u32, token: u32, buffer_ptr: u32) -> (t: Self)
            ensures
                t.next_qh == next_qh,
                t.token == token,
                t.buffer_ptr == buffer_ptr
        {
            EhciTransferDescriptor { next_qh, token, buffer_ptr }
        }
    }

    pub struct EhciQueueHead {
        pub next_qh: u32,
        pub characteristics: u32,
        pub capabilities: u32,
        pub current_td: u32,
        pub next_td: u32,
        pub token: u32,
        pub buffer_ptr: u32,
    }

    impl EhciQueueHead {
        pub fn new(next_qh: u32, characteristics: u32, capabilities: u32) -> (q: Self)
            ensures
                q.next_qh == next_qh,
                q.characteristics == characteristics,
                q.capabilities == capabilities,
                q.current_td == 0,
                q.next_td == 0,
                q.token == 0,
                q.buffer_ptr == 0
        {
            EhciQueueHead {
                next_qh,
                characteristics,
                capabilities,
                current_td: 0,
                next_td: 0,
                token: 0,
                buffer_ptr: 0,
            }
        }
    }

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

        pub fn init_device(&mut self, qh_ptr: usize) -> (success: bool) {
            // Write to command register to run
            self.write_register(0x18, qh_ptr as u32);
            self.write_register(0x20, 1);
            self.initialized = true;
            true
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug, Clone, Copy)]
pub struct EhciTransferDescriptor {
    pub next_qh: u32,
    pub token: u32,
    pub buffer_ptr: u32,
}

#[cfg(not(feature = "verus"))]
impl EhciTransferDescriptor {
    pub fn new(next_qh: u32, token: u32, buffer_ptr: u32) -> Self {
        EhciTransferDescriptor {
            next_qh,
            token,
            buffer_ptr,
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug, Clone, Copy)]
pub struct EhciQueueHead {
    pub next_qh: u32,
    pub characteristics: u32,
    pub capabilities: u32,
    pub current_td: u32,
    pub next_td: u32,
    pub token: u32,
    pub buffer_ptr: u32,
}

#[cfg(not(feature = "verus"))]
impl EhciQueueHead {
    pub fn new(next_qh: u32, characteristics: u32, capabilities: u32) -> Self {
        EhciQueueHead {
            next_qh,
            characteristics,
            capabilities,
            current_td: 0,
            next_td: 0,
            token: 0,
            buffer_ptr: 0,
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

    pub fn init_device(&mut self, qh_ptr: usize) -> bool {
        // Mock hardware initialization
        // Write to USBCMD register to run
        self.write_register(0x18, qh_ptr as u32);
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
        assert!(!drv.initialized);

        let qh = EhciQueueHead::new(0, 0, 0);
        let qh_addr = &qh as *const _ as usize;

        assert!(drv.init_device(qh_addr));
        assert!(drv.initialized);

        // Verify that ASYNC LIST ADDR (offset 0x18 is index 6) is set to qh_addr
        assert_eq!(registers[6], qh_addr as u32);
        // Verify that the command register was written (offset 0x20 is index 8)
        assert_eq!(registers[8], 1);
    }
}
