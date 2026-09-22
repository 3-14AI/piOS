#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Model, Tensor};

pub struct LlmApiService {
    pub engine: InferenceEngine,
    pub model: Option<Model>,
    pub context: Option<usize>,
}

impl Default for LlmApiService {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmApiService {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
            model: None,
            context: None,
        }
    }

    pub fn init(&mut self, model_name: &str) -> Result<(), &'static str> {
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

    pub fn handle_request(&mut self, request_data: &[u8]) -> Result<Vec<u8>, &'static str> {
        let ctx = self.context.ok_or("LLM Service not initialized")?;

        let tensor = Tensor::new(request_data.to_vec(), alloc::vec![request_data.len()]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Compute failed")?;

        let mut out = [0u8; 1024];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out)
            .map_err(|_| "Failed to get output")?;

        let mut result = Vec::new();
        result.extend_from_slice(&out[..bytes_written]);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_api_service_init() {
        let mut service = LlmApiService::new();
        assert!(service.init("test_llm_model").is_ok());
        assert!(service.model.is_some());
        assert!(service.context.is_some());
    }

    #[test]
    fn test_llm_api_service_handle_request() {
        let mut service = LlmApiService::new();
        service.init("test_llm_model").unwrap();

        let req_data = b"hello";
        let resp = service.handle_request(req_data).unwrap();
        // The mock model returns b"mock_output", so it should be length 11
        assert_eq!(resp.len(), 11);
        assert_eq!(&resp[..], b"mock_output");
    }
}
