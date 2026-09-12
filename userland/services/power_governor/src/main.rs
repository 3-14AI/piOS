#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(target_arch = "wasm32", no_main)]

#[cfg(target_arch = "wasm32")]
extern crate alloc;

use power_governor::PowerGovernor;

#[cfg(target_arch = "wasm32")]
use core::alloc::{GlobalAlloc, Layout};

// Provide a real bump allocator for the WASM environment
// We'll use a static buffer for allocation since this is a no_std environment
#[cfg(target_arch = "wasm32")]
static mut HEAP: [u8; 1024 * 64] = [0; 1024 * 64];
#[cfg(target_arch = "wasm32")]
static mut HEAP_PTR: usize = 0;

#[cfg(target_arch = "wasm32")]
struct BumpAllocator;

#[cfg(target_arch = "wasm32")]
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let mut ptr = HEAP_PTR;

        // Align pointer
        let offset = ptr % align;
        if offset != 0 {
            ptr += align - offset;
        }

        if ptr + size > 1024 * 64 {
            return core::ptr::null_mut();
        }

        HEAP_PTR = ptr + size;
        core::ptr::addr_of_mut!(HEAP).cast::<u8>().add(ptr)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't free
    }
}

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut governor = PowerGovernor::new();

    let _ = governor.init();

    #[allow(clippy::never_loop)]
    loop {
        // Mock system metric scraping.
        let mock_cpu_usage = 45;
        let mock_battery_level = 80;
        let mock_time_of_day = 14;

        let _ = governor.analyze_and_adjust(mock_cpu_usage, mock_battery_level, mock_time_of_day);

        let _ = governor.fine_tune(
            "power_weights.bin",
            b"real_telemetry_based_power_weights_data",
        );

        break;
    }

    #[allow(clippy::empty_loop)]
    loop {}
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    let mut governor = PowerGovernor::new();

    let _ = governor.init();

    #[allow(clippy::never_loop)]
    loop {
        let mock_cpu_usage = 45;
        let mock_battery_level = 80;
        let mock_time_of_day = 14;

        let _ = governor.analyze_and_adjust(mock_cpu_usage, mock_battery_level, mock_time_of_day);

        let _ = governor.fine_tune(
            "power_weights.bin",
            b"real_telemetry_based_power_weights_data",
        );

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
