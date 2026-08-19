#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(target_arch = "wasm32", no_main)]

#[cfg(target_arch = "wasm32")]
extern crate alloc;

use app_preloader::AppPreloader;

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
    let mut preloader = AppPreloader::new();
    let _ = preloader.init();

    #[allow(clippy::never_loop)]
    loop {
        let mock_time = 9;
        let mock_last_app = 3;

        let _ = preloader.predict_and_preload(mock_time, mock_last_app);

        break;
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    let mut preloader = AppPreloader::new();
    let _ = preloader.init();

    #[allow(clippy::never_loop)]
    loop {
        let mock_time = 9;
        let mock_last_app = 3;

        let _ = preloader.predict_and_preload(mock_time, mock_last_app);

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
