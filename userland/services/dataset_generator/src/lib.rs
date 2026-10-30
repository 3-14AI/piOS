extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Tensor};

pub struct DatasetGenerator {
    pub engine: InferenceEngine,
    pub collected_logs: Vec<String>,
}

impl Default for DatasetGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl DatasetGenerator {
    pub fn new() -> Self {
        let engine = InferenceEngine::new();
        Self {
            engine,
            collected_logs: Vec::new(),
        }
    }

    pub fn collect_log(&mut self, log: &str) {
        self.collected_logs.push(String::from(log));
    }

    pub fn generate_dataset(&mut self) -> Option<Vec<String>> {
        if self.collected_logs.is_empty() {
            return None;
        }

        let mut dataset = Vec::new();
        // Load a specialized dataset generation model (mocked)
        let model = self.engine.load_model_by_name("dataset_generator").ok()?;
        let ctx = self.engine.init_execution_context(&model).ok()?;

        for log in &self.collected_logs {
            let tensor = Tensor::new(log.as_bytes().to_vec(), alloc::vec![log.len()]);
            if self.engine.set_input(ctx, 0, &tensor).is_ok() && self.engine.compute(ctx).is_ok() {
                let mut out_buffer = alloc::vec![0u8; 1024];
                if let Ok(_bytes_written) = self.engine.get_output(ctx, 0, &mut out_buffer) {
                    dataset.push(alloc::format!("Sample generated from log: {}", log));
                }
            }
        }

        self.collected_logs.clear();
        Some(dataset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_generation() {
        let mut generator = DatasetGenerator::new();
        generator.collect_log("Kernel panic: out of memory");
        generator.collect_log("User command: ls -la /");

        let dataset = generator.generate_dataset().unwrap();
        assert_eq!(dataset.len(), 2);
        assert!(dataset[0].contains("Kernel panic"));
        assert!(dataset[1].contains("User command"));
    }
}
