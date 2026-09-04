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

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "wasi_ephemeral_nn")]
extern "C" {
    pub fn load(
        builder_ptr: *const u8,
        builder_len: i32,
        encoding: i32,
        target: i32,
        graph_ptr: *mut u32,
    ) -> i32;
    pub fn load_by_name(name_ptr: *const u8, name_len: i32, graph_ptr: *mut u32) -> i32;
    pub fn init_execution_context(graph: u32, context_ptr: *mut u32) -> i32;
    pub fn set_input(context: u32, index: i32, tensor_ptr: *const u8) -> i32;
    pub fn compute(context: u32) -> i32;
    pub fn get_output(
        context: u32,
        index: i32,
        out_buffer_ptr: *mut u8,
        out_buffer_max_size: i32,
        bytes_written_ptr: *mut u32,
    ) -> i32;
    pub fn save_weights(
        context: u32,
        path_ptr: *const u8,
        path_len: i32,
        weights_ptr: *const u8,
        weights_len: i32,
    ) -> i32;
}

#[cfg(not(target_arch = "wasm32"))]
pub struct InferenceEngine {
    loaded_models: usize,
    execution_contexts: usize,
    // Using a simple bool state to simulate compute readiness in the mock
    has_input: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

    pub fn set_input(
        &mut self,
        _context: usize,
        _index: u32,
        _tensor: &Tensor,
    ) -> Result<(), Error> {
        self.has_input = true;
        Ok(())
    }

    pub fn compute(&mut self, _context: usize) -> Result<(), Error> {
        if !self.has_input {
            return Err(Error::ComputeFailed);
        }
        Ok(())
    }

    pub fn get_output(
        &self,
        _context: usize,
        _index: u32,
        out_buffer: &mut [u8],
    ) -> Result<usize, Error> {
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

    pub fn save_weights(&self, _context: usize, _path: &str, _weights: &[u8]) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub struct InferenceEngine {
    _private: (),
}

#[cfg(target_arch = "wasm32")]
impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "wasm32")]
impl InferenceEngine {
    pub fn new() -> Self {
        Self { _private: () }
    }

    pub fn load_model(&mut self, model_data: &[u8]) -> Result<Model, Error> {
        let mut graph: u32 = 0;
        // Wasi-NN args (simplified): builder, builder_len, encoding, target, graph_ptr
        // For a true implementation, we'd need to properly format builder payload
        let res = unsafe {
            load(
                model_data.as_ptr(),
                model_data.len() as i32,
                0, // encoding OPENVINO=0 (mock)
                0, // target CPU=0
                &mut graph as *mut u32,
            )
        };
        if res != 0 {
            return Err(Error::InvalidModel);
        }
        Ok(Model {
            id: graph as usize,
            name: "wasi_nn_model",
        })
    }

    pub fn load_model_by_name(&mut self, name: &str) -> Result<Model, Error> {
        let mut graph: u32 = 0;
        let res = unsafe { load_by_name(name.as_ptr(), name.len() as i32, &mut graph as *mut u32) };
        if res != 0 {
            return Err(Error::InvalidModel);
        }
        Ok(Model {
            id: graph as usize,
            name: "wasi_nn_model_named",
        })
    }

    pub fn init_execution_context(&mut self, model: &Model) -> Result<usize, Error> {
        let mut context: u32 = 0;
        let res = unsafe { init_execution_context(model.id as u32, &mut context as *mut u32) };
        if res != 0 {
            return Err(Error::InvalidModel);
        }
        Ok(context as usize)
    }

    pub fn set_input(&mut self, context: usize, index: u32, tensor: &Tensor) -> Result<(), Error> {
        // Normally we might have to pass an actual WASI-NN Tensor structure pointer,
        // but for compatibility with the kernel mock we pass just the data pointer.
        let res = unsafe { set_input(context as u32, index as i32, tensor.data.as_ptr()) };
        if res != 0 {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }

    pub fn compute(&mut self, context: usize) -> Result<(), Error> {
        let res = unsafe { compute(context as u32) };
        if res != 0 {
            return Err(Error::ComputeFailed);
        }
        Ok(())
    }

    pub fn get_output(
        &self,
        context: usize,
        index: u32,
        out_buffer: &mut [u8],
    ) -> Result<usize, Error> {
        let mut bytes_written: u32 = 0;
        let res = unsafe {
            get_output(
                context as u32,
                index as i32,
                out_buffer.as_mut_ptr(),
                out_buffer.len() as i32,
                &mut bytes_written as *mut u32,
            )
        };
        if res != 0 {
            return Err(Error::ComputeFailed);
        }
        Ok(bytes_written as usize)
    }

    pub fn save_weights(&self, context: usize, path: &str, weights: &[u8]) -> Result<(), Error> {
        let res = unsafe {
            save_weights(
                context as u32,
                path.as_ptr(),
                path.len() as i32,
                weights.as_ptr(),
                weights.len() as i32,
            )
        };
        if res != 0 {
            return Err(Error::ComputeFailed);
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_inference_engine_init() {
        let engine = InferenceEngine::new();
        assert_eq!(engine.loaded_models, 0);
        assert_eq!(engine.execution_contexts, 0);
        assert!(!engine.has_input);
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

pub struct VisionModel {
    pub id: usize,
    pub name: &'static str,
}

impl VisionModel {
    pub fn new(id: usize, name: &'static str) -> Self {
        Self { id, name }
    }
}

pub fn image_to_tensor(_image_data: &[u8]) -> Result<Tensor, Error> {
    // Mock image to tensor conversion
    Ok(Tensor::new(alloc::vec![0; 100], alloc::vec![10, 10, 1]))
}

#[cfg(test)]
mod vision_tests {
    use super::*;

    #[test]
    fn test_image_to_tensor() {
        let tensor = image_to_tensor(b"dummy_image").unwrap();
        assert_eq!(tensor.dimensions, alloc::vec![10, 10, 1]);
    }
}
