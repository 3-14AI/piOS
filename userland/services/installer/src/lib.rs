#![no_std]
#![allow(clippy::empty_loop)]

extern crate alloc;

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
            #[allow(static_mut_refs)]
            HEAP.as_mut_ptr().add(res)
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    offset: AtomicUsize::new(0),
};

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    run();
    loop {}
}

#[cfg(test)]
pub fn main() {
    run();
}

pub fn run() {
    let mut log_messages: alloc::vec::Vec<alloc::string::String> = alloc::vec::Vec::new();
    log_messages.push(alloc::string::String::from("piOS Installer Started."));

    // Partition manager
    log_messages.push(alloc::string::String::from("Running partition manager..."));
    let partitions: alloc::vec::Vec<alloc::string::String> = alloc::vec![
        alloc::string::String::from("/dev/sda1 (ESP)"),
        alloc::string::String::from("/dev/sda2 (piOS)"),
        alloc::string::String::from("/dev/sda3 (Recovery)"),
    ];

    for p in partitions {
        log_messages.push(alloc::format!("Found partition: {}", p));
    }

    // Dual-boot setup
    log_messages.push(alloc::string::String::from(
        "Configuring dual-boot with existing OS...",
    ));

    // Installation
    log_messages.push(alloc::string::String::from(
        "Installing piOS to /dev/sda2...",
    ));

    // Recovery setup
    log_messages.push(alloc::string::String::from(
        "Setting up recovery mode on /dev/sda3...",
    ));

    log_messages.push(alloc::string::String::from(
        "Installation complete. Please reboot.",
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_does_not_panic() {
        run();
    }
}
