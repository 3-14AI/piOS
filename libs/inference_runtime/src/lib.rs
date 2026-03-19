#![no_std]

extern crate alloc;

use alloc::vec::Vec;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidModel,
    InvalidInput,
    ComputeFailed,
    OutputBufferTooSmall,
}

// Ensure we use the proper type path when the optional features like mistralrs are enabled or disabled.
pub struct Tensor {
    pub data: Vec<u8>,
    pub dimensions: Vec<usize>,
}

impl Tensor {
    pub fn new(data: Vec<u8>, dimensions: Vec<usize>) -> Self {
        Self { data, dimensions }
    }
}

pub struct Model {
    pub id: usize,
    pub name: &'static str,
}

pub struct InferenceEngine {
    loaded_models: usize,
    execution_contexts: usize,
    // Using a simple bool state to simulate compute readiness in the mock
    has_input: bool,
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceEngine {
    pub fn new() -> Self {
        Self {
            loaded_models: 0,
            execution_contexts: 0,
            has_input: false,
        }
    }

    pub fn load_model(&mut self, _model_data: &[u8]) -> Result<Model, Error> {
        self.loaded_models += 1;
        Ok(Model {
            id: self.loaded_models,
            name: "mock_model",
        })
    }

    pub fn load_model_by_name(&mut self, _name: &str) -> Result<Model, Error> {
        self.loaded_models += 1;
        Ok(Model {
            id: self.loaded_models,
            name: "mock_model_named",
        })
    }

    pub fn init_execution_context(&mut self, _model: &Model) -> Result<usize, Error> {
        self.execution_contexts += 1;
        self.has_input = false;
        Ok(self.execution_contexts)
    }

    pub fn set_input(&mut self, _context: usize, _index: u32, _tensor: &Tensor) -> Result<(), Error> {
        self.has_input = true;
        Ok(())
    }

    pub fn compute(&mut self, _context: usize) -> Result<(), Error> {
        if !self.has_input {
            return Err(Error::ComputeFailed);
        }
        Ok(())
    }

    pub fn get_output(&self, _context: usize, _index: u32, out_buffer: &mut [u8]) -> Result<usize, Error> {
        if !self.has_input {
            return Err(Error::ComputeFailed);
        }
        let output_data = b"mock_output";
        if out_buffer.len() < output_data.len() {
            return Err(Error::OutputBufferTooSmall);
        }
        out_buffer[..output_data.len()].copy_from_slice(output_data);
        Ok(output_data.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_inference_engine_init() {
        let engine = InferenceEngine::new();
        assert_eq!(engine.loaded_models, 0);
        assert_eq!(engine.execution_contexts, 0);
        assert_eq!(engine.has_input, false);
    }

    #[test]
    fn test_inference_engine_load_model() {
        let mut engine = InferenceEngine::new();
        let model = engine.load_model(b"dummy_data").unwrap();
        assert_eq!(model.id, 1);
        assert_eq!(model.name, "mock_model");
        assert_eq!(engine.loaded_models, 1);
    }

    #[test]
    fn test_inference_engine_compute() {
        let mut engine = InferenceEngine::new();
        let model = engine.load_model_by_name("test").unwrap();
        let ctx = engine.init_execution_context(&model).unwrap();

        // compute fails without input
        assert_eq!(engine.compute(ctx), Err(Error::ComputeFailed));

        let tensor = Tensor::new(vec![1, 2, 3], vec![3]);
        engine.set_input(ctx, 0, &tensor).unwrap();

        assert_eq!(engine.compute(ctx), Ok(()));

        let mut out = [0u8; 32];
        let bytes_written = engine.get_output(ctx, 0, &mut out).unwrap();
        assert_eq!(bytes_written, 11);
        assert_eq!(&out[..11], b"mock_output");
    }
}
