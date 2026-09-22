#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(target_arch = "wasm32", no_main)]

#[cfg(target_arch = "wasm32")]
extern crate alloc;

use self_hosting_llm::LlmApiService;

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
pub extern "C" fn _start() -> ! {
    let mut llm_service = LlmApiService::new();

    let _ = llm_service.init("system_llm_model");

    #[allow(clippy::never_loop)]
    loop {
        // Mock receiving request from network stack or IPC
        let mock_request = b"mock request";

        let _response = llm_service.handle_request(mock_request);

        // Break out of the loop for this mock execution to prevent infinite loop during testing if executed
        break;
    }

    // Exit sequence
    #[allow(clippy::empty_loop)]
    loop {}
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    let mut llm_service = LlmApiService::new();

    let _ = llm_service.init("system_llm_model");

    #[allow(clippy::never_loop)]
    loop {
        // Mock receiving request from network stack or IPC
        let mock_request = b"mock request";

        let _response = llm_service.handle_request(mock_request);

        // Break out of the loop for this mock execution to prevent infinite loop during testing if executed
        break;
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg(not(test))]
#[panic_handler]
#[allow(clippy::empty_loop)]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
