#![no_std]
#![allow(unused_imports)]

#[cfg(feature = "verus")]
use vstd::prelude::*;

#[cfg(feature = "verus")]
pub mod boot;

#[cfg(feature = "verus")]
pub mod verifier;

#[cfg(feature = "verus")]
pub mod pmm;

#[cfg(feature = "verus")]
pub mod paging;

#[cfg(feature = "verus")]
pub mod allocator;

#[cfg(feature = "verus")]
pub mod sync;

#[cfg(feature = "verus")]
pub mod thread;

#[cfg(feature = "verus")]
pub mod scheduler;

pub mod ipc;

#[cfg(feature = "verus")]
pub mod capabilities;

#[cfg(feature = "verus")]
pub mod virtio_blk;

#[cfg(feature = "verus")]
pub mod virtio_net;

#[cfg(feature = "verus")]
pub mod virtio_gpu;

#[cfg(feature = "verus")]
pub mod acpi;
#[cfg(feature = "verus")]
pub mod ahci;
#[cfg(feature = "verus")]
pub mod gpu;
#[cfg(feature = "verus")]
pub mod input;
#[cfg(feature = "verus")]
pub mod nvme;
#[cfg(feature = "verus")]
pub mod power;
#[cfg(feature = "verus")]
pub mod sound;
#[cfg(feature = "verus")]
pub mod usb;
#[cfg(feature = "verus")]
pub mod usb_hub;

#[cfg(feature = "verus")]
pub mod ehci;
#[cfg(feature = "verus")]
pub mod wifi;
#[cfg(feature = "verus")]
pub mod xhci;

#[cfg(feature = "verus")]
pub mod interrupts;

#[cfg(feature = "verus")]
pub mod wasm;

pub mod dma;

#[cfg(feature = "verus")]
pub mod vfs;

#[cfg(feature = "verus")]
pub mod guardrails;

#[cfg(not(feature = "verus"))]
pub mod boot {
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct SimpleMemoryDescriptor {
        pub type_: u32,
        pub pad: u32,
        pub phys_start: u64,
        pub virt_start: u64,
        pub page_count: u64,
        pub attribute: u64,
    }

    #[repr(C)]
    pub struct BootInfo {
        pub memory_map: *mut u8,
        pub memory_map_len: usize,
        pub descriptor_size: usize,
        pub descriptor_version: u32,
    }
}

#[cfg(not(feature = "verus"))]
pub mod verifier {
    pub fn kernel_main(_boot_info: &crate::boot::BootInfo) {
        // No-op for non-verus build
    }
}

#[cfg(not(feature = "verus"))]
pub mod wasm;

#[cfg(not(feature = "verus"))]
pub mod capabilities;

#[cfg(not(feature = "verus"))]
pub mod virtio_blk;

#[cfg(not(feature = "verus"))]
pub mod virtio_net;

#[cfg(not(feature = "verus"))]
pub mod virtio_gpu;

#[cfg(not(feature = "verus"))]
pub mod acpi;
#[cfg(not(feature = "verus"))]
pub mod ahci;
#[cfg(not(feature = "verus"))]
pub mod gpu;
#[cfg(not(feature = "verus"))]
pub mod input;
#[cfg(not(feature = "verus"))]
pub mod nvme;
#[cfg(not(feature = "verus"))]
pub mod power;
#[cfg(not(feature = "verus"))]
pub mod sound;
#[cfg(not(feature = "verus"))]
pub mod usb;
#[cfg(not(feature = "verus"))]
pub mod usb_hub;

#[cfg(not(feature = "verus"))]
pub mod ehci;
#[cfg(not(feature = "verus"))]
pub mod wifi;
#[cfg(not(feature = "verus"))]
pub mod xhci;

#[cfg(not(feature = "verus"))]
pub mod vfs;

#[cfg(not(feature = "verus"))]
pub mod guardrails;

#[cfg(not(feature = "verus"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_info() {
        let _info = boot::BootInfo {
            memory_map: core::ptr::null_mut(),
            memory_map_len: 0,
            descriptor_size: 0,
            descriptor_version: 0,
        };
        assert_eq!(_info.memory_map_len, 0);

        let desc = boot::SimpleMemoryDescriptor {
            type_: 0,
            pad: 0,
            phys_start: 0,
            virt_start: 0,
            page_count: 0,
            attribute: 0,
        };
        assert_eq!(desc.page_count, 0);
    }

    #[test]
    fn test_verifier_kernel_main() {
        let info = boot::BootInfo {
            memory_map: core::ptr::null_mut(),
            memory_map_len: 0,
            descriptor_size: 0,
            descriptor_version: 0,
        };
        verifier::kernel_main(&info);
    }
}

#[cfg(feature = "verus")]
pub mod pci;

#[cfg(feature = "verus")]
pub mod arch;

#[cfg(not(feature = "verus"))]
pub mod arch;

#[cfg(not(feature = "verus"))]
pub mod pci;

pub mod blue_green;
pub mod co_generation;
pub mod driver_pipeline;
pub mod hot_reload;

#[cfg(feature = "verus")]
pub mod telemetry;

#[cfg(not(feature = "verus"))]
pub mod telemetry;
