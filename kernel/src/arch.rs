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

            pub struct Gic {
                pub gicd_base: usize,
                pub gicc_base: usize,
            }

            impl Gic {
                pub fn new(gicd_base: usize, gicc_base: usize) -> (g: Self)
                    ensures
                        g.gicd_base == gicd_base,
                        g.gicc_base == gicc_base,
                {
                    Gic { gicd_base, gicc_base }
                }

                #[verifier::external_body]
                pub fn enable(&mut self, irq: u32) {
                    let ptr = (self.gicd_base + 0x100 + ((irq / 32) * 4) as usize) as *mut u32;
                    unsafe {
                        core::ptr::write_volatile(ptr, 1 << (irq % 32));
                    }
                }

                #[verifier::external_body]
                pub fn ack(&mut self) -> u32 {
                    let ptr = (self.gicc_base + 0x0C) as *mut u32;
                    unsafe {
                        core::ptr::read_volatile(ptr)
                    }
                }

                #[verifier::external_body]
                pub fn eoi(&mut self, irq: u32) {
                    let ptr = (self.gicc_base + 0x10) as *mut u32;
                    unsafe {
                        core::ptr::write_volatile(ptr, irq);
                    }
                }
            }

            pub struct GenericTimer;

            impl Default for GenericTimer {
                fn default() -> Self {
                    Self::new()
                }
            }

            impl GenericTimer {
                pub fn new() -> (t: Self) {
                    GenericTimer {}
                }

                #[verifier::external_body]
                #[cfg(target_arch = "aarch64")]
                pub fn enable(&mut self) {
                    unsafe {
                        core::arch::asm!(
                            "msr cntv_ctl_el0, {0}",
                            in(reg) 1u64,
                        );
                    }
                }

                #[verifier::external_body]
                #[cfg(not(target_arch = "aarch64"))]
                pub fn enable(&mut self) {}

                #[verifier::external_body]
                #[cfg(target_arch = "aarch64")]
                pub fn set_timer(&mut self, ticks: u64) {
                    unsafe {
                        core::arch::asm!(
                            "msr cntv_tval_el0, {0}",
                            in(reg) ticks,
                        );
                    }
                }

                #[verifier::external_body]
                #[cfg(not(target_arch = "aarch64"))]
                pub fn set_timer(&mut self, _ticks: u64) {}

                #[verifier::external_body]
                #[cfg(target_arch = "aarch64")]
                pub fn read_timer(&self) -> u64 {
                    let val: u64;
                    unsafe {
                        core::arch::asm!(
                            "mrs {0}, cntvct_el0",
                            out(reg) val,
                        );
                    }
                    val
                }

                #[verifier::external_body]
                #[cfg(not(target_arch = "aarch64"))]
                pub fn read_timer(&self) -> u64 {
                    0
                }
            }
        }
    }

    pub mod riscv64 {
        use vstd::prelude::*;
        verus! {
            pub struct Sbi;

            impl Sbi {
                #[verifier::external_body]
                #[cfg(target_arch = "riscv64")]
                pub fn console_putchar(ch: usize) {
                    unsafe {
                        core::arch::asm!(
                            "ecall",
                            in("a7") 1,
                            in("a0") ch,
                            options(nostack)
                        );
                    }
                }

                #[verifier::external_body]
                #[cfg(not(target_arch = "riscv64"))]
                pub fn console_putchar(_ch: usize) {}
            }

            pub struct Uart {
                pub base_addr: usize,
            }

            impl Uart {
                pub fn new(base_addr: usize) -> (u: Self)
                    ensures u.base_addr == base_addr
                {
                    Uart { base_addr }
                }

                #[verifier::external_body]
                pub fn write_byte(&mut self, byte: u8) {
                    let ptr = self.base_addr as *mut u8;
                    unsafe {
                        core::ptr::write_volatile(ptr, byte);
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

    #[derive(Debug)]
    pub struct Gic {
        pub gicd_base: usize,
        pub gicc_base: usize,
    }

    impl Gic {
        pub fn new(gicd_base: usize, gicc_base: usize) -> Self {
            Gic {
                gicd_base,
                gicc_base,
            }
        }

        pub fn enable(&mut self, irq: u32) {
            let ptr = (self.gicd_base + 0x100 + ((irq / 32) * 4) as usize) as *mut u32;
            unsafe {
                core::ptr::write_volatile(ptr, 1 << (irq % 32));
            }
        }

        pub fn ack(&mut self) -> u32 {
            let ptr = (self.gicc_base + 0x0C) as *mut u32;
            unsafe { core::ptr::read_volatile(ptr) }
        }

        pub fn eoi(&mut self, irq: u32) {
            let ptr = (self.gicc_base + 0x10) as *mut u32;
            unsafe {
                core::ptr::write_volatile(ptr, irq);
            }
        }
    }

    pub struct GenericTimer;

    impl Default for GenericTimer {
        fn default() -> Self {
            Self::new()
        }
    }

    impl GenericTimer {
        pub fn new() -> Self {
            GenericTimer {}
        }

        #[cfg(target_arch = "aarch64")]
        pub fn enable(&mut self) {
            unsafe {
                core::arch::asm!(
                    "msr cntv_ctl_el0, {0}",
                    in(reg) 1u64,
                );
            }
        }

        #[cfg(not(target_arch = "aarch64"))]
        pub fn enable(&mut self) {}

        #[cfg(target_arch = "aarch64")]
        pub fn set_timer(&mut self, ticks: u64) {
            unsafe {
                core::arch::asm!(
                    "msr cntv_tval_el0, {0}",
                    in(reg) ticks,
                );
            }
        }

        #[cfg(not(target_arch = "aarch64"))]
        pub fn set_timer(&mut self, _ticks: u64) {}

        #[cfg(target_arch = "aarch64")]
        pub fn read_timer(&self) -> u64 {
            let val: u64;
            unsafe {
                core::arch::asm!(
                    "mrs {0}, cntvct_el0",
                    out(reg) val,
                );
            }
            val
        }

        #[cfg(not(target_arch = "aarch64"))]
        pub fn read_timer(&self) -> u64 {
            0
        }
    }
}

#[cfg(not(feature = "verus"))]
pub mod riscv64 {
    pub struct Sbi;

    impl Sbi {
        #[cfg(target_arch = "riscv64")]
        pub fn console_putchar(ch: usize) {
            unsafe {
                core::arch::asm!(
                    "ecall",
                    in("a7") 1,
                    in("a0") ch,
                    options(nostack)
                );
            }
        }

        #[cfg(not(target_arch = "riscv64"))]
        pub fn console_putchar(_ch: usize) {}
    }

    #[derive(Debug)]
    pub struct Uart {
        pub base_addr: usize,
    }

    impl Uart {
        pub fn new(base_addr: usize) -> Self {
            Uart { base_addr }
        }

        pub fn write_byte(&mut self, byte: u8) {
            let ptr = self.base_addr as *mut u8;
            unsafe {
                core::ptr::write_volatile(ptr, byte);
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

        uart.write_byte(b'A');
        assert_eq!(mmio_region[0], b'A' as u32);
    }

    #[test]
    fn test_riscv64_uart_write() {
        let mut mmio_region = [0u8; 8];
        let base_addr = mmio_region.as_mut_ptr() as usize;
        let mut uart = super::riscv64::Uart::new(base_addr);

        assert_eq!(uart.base_addr, base_addr);

        uart.write_byte(b'R');
        assert_eq!(mmio_region[0], b'R');
    }

    #[test]
    fn test_riscv64_sbi() {
        // Just call it, on non-riscv64 it's a no-op
        super::riscv64::Sbi::console_putchar(b'R' as usize);
    }
}
