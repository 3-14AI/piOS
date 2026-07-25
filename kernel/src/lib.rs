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
        pub initrd_addr: usize,
        pub initrd_size: usize,
    }
}

#[cfg(not(feature = "verus"))]
pub mod verifier {
    pub fn kernel_main(boot_info: &crate::boot::BootInfo) {
        // No-op for non-verus build
        if boot_info.initrd_size > 0 && boot_info.initrd_addr > 0 {
            parse_initramfs(boot_info.initrd_addr, boot_info.initrd_size);
        }
    }

    pub fn parse_initramfs(addr: usize, size: usize) {
        let mut current = addr as *const u8;
        let end = (addr + size) as *const u8;

        unsafe {
            while current.add(110) <= end {
                // Check magic "070701"
                let mut magic_ok = true;
                let magic = b"070701";
                for i in 0..6 {
                    if *current.add(i) != magic[i] {
                        magic_ok = false;
                        break;
                    }
                }
                if !magic_ok {
                    break;
                }

                // Parse namesize (8 hex chars starting at offset 94)
                let mut namesize: usize = 0;
                for i in 0..8 {
                    let c = *current.add(94 + i);
                    let val = if c >= b'0' && c <= b'9' {
                        c - b'0'
                    } else if c >= b'A' && c <= b'F' {
                        c - b'A' + 10
                    } else if c >= b'a' && c <= b'f' {
                        c - b'a' + 10
                    } else {
                        0
                    };
                    namesize = (namesize << 4) | (val as usize);
                }

                // Parse filesize (8 hex chars starting at offset 54)
                let mut filesize: usize = 0;
                for i in 0..8 {
                    let c = *current.add(54 + i);
                    let val = if c >= b'0' && c <= b'9' {
                        c - b'0'
                    } else if c >= b'A' && c <= b'F' {
                        c - b'A' + 10
                    } else if c >= b'a' && c <= b'f' {
                        c - b'a' + 10
                    } else {
                        0
                    };
                    filesize = (filesize << 4) | (val as usize);
                }

                let name_ptr = current.add(110);

                // Check TRAILER!!!
                if namesize == 11 {
                    let trailer = b"TRAILER!!!\0";
                    let mut is_trailer = true;
                    for i in 0..11 {
                        if *name_ptr.add(i) != trailer[i] {
                            is_trailer = false;
                            break;
                        }
                    }
                    if is_trailer {
                        break;
                    }
                }

                let name_padding = (4 - ((110 + namesize) % 4)) % 4;
                let file_ptr = name_ptr.add(namesize + name_padding);

                let file_padding = (4 - (filesize % 4)) % 4;
                let next_current = file_ptr.add(filesize + file_padding);

                if next_current > end {
                    break;
                }

                // Check if name matches condition
                let mut add_entry = false;
                if filesize > 0 {
                    add_entry = true;
                }
                // Simple check for "init" prefix or ".so" suffix
                if namesize >= 4 {
                    if *name_ptr == b'i'
                        && *name_ptr.add(1) == b'n'
                        && *name_ptr.add(2) == b'i'
                        && *name_ptr.add(3) == b't'
                    {
                        add_entry = true;
                    }
                    if *name_ptr.add(namesize - 4) == b'.'
                        && *name_ptr.add(namesize - 3) == b's'
                        && *name_ptr.add(namesize - 2) == b'o'
                        && *name_ptr.add(namesize - 1) == 0
                    {
                        add_entry = true;
                    }
                }

                if add_entry {
                    add_vfs_entry_stub(name_ptr, file_ptr, filesize);
                }

                current = next_current;
            }
        }
    }

    pub fn add_vfs_entry_stub(_name: *const u8, _data: *const u8, _size: usize) {}
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
            initrd_addr: 0,
            initrd_size: 0,
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
            initrd_addr: 0,
            initrd_size: 0,
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
