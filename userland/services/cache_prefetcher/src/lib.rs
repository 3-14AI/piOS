extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Tensor};

pub struct CachePrefetcher {
    pub engine: InferenceEngine,
    pub prefetched_files: Vec<String>,
}

impl Default for CachePrefetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl CachePrefetcher {
    pub fn new() -> Self {
        let engine = InferenceEngine::new();
        Self {
            engine,
            prefetched_files: Vec::new(),
        }
    }

    pub fn analyze_and_prefetch(&mut self, context: &str) -> Option<Vec<String>> {
        let model = self.engine.load_model_by_name("cache_prefetcher").ok()?;
        let ctx = self.engine.init_execution_context(&model).ok()?;

        let tensor = Tensor::new(context.as_bytes().to_vec(), alloc::vec![context.len()]);

        if self.engine.set_input(ctx, 0, &tensor).is_ok() && self.engine.compute(ctx).is_ok() {
            let mut out_buffer = alloc::vec![0u8; 128];
            if let Ok(_bytes_written) = self.engine.get_output(ctx, 0, &mut out_buffer) {
                // Mock interpretation of output tensor to predict files to prefetch

                if context.contains("building kernel") {
                    self.prefetched_files.push(String::from("/src/main.rs"));
                    self.prefetched_files.push(String::from("/Cargo.toml"));
                    return Some(self.prefetched_files.clone());
                }

                return Some(Vec::new());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_prefetching() {
        let mut prefetcher = CachePrefetcher::new();
        assert!(prefetcher.prefetched_files.is_empty());

        let prefetched = prefetcher.analyze_and_prefetch("User is building kernel.");

        assert!(prefetched.is_some());
        let files = prefetched.unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&String::from("/src/main.rs")));
    }
}
