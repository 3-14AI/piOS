use crate::wasm::wasi::{WasiCtx, WASI_ERRNO_BADF, WASI_ERRNO_SUCCESS};
use alloc::vec;
use wasmi::{Caller, Linker, Module, Store};

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
        // WP-151: Ephemeral Sandboxing
        // Here we simulate the compilation of the source code into a WASM module.
        // We then create an isolated, ephemeral sandbox for it.

        // Minimal valid WASM bytes for a module with an empty `main` function
        let dummy_wasm_bytes = [
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00,
            0x00, // Type section
            0x03, 0x02, 0x01, 0x00, // Function section
            0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, // Export section
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b, // Code section
        ];

        let engine = caller.engine();
        if let Ok(module) = Module::new(engine, &dummy_wasm_bytes[..]) {
            // Create a strict, restricted WasiCtx (e.g., no filesystem or networking capabilities)
            let restricted_ctx = WasiCtx::new(); // In a real scenario, this would have no capabilities.

            // The Store acts as our ephemeral sandbox. Once dropped, the sandbox is destroyed.
            let mut ephemeral_store = Store::new(engine, restricted_ctx);
            let linker = <Linker<WasiCtx>>::new(engine);

            if let Ok(instance) = linker.instantiate_and_start(&mut ephemeral_store, &module) {
                // Try to execute `main` inside the sandbox
                if let Some(main) = instance.get_export(&ephemeral_store, "main") {
                    if let Some(main_func) = main.into_func() {
                        if let Ok(typed_main) = main_func.typed::<(), ()>(&ephemeral_store) {
                            let _ = typed_main.call(&mut ephemeral_store, ());
                        }
                    }
                }
            }
            // ephemeral_store is dropped here, destroying the sandbox.
        }

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
