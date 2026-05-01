#![no_std]
#![allow(clippy::empty_loop)]

extern crate alloc;

use alloc::string::String;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(test))]
use core::alloc::{GlobalAlloc, Layout};

#[cfg(not(test))]
struct DummyAllocator;

#[cfg(not(test))]
unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: DummyAllocator = DummyAllocator;

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
    let _message = String::from("Hello, World from piOS!");
    // In a real WASM app, we would use a system call here to print
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_does_not_panic() {
        run();
    }
}
