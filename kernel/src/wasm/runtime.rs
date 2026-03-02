extern crate alloc;

use alloc::vec::Vec;
use wasmi::{Caller, Engine, Extern, Func, Linker, Module, Store};

pub struct WasmRuntime {
    engine: Engine,
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmRuntime {
    pub fn new() -> Self {
        let engine = Engine::default();
        Self { engine }
    }

    pub fn run(&self, wasm_bytes: &[u8]) -> Result<(), wasmi::Error> {
        let module = Module::new(&self.engine, wasm_bytes)?;

        type HostState = u32;
        let mut store = Store::new(&self.engine, 42);

        let mut linker = <Linker<HostState>>::new(&self.engine);
        linker.define(
            "env",
            "hello",
            Func::wrap(&mut store, |caller: Caller<'_, HostState>, param: i32| {
                // Dummy syscall or host function
                log::info!(
                    "Hello from WASM host! State: {}, Param: {}",
                    caller.data(),
                    param
                );
            }),
        )?;

        let instance = linker.instantiate_and_start(&mut store, &module)?;

        if let Some(Extern::Func(main)) = instance.get_export(&mut store, "main") {
            let typed_main = main.typed::<(), ()>(&store)?;
            typed_main.call(&mut store, ())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_runtime_creation() {
        let _runtime = WasmRuntime::new();
        let _default_runtime = WasmRuntime::default();
        assert!(true);
    }

    #[test]
    fn test_wasm_runtime_run_invalid_bytes() {
        let runtime = WasmRuntime::new();
        let res = runtime.run(&[0x00, 0x01, 0x02]); // Invalid WASM header
        assert!(res.is_err());
    }

    #[test]
    fn test_wasm_runtime_run_valid_bytes() {
        let runtime = WasmRuntime::new();
        // Minimal valid WASM bytes
        let wasm_bytes = [
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7f,
            0x00, 0x60, 0x00, 0x00, 0x02, 0x0d, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x05, 0x68, 0x65,
            0x6c, 0x6c, 0x6f, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x07, 0x08, 0x01, 0x04, 0x6d,
            0x61, 0x69, 0x6e, 0x00, 0x01, 0x0a, 0x08, 0x01, 0x06, 0x00, 0x41, 0x2a, 0x10, 0x00,
            0x0b, 0x00, 0x15, 0x04, 0x6e, 0x61, 0x6d, 0x65, 0x01, 0x0e, 0x02, 0x00, 0x05, 0x68,
            0x65, 0x6c, 0x6c, 0x6f, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e,
        ];
        let res = runtime.run(&wasm_bytes);
        assert!(res.is_ok(), "failed to run valid wasm: {:?}", res.err());
    }
}
