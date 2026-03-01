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
            Func::wrap(&mut store, |caller: Caller<'_, HostState>| {
                // Dummy syscall or host function
                log::info!("Hello from WASM host! State: {}", caller.data());
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
}
