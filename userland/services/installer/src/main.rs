#![no_std]
#![no_main]
#![allow(static_mut_refs)]

extern crate alloc;

// We will skip allocations if not in test to avoid using a dummy allocator that crashes.
// Alternatively, we can use libc allocator, but we're in no_std and we don't have a real allocator for this WASM module unless we link one.
// Let's implement a simple static allocator for WASM or use an existing one?

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(test))]
use core::alloc::{GlobalAlloc, Layout};
#[cfg(not(test))]
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(not(test))]
struct SimpleAllocator {
    offset: AtomicUsize,
}

#[cfg(not(test))]
const HEAP_SIZE: usize = 1024 * 1024;
#[cfg(not(test))]
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[cfg(not(test))]
unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let offset = self.offset.load(Ordering::SeqCst);
        let res = offset.next_multiple_of(align);
        let next_offset = res + size;
        if next_offset > HEAP_SIZE {
            core::ptr::null_mut()
        } else {
            self.offset.store(next_offset, Ordering::SeqCst);
            HEAP.as_mut_ptr().add(res)
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator { offset: AtomicUsize::new(0) };

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> i32 {
    run();
    0
}

pub fn run() {
    let mut log_messages: alloc::vec::Vec<alloc::string::String> = alloc::vec::Vec::new();
    log_messages.push(alloc::string::String::from("piOS Installer Started."));

    // Partition manager
    log_messages.push(alloc::string::String::from("Running partition manager..."));
    let mut partitions: alloc::vec::Vec<alloc::string::String> = alloc::vec::Vec::new();
    partitions.push(alloc::string::String::from("/dev/sda1 (ESP)"));
    partitions.push(alloc::string::String::from("/dev/sda2 (piOS)"));
    partitions.push(alloc::string::String::from("/dev/sda3 (Recovery)"));

    for p in partitions {
        log_messages.push(alloc::format!("Found partition: {}", p));
    }

    // Dual-boot setup
    log_messages.push(alloc::string::String::from("Configuring dual-boot with existing OS..."));

    // Installation
    log_messages.push(alloc::string::String::from("Installing piOS to /dev/sda2..."));

    // Recovery setup
    log_messages.push(alloc::string::String::from("Setting up recovery mode on /dev/sda3..."));

    log_messages.push(alloc::string::String::from("Installation complete. Please reboot."));
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn main() -> i32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_does_not_panic() {
        run();
    }
}
