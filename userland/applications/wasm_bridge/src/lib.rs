#![allow(clippy::missing_safety_doc)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

#[no_mangle]
pub unsafe extern "C" fn wasm_bridge_init() -> i32 {
    0
}

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
