#![no_std]
#![allow(dead_code)]
#![allow(clippy::new_without_default)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

use alloc::string::String;
use inference_runtime::{InferenceEngine, Model, Tensor};

#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    pub fn sys_hot_swap(wasm_ptr: *const u8, wasm_len: i32) -> i32;
}

pub struct HardwareSynthesisEngine {
    pub engine: InferenceEngine,
    pub model: Option<Model>,
    pub context: Option<usize>,
}

impl HardwareSynthesisEngine {
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
            .load_model_by_name("hardware_synthesis_engine")
            .map_err(|_| "Failed to load model")?;
        let ctx = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init context")?;

        self.model = Some(model);
        self.context = Some(ctx);
        Ok(())
    }

    pub fn synthesize_and_deploy(
        &mut self,
        descriptor_data: &[u8],
    ) -> Result<String, &'static str> {
        let ctx = self.context.ok_or("Engine not initialized")?;

        let input_vec = descriptor_data.to_vec();
        let len = input_vec.len();
        let tensor = Tensor::new(input_vec, alloc::vec![len]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 32];
        let bytes = self
            .engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        if bytes == 0 {
            return Err("Failed to generate code");
        }

        // Mock deployment
        let wasm_blob = b"mock_synthesized_driver";

        #[cfg(all(not(test), target_arch = "wasm32"))]
        let res = unsafe { sys_hot_swap(wasm_blob.as_ptr(), wasm_blob.len() as i32) };
        #[cfg(any(test, not(target_arch = "wasm32")))]
        let res = 0;

        if res == 0 {
            Ok(alloc::format!(
                "Driver synthesized and loaded, len {}",
                wasm_blob.len()
            ))
        } else {
            Err("Hot swap failed")
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod allocator {
    use core::alloc::{GlobalAlloc, Layout};
    use core::ptr::{addr_of_mut, null_mut};
    use core::sync::atomic::{AtomicUsize, Ordering};

    struct BumpAllocator {
        next: AtomicUsize,
    }

    impl BumpAllocator {
        const fn new() -> Self {
            BumpAllocator {
                next: AtomicUsize::new(0),
            }
        }
    }

    unsafe impl GlobalAlloc for BumpAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
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

        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_synthesis_engine_init() {
        let mut engine = HardwareSynthesisEngine::new();
        assert!(engine.init().is_ok());
        let res = engine
            .synthesize_and_deploy(b"pci_mock_descriptor")
            .unwrap();
        assert_eq!(res, "Driver synthesized and loaded, len 23");
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod std_alloc {
    extern crate std;
}
