#![no_std]
#![no_main]

extern crate alloc;

use sys_optimizer::SysOptimizer;

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

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut optimizer = SysOptimizer::new();

    // In a real WASM environment, initialization might be provided.
    // For now, we attempt to initialize it.
    let _ = optimizer.init();

    #[allow(clippy::never_loop)]
    loop {
        // Mock system metric scraping. In reality this would call out to host functions
        // provided by the kernel (e.g. via WASI) to get real cpu_usage and mem_usage.
        let mock_cpu_usage = 50;
        let mock_mem_usage = 50;

        let _ = optimizer.analyze_and_adjust(mock_cpu_usage, mock_mem_usage);

        // Mock sleep. In a real environment this would use WASI poll_oneoff or similar to
        // sleep and continuously monitor.

        // Break out of the loop for this mock execution to prevent infinite loop during testing if executed
        break;
    }

    // Exit sequence
    #[allow(clippy::empty_loop)]
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
#[allow(clippy::empty_loop)]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
