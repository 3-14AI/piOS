#![no_std]
#![allow(dead_code)]

extern crate alloc;
#[cfg(not(target_arch = "wasm32"))]
extern crate std;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

// Need to define WasmNetStack mock internally since the workspace structure is getting complex.
pub struct WasmNetStack;
impl WasmNetStack {
    pub fn new() -> Self {
        Self
    }
    pub fn mock_query_swarm(&self) -> Vec<UpdateManifest> {
        let mut updates = Vec::new();
        updates.push(UpdateManifest {
            version: "1.1.0".to_string(),
            download_url: "swarm://updates/1.1.0".to_string(),
            signature: alloc::vec![1, 2, 3, 4],
        });
        updates
    }
}

// Minimal mock for wasi_crypto verify
pub unsafe fn sys_crypto_verify(
    _patch_data: *const u8,
    _patch_len: u32,
    _sig_data: *const u8,
    _sig_len: u32,
) -> i32 {
    0 // Success mock
}

pub unsafe fn sys_ota_apply_update(_patch_data: *const u8, _patch_len: u32) -> i32 {
    0 // Success mock
}

#[no_mangle]
pub extern "C" fn start_ota_update_daemon() {
    let mut ota_mgr = OtaUpdateManager::new();
    ota_mgr.run();
}

pub struct OtaUpdateManager {
    net_stack: WasmNetStack,
    current_version: String,
}

impl OtaUpdateManager {
    pub fn new() -> Self {
        Self {
            net_stack: WasmNetStack::new(),
            current_version: "1.0.0".to_string(),
        }
    }

    pub fn run(&mut self) {
        // Query swarm for updates
        let updates = self.query_updates();
        if !updates.is_empty() {
            // Find highest version update
            let mut highest_version: Option<UpdateManifest> = None;
            for update in &updates {
                if let Some(ref current_highest) = highest_version {
                    if Self::version_greater(&update.version, &current_highest.version) {
                        highest_version = Some(update.clone());
                    }
                } else {
                    highest_version = Some(update.clone());
                }
            }

            if let Some(update) = highest_version {
                if Self::version_greater(&update.version, &self.current_version) {
                    if let Some(patch_data) = self.download_patch(&update.download_url) {
                        if self.verify_signature(&patch_data, &update.signature) {
                            self.apply_update(&patch_data);
                            self.current_version = update.version;
                        }
                    }
                }
            }
        }
    }

    fn version_greater(v1: &str, v2: &str) -> bool {
        let v1_parts: Vec<&str> = v1.split('.').collect();
        let v2_parts: Vec<&str> = v2.split('.').collect();

        for i in 0..core::cmp::min(v1_parts.len(), v2_parts.len()) {
            if let (Ok(num1), Ok(num2)) = (v1_parts[i].parse::<u32>(), v2_parts[i].parse::<u32>()) {
                if num1 > num2 {
                    return true;
                } else if num1 < num2 {
                    return false;
                }
            }
        }

        v1_parts.len() > v2_parts.len()
    }

    fn query_updates(&self) -> Vec<UpdateManifest> {
        self.net_stack.mock_query_swarm()
    }

    fn download_patch(&mut self, _url: &str) -> Option<Vec<u8>> {
        // Mock download via net_stack
        Some(alloc::vec![0x4F, 0x54, 0x41, 0x01])
    }

    fn verify_signature(&self, patch_data: &[u8], signature: &[u8]) -> bool {
        // Verify via `wasi_crypto` interface
        unsafe {
            sys_crypto_verify(
                patch_data.as_ptr(),
                patch_data.len() as u32,
                signature.as_ptr(),
                signature.len() as u32,
            ) == 0
        }
    }

    fn apply_update(&mut self, patch_data: &[u8]) {
        // Apply update to A/B partitioned rootfs natively
        unsafe {
            sys_ota_apply_update(patch_data.as_ptr(), patch_data.len() as u32);
        }
    }
}

#[derive(Clone)]
pub struct UpdateManifest {
    pub version: String,
    pub download_url: String,
    pub signature: Vec<u8>,
}

// Minimal global allocator
#[cfg(target_arch = "wasm32")]
mod allocator {
    use core::alloc::{GlobalAlloc, Layout};
    use core::ptr::{addr_of_mut, null_mut};
    use core::sync::atomic::{AtomicUsize, Ordering};

    struct BumpAllocator {
        heap_start: usize,
        heap_end: usize,
        next: AtomicUsize,
    }

    impl BumpAllocator {
        const fn new() -> Self {
            BumpAllocator {
                heap_start: 0,
                heap_end: 0,
                next: AtomicUsize::new(0),
            }
        }
    }

    unsafe impl GlobalAlloc for BumpAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            // Lazy initialization of heap addresses
            let mut current_next = self.next.load(Ordering::Acquire);
            if current_next == 0 {
                let ptr = addr_of_mut!(HEAP) as usize;
                match self
                    .next
                    .compare_exchange(0, ptr, Ordering::AcqRel, Ordering::Acquire)
                {
                    Ok(_) => current_next = ptr,
                    Err(val) => current_next = val,
                }
            }

            let align = layout.align();

            // Atomic loop for bump allocation
            loop {
                let mut offset = current_next % align;
                if offset != 0 {
                    offset = align - offset;
                }

                let alloc_start = current_next + offset;
                let heap_size = 16 * 1024 * 1024;
                let heap_end = addr_of_mut!(HEAP) as usize + heap_size;

                if alloc_start + layout.size() > heap_end {
                    return null_mut();
                }

                let next_val = alloc_start + layout.size();
                match self.next.compare_exchange_weak(
                    current_next,
                    next_val,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => return alloc_start as *mut u8,
                    Err(val) => current_next = val,
                }
            }
        }

        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
            // Bump allocator doesn't free
        }
    }

    // Static buffer for our heap (16MB)
    #[link_section = ".bss"]
    static mut HEAP: [u8; 16 * 1024 * 1024] = [0; 16 * 1024 * 1024];

    #[global_allocator]
    static ALLOCATOR: BumpAllocator = BumpAllocator::new();
}

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
