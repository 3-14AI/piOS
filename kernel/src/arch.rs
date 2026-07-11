#![allow(dead_code)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
verus! {
    pub enum Architecture {
        X86_64,
        AArch64,
        RiscV,
    }

    pub struct ArchSupport {
        pub arch: Architecture,
    }

    impl ArchSupport {
        pub fn new(arch: Architecture) -> (s: Self)
            ensures s.arch == arch
        {
            ArchSupport { arch }
        }
    }

    pub mod aarch64 {
        use vstd::prelude::*;
        verus! {
            pub struct Uart {
                pub base_addr: usize,
            }

            impl Uart {
                pub fn new(base_addr: usize) -> (u: Self)
                    ensures
                        u.base_addr == base_addr
                {
                    Uart { base_addr }
                }

                #[verifier::external_body]
                pub fn write_byte(&mut self, byte: u8) {
                    // Flag register offset: 0x18
                    // Data register offset: 0x000
                    // Bit 5 (TXFF) is 1 when transmit FIFO is full
                    let ptr = self.base_addr as *mut u32;
                    unsafe {
                        while (core::ptr::read_volatile(ptr.add(6)) & (1 << 5)) != 0 {
                            core::hint::spin_loop();
                        }
                        core::ptr::write_volatile(ptr, byte as u32);
                    }
                }
            }
        }
    }
}

#[cfg(not(feature = "verus"))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Architecture {
    X86_64,
    AArch64,
    RiscV,
}

#[cfg(not(feature = "verus"))]
pub struct ArchSupport {
    pub arch: Architecture,
}

#[cfg(not(feature = "verus"))]
impl ArchSupport {
    pub fn new(arch: Architecture) -> Self {
        ArchSupport { arch }
    }
}

#[cfg(not(feature = "verus"))]
pub mod aarch64 {
    #[derive(Debug)]
    pub struct Uart {
        pub base_addr: usize,
    }

    impl Uart {
        pub fn new(base_addr: usize) -> Self {
            Uart { base_addr }
        }

        pub fn write_byte(&mut self, byte: u8) {
            let ptr = self.base_addr as *mut u32;
            unsafe {
                while (core::ptr::read_volatile(ptr.add(6)) & (1 << 5)) != 0 {
                    core::hint::spin_loop();
                }
                core::ptr::write_volatile(ptr, byte as u32);
            }
        }
    }
}

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_support() {
        let arch_x86 = ArchSupport::new(Architecture::X86_64);
        assert_eq!(arch_x86.arch, Architecture::X86_64);

        let arch_arm = ArchSupport::new(Architecture::AArch64);
        assert_eq!(arch_arm.arch, Architecture::AArch64);

        let arch_riscv = ArchSupport::new(Architecture::RiscV);
        assert_eq!(arch_riscv.arch, Architecture::RiscV);
    }

    #[test]
    fn test_aarch64_uart_write() {
        let mut mmio_region = [0u32; 8];
        let base_addr = mmio_region.as_mut_ptr() as usize;
        let mut uart = super::aarch64::Uart::new(base_addr);

        assert_eq!(uart.base_addr, base_addr);

        // Test writing a byte. Since mmio_region is all 0, TXFF is 0 (not full)
        uart.write_byte(b'A');
        assert_eq!(mmio_region[0], b'A' as u32);
    }
}
