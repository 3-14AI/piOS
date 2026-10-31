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

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: pios_common::DummyAllocator = pios_common::DummyAllocator;
