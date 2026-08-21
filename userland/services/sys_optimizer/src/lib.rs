#![no_std]

extern crate alloc;

use inference_runtime::{InferenceEngine, Model, Tensor};

pub struct SysOptimizer {
    pub engine: InferenceEngine,
    pub model: Option<Model>,
    pub context: Option<usize>,
    pub memory_limit: usize,
    pub scheduler_quantum: usize,
    pub oom_prediction_threshold: u8,
}

impl Default for SysOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl SysOptimizer {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            model: None,
            context: None,
            memory_limit: 1024 * 1024 * 1024, // 1GB
            scheduler_quantum: 10,            // 10ms
            oom_prediction_threshold: 85,
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        let model = self
            .engine
            .load_model_by_name("system_optimizer")
            .map_err(|_| "Failed to load model")?;
        let ctx = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init execution context")?;
        self.model = Some(model);
        self.context = Some(ctx);
        Ok(())
    }

    pub fn analyze_and_adjust(&mut self, cpu_usage: u8, mem_usage: u8) -> Result<(), &'static str> {
        let ctx = self.context.ok_or("Optimizer not initialized")?;

        // Predict OOM and compact memory proactively
        if mem_usage > self.oom_prediction_threshold {
            unsafe {
                sys_compact_memory();
            }
        }

        let input_data = alloc::vec![cpu_usage, mem_usage];
        let tensor = Tensor::new(input_data, alloc::vec![2]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 32];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        if bytes_written > 0 {
            // Read output tensor logic to adjust parameters based on AI inference
            // Mocking the tensor format. The model is assumed to output 2 bytes:
            // [desired_quantum_ms, desired_mem_limit_scale]
            let desired_quantum = out[0];
            let memory_scale = out[1];

            if desired_quantum > 0 {
                self.scheduler_quantum = desired_quantum as usize;
            } else {
                self.scheduler_quantum = 15; // default fallback
            }

            if memory_scale > 0 {
                self.memory_limit = (memory_scale as usize) * 10 * 1024 * 1024; // Scale limit
            } else {
                self.memory_limit = 1024 * 1024 * 1024; // default fallback
            }

            unsafe {
                // Call WASI API to tune kernel parameter
                tune_kernel_parameters(self.scheduler_quantum as i32, self.memory_limit as i32);
            }
        }

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn sys_tune_kernel(quantum: i32, memory_limit: i32) -> i32;
    fn sys_compact_memory() -> i32;
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn sys_tune_kernel(_quantum: i32, _memory_limit: i32) -> i32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn sys_compact_memory() -> i32 {
    0
}

unsafe fn tune_kernel_parameters(quantum: i32, memory_limit: i32) -> i32 {
    sys_tune_kernel(quantum, memory_limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_optimizer_init() {
        let mut optimizer = SysOptimizer::new();
        assert!(optimizer.init().is_ok());
        assert!(optimizer.model.is_some());
        assert!(optimizer.context.is_some());
    }

    #[test]
    fn test_sys_optimizer_adjust() {
        let mut optimizer = SysOptimizer::new();
        optimizer.init().unwrap();

        optimizer.analyze_and_adjust(90, 95).unwrap();

        // With the mock model bytes (b"mock_output"),
        // out[0] == b'm' == 109, out[1] == b'o' == 111.
        // So scheduler_quantum should be 109 and memory limit 111 * 10 * 1024 * 1024.
        assert_eq!(optimizer.scheduler_quantum, 109);
        assert_eq!(optimizer.memory_limit, 111 * 10 * 1024 * 1024);
    }
}
