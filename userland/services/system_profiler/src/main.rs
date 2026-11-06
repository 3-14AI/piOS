#![cfg_attr(target_arch = "wasm32", no_std)]
#![cfg_attr(target_arch = "wasm32", no_main)]

#[cfg(target_arch = "wasm32")]
extern crate alloc;

use system_profiler::SystemProfiler;

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

struct BumpAllocator {
    offset: AtomicUsize,
}

const HEAP_SIZE: usize = 32 * 1024 * 1024;
#[repr(C, align(4096))]
struct AlignedHeap([u8; HEAP_SIZE]);

struct SyncHeap(core::cell::UnsafeCell<AlignedHeap>);

unsafe impl Sync for SyncHeap {}

static HEAP: SyncHeap = SyncHeap(core::cell::UnsafeCell::new(AlignedHeap([0; HEAP_SIZE])));

unsafe impl GlobalAlloc for BumpAllocator {
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
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator {
    offset: AtomicUsize::new(0),
};

#[cfg(target_arch = "wasm32")]
#[repr(C)]
struct SubscriptionClock {
    id: u64,
    timeout: u64,
    precision: u64,
    flags: u16,
    // Add padding to match the C union alignment which requires 8 byte alignment for u64
    // 8 + 8 + 8 + 2 = 26 bytes. Next 8-byte boundary is 32, so 6 bytes of padding.
    _padding: [u8; 6],
}

#[cfg(target_arch = "wasm32")]
#[repr(C)]
struct Subscription {
    userdata: u64,
    type_: u8,
    // Add padding to align the union
    _padding: [u8; 7],
    u: SubscriptionU,
}

#[cfg(target_arch = "wasm32")]
#[repr(C)]
union SubscriptionU {
    clock: SubscriptionClock,
}

#[cfg(target_arch = "wasm32")]
#[repr(C)]
struct Event {
    userdata: u64,
    error: u16,
    type_: u8,
    // Add padding to align fd_readwrite
    _padding: [u8; 5],
    fd_readwrite: EventFdReadwrite,
}

#[cfg(target_arch = "wasm32")]
#[repr(C)]
struct EventFdReadwrite {
    nbytes: u64,
    flags: u16,
    _padding: [u8; 6],
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    fn poll_oneoff(
        in_: *const Subscription,
        out: *mut Event,
        nsubscriptions: usize,
        nevents: *mut usize,
    ) -> u16;
}

fn sleep_ms(ms: u64) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        let clock = SubscriptionClock {
            id: 0, // CLOCKID_REALTIME
            timeout: ms * 1_000_000,
            precision: 0,
            flags: 0, // 0 means relative time
            _padding: [0; 6],
        };
        let sub = Subscription {
            userdata: 0,
            type_: 0, // EVENTTYPE_CLOCK
            _padding: [0; 7],
            u: SubscriptionU { clock },
        };
        let mut event = Event {
            userdata: 0,
            error: 0,
            type_: 0,
            _padding: [0; 5],
            fd_readwrite: EventFdReadwrite {
                nbytes: 0,
                flags: 0,
                _padding: [0; 6],
            },
        };
        let mut nevents = 0;

        poll_oneoff(&sub, &mut event, 1, &mut nevents);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Mock sleep for host execution (e.g. tests)
        let _ = ms;
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut profiler = SystemProfiler::new();

    // In a real WASM environment, initialization might be provided.
    // For now, we attempt to initialize it.
    let _ = profiler.init();

    // Save offset after initialization to prevent corrupting inference models
    let base_offset = ALLOCATOR.offset.load(Ordering::Acquire);

    #[allow(clippy::empty_loop)]
    loop {
        // Since we are using a bump allocator that doesn't deallocate, we need to reset it
        // on each loop iteration to avoid an immediate OOM panic.
        ALLOCATOR.offset.store(base_offset, Ordering::Release);

        // Mock system metric scraping. In reality this would call out to host functions
        // provided by the kernel (e.g. via WASI) to get real cpu_usage, mem_usage, io_usage.
        let mock_cpu_usage = 45;
        let mock_mem_usage = 60;
        let mock_io_usage = 30;

        let _ = profiler.analyze_and_adjust(mock_cpu_usage, mock_mem_usage, mock_io_usage);

        // Sleep for 1 second (1000ms)
        sleep_ms(1000);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    let mut profiler = SystemProfiler::new();

    // In a real WASM environment, initialization might be provided.
    // For now, we attempt to initialize it.
    let _ = profiler.init();

    let base_offset = ALLOCATOR.offset.load(Ordering::Acquire);

    #[allow(clippy::never_loop)]
    loop {
        ALLOCATOR.offset.store(base_offset, Ordering::Release);

        // Mock system metric scraping. In reality this would call out to host functions
        // provided by the kernel (e.g. via WASI) to get real cpu_usage, mem_usage, io_usage.
        let mock_cpu_usage = 45;
        let mock_mem_usage = 60;
        let mock_io_usage = 30;

        let _ = profiler.analyze_and_adjust(mock_cpu_usage, mock_mem_usage, mock_io_usage);

        sleep_ms(1000);

        break; // For native mock testing, prevent infinite loop. In a real daemon this wouldn't break.
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg(not(test))]
#[panic_handler]
#[allow(clippy::empty_loop)]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
