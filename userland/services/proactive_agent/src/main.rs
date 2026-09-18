#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(target_arch = "wasm32", no_main)]

#[cfg(target_arch = "wasm32")]
extern crate alloc;

use proactive_agent::ProactiveAgent;

#[cfg(target_arch = "wasm32")]
use core::alloc::{GlobalAlloc, Layout};

#[cfg(target_arch = "wasm32")]
struct DummyAllocator;

#[cfg(target_arch = "wasm32")]
unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: DummyAllocator = DummyAllocator;

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut agent = ProactiveAgent::new();

    let _ = agent.init();

    #[allow(clippy::never_loop)]
    loop {
        let mock_system_load = 50;
        let mock_user_activity = 10;

        let _ = agent.analyze_and_act(mock_system_load, mock_user_activity);

        break;
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    let mut agent = ProactiveAgent::new();

    let _ = agent.init();

    #[allow(clippy::never_loop)]
    loop {
        let mock_system_load = 50;
        let mock_user_activity = 10;

        let _ = agent.analyze_and_act(mock_system_load, mock_user_activity);

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
