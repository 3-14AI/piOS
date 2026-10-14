use alloc::vec::Vec;
use core::ffi::c_void;
use inference_runtime::{InferenceEngine, Model, Tensor};

#[cfg(not(test))]
extern "C" {
    fn wasi_syscall_fuzz_target(syscall_id: u32, arg_ptr: *const c_void, arg_len: usize) -> i32;
}

#[cfg(test)]
unsafe fn wasi_syscall_fuzz_target(_id: u32, _ptr: *const c_void, _len: usize) -> i32 {
    0
}

pub struct SysFuzzer {
    engine: InferenceEngine,
    model: Model,
}

impl SysFuzzer {
    pub fn new() -> Result<Self, &'static str> {
        let mut engine = InferenceEngine::new();
        let model = engine
            .load_model_by_name("syscall_mutator")
            .map_err(|_| "Failed to load fuzzer model")?;
        Ok(Self { engine, model })
    }

    pub fn generate_test_case(&mut self, prior_path: &[u8]) -> Vec<u8> {
        let ctx = match self.engine.init_execution_context(&self.model) {
            Ok(c) => c,
            Err(_) => return alloc::vec![],
        };

        let tensor = Tensor::new(prior_path.to_vec(), alloc::vec![prior_path.len()]);
        let _ = self.engine.set_input(ctx, 0, &tensor);
        let _ = self.engine.compute(ctx);

        let mut out = alloc::vec![0u8; 128];
        if let Ok(bytes_written) = self.engine.get_output(ctx, 0, &mut out) {
            out.truncate(bytes_written);
            return out;
        }

        alloc::vec![]
    }

    pub fn fuzz_target(&mut self, syscall_id: u32, prior_path: &[u8]) -> i32 {
        let mutated_args = self.generate_test_case(prior_path);

        unsafe {
            wasi_syscall_fuzz_target(
                syscall_id,
                mutated_args.as_ptr() as *const c_void,
                mutated_args.len(),
            )
        }
    }
}
