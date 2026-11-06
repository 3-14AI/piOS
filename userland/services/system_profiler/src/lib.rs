#![no_std]

extern crate alloc;

use inference_runtime::{InferenceEngine, Model, Tensor};

pub struct SystemProfiler {
    pub engine: InferenceEngine,
    pub model: Option<Model>,
    pub context: Option<usize>,
    pub scheduler_quantum: usize,
}

impl Default for SystemProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemProfiler {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            model: None,
            context: None,
            scheduler_quantum: 10, // 10ms default
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        let model = self
            .engine
            .load_model_by_name("autonomous_profiler")
            .map_err(|_| "Failed to load model")?;
        let ctx = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init execution context")?;
        self.model = Some(model);
        self.context = Some(ctx);
        Ok(())
    }

    pub fn analyze_and_adjust(
        &mut self,
        cpu_usage: u8,
        mem_usage: u8,
        io_usage: u8,
    ) -> Result<(), &'static str> {
        let ctx = self.context.ok_or("Profiler not initialized")?;

        let input_data = alloc::vec![cpu_usage, mem_usage, io_usage];
        let tensor = Tensor::new(input_data, alloc::vec![3]);

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
            // [desired_quantum_ms, needs_compaction]
            let desired_quantum = out[0];
            let needs_compaction = out[1];

            if desired_quantum > 0 {
                self.scheduler_quantum = desired_quantum as usize;
            } else {
                self.scheduler_quantum = 15; // default fallback
            }

            #[cfg(target_arch = "wasm32")]
            {
                unsafe {
                    set_scheduler_quantum(self.scheduler_quantum as i32);
                    if needs_compaction > 0 {
                        compact_memory();
                    }
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Just to use the variable in non-wasm builds and prevent unused variable warning
                let _ = needs_compaction;
            }
        }

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    fn set_scheduler_quantum(quantum: i32) -> i32;
    fn compact_memory() -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_profiler_init() {
        let mut profiler = SystemProfiler::new();
        assert!(profiler.init().is_ok());
        assert!(profiler.model.is_some());
        assert!(profiler.context.is_some());
    }

    #[test]
    fn test_system_profiler_adjust() {
        let mut profiler = SystemProfiler::new();
        profiler.init().unwrap();

        profiler.analyze_and_adjust(80, 70, 90).unwrap();

        // With the mock model bytes (b"mock_output"),
        // out[0] == b'm' == 109, out[1] == b'o' == 111.
        // So scheduler_quantum should be 109.
        assert_eq!(profiler.scheduler_quantum, 109);
    }
}
