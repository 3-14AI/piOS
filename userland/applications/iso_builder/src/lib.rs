#![cfg_attr(not(test), no_std)]
#![allow(clippy::empty_loop)]

extern crate alloc;

#[cfg(not(target_arch = "wasm32"))]
extern crate std;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(target_arch = "wasm32")]
use core::alloc::{GlobalAlloc, Layout};

#[cfg(target_arch = "wasm32")]
struct SimpleAllocator {
    heap: core::cell::UnsafeCell<[u8; 65536]>,
    bump_ptr: core::cell::UnsafeCell<usize>,
}

#[cfg(target_arch = "wasm32")]
unsafe impl Sync for SimpleAllocator {}

#[cfg(target_arch = "wasm32")]
unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let bump_ptr = self.bump_ptr.get();
        let heap = self.heap.get();

        let align_offset = (*bump_ptr).wrapping_add(layout.align() - 1) & !(layout.align() - 1);

        if align_offset + layout.size() > (*heap).len() {
            return core::ptr::null_mut(); // Out of memory
        }

        let ptr = (*heap).as_mut_ptr().add(align_offset);
        *bump_ptr = align_offset + layout.size();
        ptr
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {} // Memory leak by design
}

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    heap: core::cell::UnsafeCell::new([0; 65536]),
    bump_ptr: core::cell::UnsafeCell::new(0),
};

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn main() -> i32 {
    let args_vec = alloc::vec!["build".to_string()]; // fallback

    match run(args_vec) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

pub fn run(args: Vec<String>) -> Result<String, String> {
    if args.is_empty() {
        return Err("No arguments provided".to_string());
    }

    let command = args[0].as_str();

    match command {
        "build" => {
            // Mock the process of creating a signed bootable ISO image
            let mut output = String::new();
            output.push_str("Starting mkisofs-based toolchain mock...\n");
            output.push_str("1. Creating EFI System Partition (ESP)...\n");
            output.push_str("2. Copying kernel payload (kernel.efi)...\n");
            output.push_str("3. Signing UEFI bootloader for Secure Boot...\n");
            output.push_str("4. Packing into El Torito bootable ISO image...\n");
            output.push_str("Bootable ISO built successfully");
            Ok(output)
        }
        _ => Err("Unknown command".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iso_builder_no_args() {
        let args = alloc::vec![];
        let result = run(args);
        assert_eq!(result, Err("No arguments provided".to_string()));
    }

    #[test]
    fn test_iso_builder_success() {
        let args = alloc::vec!["build".to_string()];
        let result = run(args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Bootable ISO built successfully"));
    }
}
