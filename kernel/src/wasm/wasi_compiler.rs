use crate::wasm::wasi::{WasiCtx, WASI_ERRNO_BADF, WASI_ERRNO_SUCCESS};
use alloc::vec;
use wasmi::Caller;

pub fn wasi_ephemeral_compiler(caller: Caller<'_, WasiCtx>, code_ptr: i32, code_len: i32) -> i32 {
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return WASI_ERRNO_BADF,
    };

    let mut code_buf = vec![0u8; code_len as usize];
    if memory
        .read(&caller, code_ptr as usize, &mut code_buf)
        .is_err()
    {
        return WASI_ERRNO_BADF;
    }

    if core::str::from_utf8(&code_buf).is_ok() {
        WASI_ERRNO_SUCCESS
    } else {
        WASI_ERRNO_BADF
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmi::{Engine, Memory, MemoryType, Store};

    #[test]
    fn test_wasi_ephemeral_compiler_stub() {
        let engine = Engine::default();
        let mut store = Store::new(&engine, WasiCtx::new());
        let _memory = Memory::new(&mut store, MemoryType::new(1, None)).unwrap();
        // Just verifying it compiles and can theoretically be called
    }
}
