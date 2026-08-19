#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(target_arch = "wasm32", no_main)]

#[cfg(target_arch = "wasm32")]
extern crate alloc;

use driver_synthesizer::DriverSynthesizer;

use core::alloc::{GlobalAlloc, Layout};

struct DummyAllocator;

unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOCATOR: DummyAllocator = DummyAllocator;

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut synth = DriverSynthesizer::new();
    let _ = synth.init();

    #[allow(clippy::never_loop)]
    loop {
        let _ = synth.synthesize_and_load("mock_hw");
        break;
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    let mut synth = DriverSynthesizer::new();
    let _ = synth.init();

    #[allow(clippy::never_loop)]
    loop {
        let _ = synth.synthesize_and_load("mock_hw");
        break;
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

#[cfg(target_arch = "wasm32")]
#[cfg(not(test))]
#[panic_handler]
#[allow(clippy::empty_loop)]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
