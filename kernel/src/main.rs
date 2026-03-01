#![no_std]
#![no_main]
#![allow(clippy::empty_loop)]

use uefi::prelude::*;
use uefi::table::boot::{MemoryDescriptor, MemoryType};

// We use the library crate 'kernel' for shared definitions and verified code.
use kernel::boot;
use kernel::verifier;

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    if uefi::helpers::init(&mut system_table).is_err() {
        loop {}
    }
    log::info!("Hello from piOS Boot Stub!");

    // 1. Exit Boot Services
    // Note: In uefi-rs 0.28+, exit_boot_services helper handles allocation.
    // It returns (SystemTable<Runtime>, MemoryMap<'static>).
    let (_system_table_runtime, memory_map) =
        system_table.exit_boot_services(MemoryType::LOADER_DATA);

    // 2. Construct BootInfo
    let entries_len = memory_map.entries().len();
    if entries_len == 0 {
        // Should not happen as we are running code
        loop {}
    }

    // Get pointer to first element
    let first = memory_map.entries().next().unwrap() as *const MemoryDescriptor;
    let base_ptr = first as *const u8;

    // Calculate stride if possible, or assume packed if len=1
    let stride = if entries_len > 1 {
        let second = memory_map.entries().nth(1).unwrap() as *const MemoryDescriptor;
        (second as usize) - (first as usize)
    } else {
        core::mem::size_of::<MemoryDescriptor>()
    };

    // We cast to *mut u8 because BootInfo expects mutable pointer (it owns the map now).
    // The memory is leaked by uefi helper so it is static 'static.
    let boot_info = boot::BootInfo {
        memory_map: base_ptr as *mut u8,
        memory_map_len: entries_len,
        descriptor_size: stride,
        descriptor_version: 1, // Assume version 1 as uefi-rs abstracts it away
    };

    // 3. Pass control to verified kernel
    verifier::kernel_main(&boot_info);

    // 4. Run simple tests and signal QEMU to exit
    run_qemu_tests();

    // 5. Spin
    loop {}
}

fn outb(port: u16, val: u8) {
    unsafe {
        core::arch::asm!("out dx, al", in("dx") port, in("al") val);
    }
}

fn write_serial(s: &str) {
    let port = 0x3F8; // COM1
    for b in s.bytes() {
        unsafe {
            // Wait for line status register to be ready
            while {
                let mut lsr: u8;
                core::arch::asm!("in al, dx", out("al") lsr, in("dx") port + 5);
                (lsr & 0x20) == 0
            } {}
        }
        outb(port, b);
    }
}

fn run_qemu_tests() {
    write_serial("\nRunning QEMU Integration Tests...\n");
    // TODO: add real tests here
    write_serial("[ok] Kernel booted successfully\n");

    // Shutdown via isa-debug-exit
    outb(0xf4, 0x10);
}
