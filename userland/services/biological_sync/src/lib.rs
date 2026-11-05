#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use inference_runtime::{InferenceEngine, Model, Tensor};

#[cfg(not(target_arch = "wasm32"))]
extern crate std;

#[cfg(target_arch = "wasm32")]
use core::alloc::{GlobalAlloc, Layout};

#[cfg(target_arch = "wasm32")]
struct BumpAllocator {
    heap: core::cell::UnsafeCell<[u8; 65536 * 16]>,
    bump_ptr: core::cell::UnsafeCell<usize>,
}

#[cfg(target_arch = "wasm32")]
unsafe impl Sync for BumpAllocator {}

#[cfg(target_arch = "wasm32")]
unsafe impl GlobalAlloc for BumpAllocator {
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
static ALLOCATOR: BumpAllocator = BumpAllocator {
    heap: core::cell::UnsafeCell::new([0; 65536 * 16]),
    bump_ptr: core::cell::UnsafeCell::new(0),
};

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    pub fn sys_get_biological_metrics(out_ptr: *mut u8, max_len: i32) -> i32;
    pub fn sys_set_ambient_environment(settings_ptr: *const u8, len: i32) -> i32;
}

pub struct BiologicalSyncManager {
    engine: InferenceEngine,
    model: Option<Model>,
    context: Option<usize>,
}

impl Default for BiologicalSyncManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BiologicalSyncManager {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            model: None,
            context: None,
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        let model = self
            .engine
            .load_model_by_name("biological_sync_model")
            .map_err(|_| "Failed to load model")?;
        let ctx = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init execution context")?;
        self.model = Some(model);
        self.context = Some(ctx);
        Ok(())
    }

    pub fn sync_environment(&mut self) -> Result<String, &'static str> {
        let mut metrics_buf = [0u8; 512];
        let bytes_read = {
            #[cfg(all(not(test), target_arch = "wasm32"))]
            unsafe {
                sys_get_biological_metrics(metrics_buf.as_mut_ptr(), metrics_buf.len() as i32)
            }
            #[cfg(any(test, not(target_arch = "wasm32")))]
            {
                let mock_metrics =
                    b"heart_rate: 60, circadian_phase: late_evening, stress_level: low";
                metrics_buf[..mock_metrics.len()].copy_from_slice(mock_metrics);
                mock_metrics.len() as i32
            }
        };

        if bytes_read <= 0 {
            return Err("Failed to read biological metrics");
        }

        let metrics_str = core::str::from_utf8(&metrics_buf[..bytes_read as usize])
            .map_err(|_| "Invalid UTF-8 in biological metrics")?;

        let ctx = self.context.ok_or("Manager not initialized")?;

        let prompt = format!("Determine optimal ambient environment settings (lighting, temperature, acoustics) for metrics: {}", metrics_str);
        let input_tensor = Tensor::new(prompt.into_bytes(), alloc::vec![1]);

        self.engine
            .set_input(ctx, 0, &input_tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 1024];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        if bytes_written == 0 {
            return Err("Model produced empty output");
        }

        let apply_res = {
            #[cfg(all(not(test), target_arch = "wasm32"))]
            unsafe {
                sys_set_ambient_environment(out.as_ptr(), bytes_written as i32)
            }
            #[cfg(any(test, not(target_arch = "wasm32")))]
            0
        };

        if apply_res != 0 {
            return Err("Failed to set ambient environment");
        }

        Ok(String::from(
            "Successfully synchronized ambient environment to biological metrics",
        ))
    }
}

#[cfg(any(test, not(target_arch = "wasm32")))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_manager_init() {
        let mut mgr = BiologicalSyncManager::new();
        assert!(mgr.init().is_ok());
        assert!(mgr.model.is_some());
    }

    #[test]
    fn test_sync_environment() {
        let mut mgr = BiologicalSyncManager::new();
        mgr.init().unwrap();

        let res = mgr.sync_environment().unwrap();
        assert!(res.contains("Successfully synchronized"));
    }
}
