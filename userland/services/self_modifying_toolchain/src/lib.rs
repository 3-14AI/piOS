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
    pub fn sys_get_hardware_constraints(out_ptr: *mut u8, max_len: i32) -> i32;
    pub fn sys_deploy_optimized_toolchain(wasm_ptr: *const u8, len: i32) -> i32;
}

pub struct ToolchainOptimizer {
    engine: InferenceEngine,
    model: Option<Model>,
    context: Option<usize>,
}

impl Default for ToolchainOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolchainOptimizer {
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
            .load_model_by_name("toolchain_optimizer_model")
            .map_err(|_| "Failed to load toolchain optimizer model")?;
        let ctx = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init execution context")?;
        self.model = Some(model);
        self.context = Some(ctx);
        Ok(())
    }

    pub fn optimize_and_deploy(&mut self) -> Result<String, &'static str> {
        let mut hw_constraints = [0u8; 512];
        let bytes_read = {
            #[cfg(all(not(test), target_arch = "wasm32"))]
            unsafe {
                sys_get_hardware_constraints(
                    hw_constraints.as_mut_ptr(),
                    hw_constraints.len() as i32,
                )
            }
            #[cfg(any(test, not(target_arch = "wasm32")))]
            {
                let mock_data = b"CPU: x86_64, AVX512: true, RAM: 16GB";
                hw_constraints[..mock_data.len()].copy_from_slice(mock_data);
                mock_data.len() as i32
            }
        };

        if bytes_read <= 0 {
            return Err("Failed to read hardware constraints");
        }

        let constraints_str = core::str::from_utf8(&hw_constraints[..bytes_read as usize])
            .map_err(|_| "Invalid UTF-8 in hardware constraints")?;

        let ctx = self.context.ok_or("Optimizer not initialized")?;

        let prompt = format!(
            "Optimize rustc codegen for constraints: {}",
            constraints_str
        );
        let input_tensor = Tensor::new(prompt.into_bytes(), alloc::vec![1]);

        self.engine
            .set_input(ctx, 0, &input_tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 4096];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        if bytes_written == 0 {
            return Err("Model produced empty output");
        }

        let deploy_res = {
            #[cfg(all(not(test), target_arch = "wasm32"))]
            unsafe {
                sys_deploy_optimized_toolchain(out.as_ptr(), bytes_written as i32)
            }
            #[cfg(any(test, not(target_arch = "wasm32")))]
            0
        };

        if deploy_res != 0 {
            return Err("Failed to deploy optimized toolchain");
        }

        Ok(String::from(
            "Successfully generated and deployed optimized toolchain",
        ))
    }
}

#[cfg(any(test, not(target_arch = "wasm32")))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_init() {
        let mut opt = ToolchainOptimizer::new();
        assert!(opt.init().is_ok());
        assert!(opt.model.is_some());
    }

    #[test]
    fn test_optimize_and_deploy() {
        let mut opt = ToolchainOptimizer::new();
        opt.init().unwrap();

        let res = opt.optimize_and_deploy().unwrap();
        assert!(res.contains("Successfully generated and deployed"));
    }
}
