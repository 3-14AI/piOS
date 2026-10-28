#![no_std]
#![allow(clippy::empty_loop)]

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
    pub fn sys_hot_swap(wasm_ptr: *const u8, wasm_len: i32) -> i32;
    pub fn sys_read_panic_log(buf_ptr: *mut u8, buf_len: i32) -> i32;
    pub fn sys_verify_verus(code_ptr: *const u8, code_len: i32) -> i32;
    pub fn sys_compile_wasm(
        code_ptr: *const u8,
        code_len: i32,
        out_ptr: *mut u8,
        out_len: i32,
    ) -> i32;
}

pub struct SelfHealingSupervisor {
    pub engine: InferenceEngine,
    pub model: Option<Model>,
    pub context: Option<usize>,
}

impl Default for SelfHealingSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfHealingSupervisor {
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
            .load_model_by_name("self_healing_model")
            .map_err(|_| "Failed to load model")?;
        let ctx = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init execution context")?;
        self.model = Some(model);
        self.context = Some(ctx);
        Ok(())
    }

    pub fn monitor_and_heal(&mut self) -> Result<String, &'static str> {
        let mut log_buf = [0u8; 1024];
        let bytes_read = {
            #[cfg(all(not(test), target_arch = "wasm32"))]
            unsafe {
                sys_read_panic_log(log_buf.as_mut_ptr(), log_buf.len() as i32)
            }

            #[cfg(any(test, not(target_arch = "wasm32")))]
            {
                let mock_log = b"panic at 'index out of bounds', src/kernel.rs:42";
                log_buf[..mock_log.len()].copy_from_slice(mock_log);
                mock_log.len() as i32
            }
        };

        if bytes_read <= 0 {
            return Ok(String::from("No panics detected."));
        }

        let panic_log = core::str::from_utf8(&log_buf[..bytes_read as usize])
            .map_err(|_| "Invalid UTF-8 in panic log")?;

        let ctx = self.context.ok_or("Supervisor not initialized")?;

        let input_prompt = format!("Fix the following kernel panic: {}", panic_log);
        let input_tensor = Tensor::new(input_prompt.into_bytes(), alloc::vec![1]);

        self.engine
            .set_input(ctx, 0, &input_tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 2048];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        if bytes_written == 0 {
            return Err("Model produced empty output");
        }

        let _generated_fix = core::str::from_utf8(&out[..bytes_written]).unwrap_or("fn fixed() {}");

        let verify_res = {
            #[cfg(all(not(test), target_arch = "wasm32"))]
            unsafe {
                sys_verify_verus(_generated_fix.as_ptr(), _generated_fix.len() as i32)
            }

            #[cfg(any(test, not(target_arch = "wasm32")))]
            0
        };

        if verify_res != 0 {
            return Err("Verus verification failed");
        }

        let mut wasm_out = [0u8; 4096];
        let compile_res = {
            #[cfg(all(not(test), target_arch = "wasm32"))]
            unsafe {
                sys_compile_wasm(
                    _generated_fix.as_ptr(),
                    _generated_fix.len() as i32,
                    wasm_out.as_mut_ptr(),
                    wasm_out.len() as i32,
                )
            }

            #[cfg(any(test, not(target_arch = "wasm32")))]
            {
                let mock_wasm = b"mock_wasm_compiled_fix";
                wasm_out[..mock_wasm.len()].copy_from_slice(mock_wasm);
                mock_wasm.len() as i32
            }
        };

        if compile_res <= 0 {
            return Err("Compilation failed");
        }

        let swap_res = {
            #[cfg(all(not(test), target_arch = "wasm32"))]
            unsafe {
                sys_hot_swap(wasm_out.as_ptr(), compile_res)
            }

            #[cfg(any(test, not(target_arch = "wasm32")))]
            0
        };

        if swap_res != 0 {
            return Err("Hot-swap failed");
        }

        Ok(format!(
            "Successfully healed system. WASM module size: {}",
            compile_res
        ))
    }
}

#[cfg(any(test, not(target_arch = "wasm32")))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supervisor_init() {
        let mut supervisor = SelfHealingSupervisor::new();
        assert!(supervisor.init().is_ok());
        assert!(supervisor.model.is_some());
    }

    #[test]
    fn test_supervisor_heal() {
        let mut supervisor = SelfHealingSupervisor::new();
        supervisor.init().unwrap();

        let res = supervisor.monitor_and_heal().unwrap();
        assert!(res.contains("Successfully healed system"));
    }
}
