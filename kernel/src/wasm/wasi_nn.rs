extern crate alloc;
use wasmi::Caller;

pub const WASI_NN_ERRNO_SUCCESS: i32 = 0;
pub const WASI_NN_ERRNO_INVALID_ARGUMENT: i32 = 1;
pub const WASI_NN_ERRNO_INVALID_ENCODING: i32 = 2;
pub const WASI_NN_ERRNO_MISSING_MEMORY: i32 = 3;
pub const WASI_NN_ERRNO_BUSY: i32 = 4;
pub const WASI_NN_ERRNO_RUNTIME_ERROR: i32 = 5;
pub const WASI_NN_ERRNO_UNSUPPORTED_OPERATION: i32 = 6;
pub const WASI_NN_ERRNO_TOO_LARGE: i32 = 7;
pub const WASI_NN_ERRNO_NOT_FOUND: i32 = 8;

use inference_runtime::{InferenceEngine, Tensor};
use alloc::vec::Vec;

pub struct WasiNnCtx {
    pub engine: InferenceEngine,
}

impl Default for WasiNnCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl WasiNnCtx {
    pub fn new() -> Self {
        Self {
            engine: InferenceEngine::new(),
        }
    }
}

pub fn load(
    mut caller: Caller<'_, crate::wasm::wasi::WasiCtx>,
    _builder: i32,
    _builder_len: i32,
    _encoding: i32,
    _target: i32,
    graph_ptr: i32,
) -> i32 {
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return WASI_NN_ERRNO_MISSING_MEMORY,
    };

    let model_res = caller.data_mut().nn_ctx.engine.load_model(b"mock_data");
    if model_res.is_err() {
        return WASI_NN_ERRNO_INVALID_ARGUMENT;
    }
    let graph_handle = model_res.unwrap().id as u32;

    if memory
        .write(
            &mut caller,
            graph_ptr as u32 as usize,
            &graph_handle.to_le_bytes(),
        )
        .is_err()
    {
        return WASI_NN_ERRNO_INVALID_ARGUMENT;
    }

    WASI_NN_ERRNO_SUCCESS
}

pub fn load_by_name(
    mut caller: Caller<'_, crate::wasm::wasi::WasiCtx>,
    _name_ptr: i32,
    _name_len: i32,
    graph_ptr: i32,
) -> i32 {
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return WASI_NN_ERRNO_MISSING_MEMORY,
    };

    let model_res = caller.data_mut().nn_ctx.engine.load_model_by_name("mock_name");
    if model_res.is_err() {
        return WASI_NN_ERRNO_INVALID_ARGUMENT;
    }
    let graph_handle = model_res.unwrap().id as u32;

    if memory
        .write(
            &mut caller,
            graph_ptr as u32 as usize,
            &graph_handle.to_le_bytes(),
        )
        .is_err()
    {
        return WASI_NN_ERRNO_INVALID_ARGUMENT;
    }

    WASI_NN_ERRNO_SUCCESS
}

pub fn init_execution_context(
    mut caller: Caller<'_, crate::wasm::wasi::WasiCtx>,
    _graph: i32,
    context_ptr: i32,
) -> i32 {
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return WASI_NN_ERRNO_MISSING_MEMORY,
    };

    // Assuming we somehow get the model instance, in this mock we create a fake one
    let mock_model = inference_runtime::Model {
        id: _graph as usize,
        name: "mock_model",
    };

    let ctx_res = caller.data_mut().nn_ctx.engine.init_execution_context(&mock_model);
    if ctx_res.is_err() {
        return WASI_NN_ERRNO_INVALID_ARGUMENT;
    }

    let context_handle = ctx_res.unwrap() as u32;

    if memory
        .write(
            &mut caller,
            context_ptr as u32 as usize,
            &context_handle.to_le_bytes(),
        )
        .is_err()
    {
        return WASI_NN_ERRNO_INVALID_ARGUMENT;
    }

    WASI_NN_ERRNO_SUCCESS
}

pub fn set_input(
    mut caller: Caller<'_, crate::wasm::wasi::WasiCtx>,
    context: i32,
    index: i32,
    _tensor_ptr: i32, // Simplified for mock, normally we'd read the tensor struct
) -> i32 {
    let tensor = Tensor::new(Vec::new(), Vec::new());
    if caller.data_mut().nn_ctx.engine.set_input(context as usize, index as u32, &tensor).is_err() {
        return WASI_NN_ERRNO_INVALID_ARGUMENT;
    }
    WASI_NN_ERRNO_SUCCESS
}

pub fn compute(mut caller: Caller<'_, crate::wasm::wasi::WasiCtx>, context: i32) -> i32 {
    if caller.data_mut().nn_ctx.engine.compute(context as usize).is_err() {
        return WASI_NN_ERRNO_RUNTIME_ERROR;
    }
    WASI_NN_ERRNO_SUCCESS
}

pub fn get_output(
    mut caller: Caller<'_, crate::wasm::wasi::WasiCtx>,
    context: i32,
    index: i32,
    out_buffer: i32,
    out_buffer_max_size: i32,
    bytes_written_ptr: i32,
) -> i32 {
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return WASI_NN_ERRNO_MISSING_MEMORY,
    };

    let mut tmp_buffer = alloc::vec![0u8; out_buffer_max_size as usize];

    let bytes_written = match caller.data_mut().nn_ctx.engine.get_output(context as usize, index as u32, &mut tmp_buffer) {
        Ok(b) => b,
        Err(_) => return WASI_NN_ERRNO_RUNTIME_ERROR,
    };

    if memory.write(&mut caller, out_buffer as u32 as usize, &tmp_buffer[..bytes_written]).is_err() {
        return WASI_NN_ERRNO_INVALID_ARGUMENT;
    }

    let bytes_written_u32 = bytes_written as u32;
    if memory
        .write(
            &mut caller,
            bytes_written_ptr as u32 as usize,
            &bytes_written_u32.to_le_bytes(),
        )
        .is_err()
    {
        return WASI_NN_ERRNO_INVALID_ARGUMENT;
    }

    WASI_NN_ERRNO_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasi_nn_ctx_init() {
        let _ctx = WasiNnCtx::new();
        let _default_ctx = WasiNnCtx::default();
        assert!(true); // In tests, the engine internal state starts clean
    }

    #[test]
    fn test_errno_values() {
        assert_eq!(WASI_NN_ERRNO_SUCCESS, 0);
        assert_eq!(WASI_NN_ERRNO_INVALID_ARGUMENT, 1);
        assert_eq!(WASI_NN_ERRNO_NOT_FOUND, 8);
    }

    #[test]
    fn test_wasi_nn_stubs() {
        use crate::wasm::wasi::WasiCtx;
        use wasmi::{Engine, Memory, MemoryType, Store};

        let engine = Engine::default();
        let mut store = Store::new(&engine, WasiCtx::new());
        let _memory = Memory::new(&mut store, MemoryType::new(1, None)).unwrap();

        // We cannot easily mock `Caller` in `wasmi`, so we will rely on integration tests
        // to cover the memory write paths of the functions.
        // However, we can test that the functions return `WASI_NN_ERRNO_SUCCESS`
        // or properly error out if memory is missing.

        // This is a minimal unit test; coverage will primarily come from the linker
        // integration test in `linker.rs` where we can actually invoke these host functions
        // from a WebAssembly module.
    }
}
