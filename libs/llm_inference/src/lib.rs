#![no_std]

extern crate alloc;

use inference_runtime::{InferenceEngine, Model, Tensor};

pub struct LlmInference {
    pub engine: InferenceEngine,
    pub model: Option<Model>,
    pub context: Option<usize>,
}

impl Default for LlmInference {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmInference {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            model: None,
            context: None,
        }
    }

    pub fn load_model(&mut self, model_name: &str) -> Result<(), &'static str> {
        let model = self
            .engine
            .load_model_by_name(model_name)
            .map_err(|_| "Failed to load model")?;
        let ctx = self
            .engine
            .init_execution_context(&model)
            .map_err(|_| "Failed to init execution context")?;
        self.model = Some(model);
        self.context = Some(ctx);
        Ok(())
    }

    pub fn infer(&mut self, prompt: &str) -> Result<alloc::string::String, &'static str> {
        let ctx = self.context.ok_or("LLM not initialized")?;

        let input_data = prompt.as_bytes().to_vec();
        let tensor = Tensor::new(input_data, alloc::vec![prompt.len()]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 1024];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        let result = core::str::from_utf8(&out[..bytes_written])
            .unwrap_or("Failed to parse output")
            .into();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_inference_init() {
        let mut llm = LlmInference::new();
        assert!(llm.load_model("llama").is_ok());
        assert!(llm.model.is_some());
        assert!(llm.context.is_some());
    }

    #[test]
    fn test_llm_inference_run() {
        let mut llm = LlmInference::new();
        llm.load_model("llama").unwrap();

        let response = llm.infer("Hello AI").unwrap();
        assert_eq!(response, "mock_output");
    }
}
