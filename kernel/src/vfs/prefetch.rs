extern crate alloc;

use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Model, Tensor};

pub struct PrefetchPredictor {
    engine: InferenceEngine,
    model: Model,
    pub prefetched_inodes: Vec<u64>,
}

impl PrefetchPredictor {
    pub fn new() -> Result<Self, &'static str> {
        let mut engine = InferenceEngine::new();
        let model = engine
            .load_model_by_name("prefetch_model")
            .map_err(|_| "Failed to load prefetch model")?;

        Ok(Self {
            engine,
            model,
            prefetched_inodes: Vec::new(),
        })
    }

    pub fn predict_and_prefetch(&mut self, current_inode: u64) -> Result<u64, &'static str> {
        let ctx = self
            .engine
            .init_execution_context(&self.model)
            .map_err(|_| "Failed to init context")?;

        let input_data = alloc::vec![current_inode as u8];
        let tensor = Tensor::new(input_data, alloc::vec![1]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;

        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 32];
        self.engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        let predicted_inode = out[0] as u64;
        if predicted_inode > 0 {
            self.prefetched_inodes.push(predicted_inode);
        }

        Ok(predicted_inode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefetch_predictor() {
        let mut predictor = PrefetchPredictor::new().unwrap();
        // With mock engine, out[0] usually relies on internal mock behaviour, but we can verify it doesn't panic.
        let result = predictor.predict_and_prefetch(5);
        assert!(result.is_ok());
    }
}
