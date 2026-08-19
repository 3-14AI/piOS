#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(target_arch = "wasm32", no_main)]

#[cfg(target_arch = "wasm32")]
extern crate alloc;

use ids_ips::IdsIps;

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
    let mut ids = IdsIps::new();
    let _ = ids.init();

    #[allow(clippy::never_loop)]
    loop {
        // Mock system behavior checking
        let mock_syscall = 1;
        let mock_arg1 = 2;
        let mock_arg2 = 3;

        let _ = ids.analyze_and_block(mock_syscall, mock_arg1, mock_arg2);

        break;
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    let mut ids = IdsIps::new();
    let _ = ids.init();

    #[allow(clippy::never_loop)]
    loop {
        let mock_syscall = 1;
        let mock_arg1 = 2;
        let mock_arg2 = 3;

        let _ = ids.analyze_and_block(mock_syscall, mock_arg1, mock_arg2);

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
