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
