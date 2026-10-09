use vstd::prelude::*;

verus! {

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
    pub initrd_addr: usize,
    pub initrd_size: usize,
    pub fb_addr: u64,
    pub fb_size: u64,
    pub fb_width: u32,
    pub fb_height: u32,
    pub fb_stride: u32,
}

}
