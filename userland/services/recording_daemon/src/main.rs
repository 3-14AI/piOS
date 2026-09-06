#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(target_arch = "wasm32", no_main)]

#[cfg(target_arch = "wasm32")]
extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};

#[cfg(target_arch = "wasm32")]
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(target_arch = "wasm32")]
const HEAP_SIZE: usize = 32 * 1024 * 1024;

#[cfg(target_arch = "wasm32")]
#[repr(C, align(4096))]
struct AlignedHeap([u8; HEAP_SIZE]);

#[cfg(target_arch = "wasm32")]
struct SyncHeap(core::cell::UnsafeCell<AlignedHeap>);

#[cfg(target_arch = "wasm32")]
unsafe impl Sync for SyncHeap {}

#[cfg(target_arch = "wasm32")]
static HEAP: SyncHeap = SyncHeap(core::cell::UnsafeCell::new(AlignedHeap([0; HEAP_SIZE])));

#[cfg(target_arch = "wasm32")]
struct SimpleAllocator {
    offset: AtomicUsize,
}

#[cfg(not(target_arch = "wasm32"))]
struct SimpleAllocator;

unsafe impl GlobalAlloc for SimpleAllocator {
    #[cfg(target_arch = "wasm32")]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();

        let mut current_offset = self.offset.load(Ordering::Acquire);
        loop {
            let res = current_offset.next_multiple_of(align);
            let next_offset = res + size;

            if next_offset > HEAP_SIZE {
                return core::ptr::null_mut();
            }

            match self.offset.compare_exchange_weak(
                current_offset,
                next_offset,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    let heap_ptr = HEAP.0.get() as *mut u8;
                    return heap_ptr.add(res);
                }
                Err(new_offset) => {
                    current_offset = new_offset;
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator {
    offset: AtomicUsize::new(0),
};

#[cfg(not(target_arch = "wasm32"))]
#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator;

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn start() -> ! {
    let mut daemon = recording_daemon::RecordingDaemon::new();
    daemon.start_recording();

    // In a real OS, this would hook into input subsystems and display compositors,
    // intercepting frames and keypresses. For WP-148 completion we spin in a loop
    // representing the daemon's active lifecycle running in the background.
    loop {
        // Sleep or wait for IPC signals
        core::hint::spin_loop();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    let mut daemon = recording_daemon::RecordingDaemon::new();
    daemon.start_recording();

    // Not targeting WASM, typically tests or CI workspace build
}
