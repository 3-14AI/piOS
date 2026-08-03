extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
// use alloc::vec::Vec;
use crate::wasm::wasi::WasiCtx;
#[cfg(not(feature = "verus"))]
use crate::wasm::wasi_nn::{
    compute, get_output, init_execution_context, load, load_by_name, set_input,
};
use wasmi::{Engine, Func, Instance, Linker, Module, Store};

pub struct WasmComponentLinker {
    engine: Engine,
    modules: BTreeMap<String, Module>,
}

impl Default for WasmComponentLinker {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmComponentLinker {
    pub fn new() -> Self {
        let mut config = wasmi::Config::default();
        #[cfg(target_arch = "x86_64")]
        config.wasm_simd(true);
        #[cfg(target_arch = "aarch64")]
        config.wasm_simd(true);
        #[cfg(target_arch = "riscv64")]
        config.wasm_simd(false);
        let engine = Engine::new(&config);
        Self {
            engine,
            modules: BTreeMap::new(),
        }
    }

    pub fn add_module(&mut self, name: &str, wasm_bytes: &[u8]) -> Result<(), wasmi::Error> {
        let module = Module::new(&self.engine, wasm_bytes)?;
        self.modules.insert(name.to_string(), module);
        Ok(())
    }

    pub fn link_and_run(&self, main_module_name: &str) -> Result<(), wasmi::Error> {
        let mut store = Store::new(&self.engine, WasiCtx::new());
        let mut linker = <Linker<WasiCtx>>::new(&self.engine);

        // Define WASI imports
        linker.define(
            "wasi_snapshot_preview1",
            "fd_write",
            wasmi::Func::wrap(&mut store, crate::wasm::wasi::fd_write),
        )?;
        linker.define(
            "wasi_snapshot_preview1",
            "fd_read",
            wasmi::Func::wrap(&mut store, crate::wasm::wasi::fd_read),
        )?;
        linker.define(
            "wasi_snapshot_preview1",
            "fd_close",
            wasmi::Func::wrap(&mut store, crate::wasm::wasi::fd_close),
        )?;
        linker.define(
            "wasi_snapshot_preview1",
            "environ_get",
            wasmi::Func::wrap(&mut store, crate::wasm::wasi::environ_get),
        )?;
        linker.define(
            "wasi_snapshot_preview1",
            "environ_sizes_get",
            wasmi::Func::wrap(&mut store, crate::wasm::wasi::environ_sizes_get),
        )?;
        linker.define(
            "wasi_snapshot_preview1",
            "args_get",
            wasmi::Func::wrap(&mut store, crate::wasm::wasi::args_get),
        )?;
        linker.define(
            "wasi_snapshot_preview1",
            "args_sizes_get",
            wasmi::Func::wrap(&mut store, crate::wasm::wasi::args_sizes_get),
        )?;
        linker.define(
            "wasi_snapshot_preview1",
            "proc_exit",
            wasmi::Func::wrap(&mut store, crate::wasm::wasi::proc_exit),
        )?;

        // Conditional WASI-NN mock imports
        #[cfg(not(feature = "verus"))]
        {
            linker.define(
                "wasi_ephemeral_nn",
                "load",
                wasmi::Func::wrap(&mut store, load),
            )?;
            linker.define(
                "wasi_ephemeral_nn",
                "load_by_name",
                wasmi::Func::wrap(&mut store, load_by_name),
            )?;
            linker.define(
                "wasi_ephemeral_nn",
                "init_execution_context",
                wasmi::Func::wrap(&mut store, init_execution_context),
            )?;
            linker.define(
                "wasi_ephemeral_nn",
                "set_input",
                wasmi::Func::wrap(&mut store, set_input),
            )?;
            linker.define(
                "wasi_ephemeral_nn",
                "compute",
                wasmi::Func::wrap(&mut store, compute),
            )?;
            linker.define(
                "wasi_ephemeral_nn",
                "get_output",
                wasmi::Func::wrap(&mut store, get_output),
            )?;
        }

        // Add custom `sys_intent` if not running Verus
        #[cfg(not(feature = "verus"))]
        linker.define(
            "wasi_snapshot_preview1",
            "sys_intent",
            wasmi::Func::wrap(&mut store, crate::wasm::wasi::sys_intent),
        )?;

        // Provide cryptographic functions
        linker.define(
            "wasi_ephemeral_crypto",
            "constant_time_eq",
            wasmi::Func::wrap(&mut store, crate::wasm::wasi_crypto::constant_time_eq_host),
        )?;

        // Link all modules to each other
        for (name, module) in &self.modules {
            if name != main_module_name {
                let instance = linker.instantiate(&mut store, module)?.start(&mut store)?;
                linker.define_instance(&mut store, name, instance)?;
            }
        }

        let main_module = self
            .modules
            .get(main_module_name)
            .ok_or_else(|| wasmi::Error::new("Main module not found"))?;

        let instance = linker
            .instantiate(&mut store, main_module)?
            .start(&mut store)?;

        if let Some(main_func) = instance.get_export(&store, "main") {
            if let Some(func) = main_func.into_func() {
                func.call(&mut store, &[], &mut [])?;
                return Ok(());
            }
        }

        if let Some(start_func) = instance.get_export(&store, "_start") {
            if let Some(func) = start_func.into_func() {
                func.call(&mut store, &[], &mut [])?;
                return Ok(());
            }
        }

        Err(wasmi::Error::new("No entry point found"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_component_linker_creation() {
        let linker = WasmComponentLinker::new();
        assert_eq!(linker.modules.len(), 0);
        let default_linker = WasmComponentLinker::default();
        assert_eq!(default_linker.modules.len(), 0);
    }

    #[test]
    fn test_wasm_component_linker_add_invalid_bytes() {
        let mut linker = WasmComponentLinker::new();
        let invalid_bytes = [0x00, 0x01, 0x02];
        let result = linker.add_module("invalid", &invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_wasm_component_linker_valid_bytes() {
        let mut linker = WasmComponentLinker::new();
        // A minimal valid wasm module (empty)
        let wasm_bytes = [
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b, 0x00, 0x1f, 0x04, 0x6e, 0x61, 0x6d, 0x65, 0x01,
            0x07, 0x01, 0x00, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x02, 0x07, 0x01, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        assert!(linker.add_module("main", &wasm_bytes).is_ok());
        let res = linker.link_and_run("main");
        assert!(
            res.is_ok(),
            "failed to link and run valid wasm: {:?}",
            res.err()
        );
    }

    #[test]
    fn test_wasm_component_linker_with_dependency() {
        let mut linker = WasmComponentLinker::new();

        let add_wasm = wat::parse_str(
            r#"
            (module
              (func $add (param $x i32) (param $y i32) (result i32)
                local.get $x
                local.get $y
                i32.add
              )
              (export "add" (func $add))
            )
        "#,
        )
        .unwrap();
        assert!(linker.add_module("math", &add_wasm).is_ok());

        let main_wasm = wat::parse_str(
            r#"
            (module
              (import "math" "add" (func $add (param i32 i32) (result i32)))
              (func $main (export "main")
                i32.const 2
                i32.const 3
                call $add
                drop
              )
            )
        "#,
        )
        .unwrap();

        match linker.add_module("main", &main_wasm) {
            Ok(_) => (),
            Err(e) => panic!("Error adding module: {:?}", e),
        }

        let res = linker.link_and_run("main");
        assert!(
            res.is_ok(),
            "failed to link and run with dependency: {:?}",
            res.err()
        );
    }

    #[test]
    fn test_wasi_nn_linking() {
        let mut linker = WasmComponentLinker::new();

        let main_wasm = wat::parse_str(
            r#"
            (module
              (import "wasi_ephemeral_nn" "load" (func $load (param i32 i32 i32 i32 i32) (result i32)))
              (import "wasi_ephemeral_nn" "load_by_name" (func $load_by_name (param i32 i32 i32) (result i32)))
              (import "wasi_ephemeral_nn" "init_execution_context" (func $init_execution_context (param i32 i32) (result i32)))
              (import "wasi_ephemeral_nn" "set_input" (func $set_input (param i32 i32 i32) (result i32)))
              (import "wasi_ephemeral_nn" "compute" (func $compute (param i32) (result i32)))
              (import "wasi_ephemeral_nn" "get_output" (func $get_output (param i32 i32 i32 i32 i32) (result i32)))
              (memory (export "memory") 1)
              (func $main (export "main")
                i32.const 0
                i32.const 0
                i32.const 0
                i32.const 0
                i32.const 16
                call $load
                drop

                i32.const 0
                i32.const 0
                i32.const 20
                call $load_by_name
                drop

                i32.const 1
                i32.const 24
                call $init_execution_context
                drop

                i32.const 1
                i32.const 0
                i32.const 30
                call $set_input
                drop

                i32.const 1
                call $compute
                drop

                i32.const 1
                i32.const 0
                i32.const 30
                i32.const 40
                i32.const 44
                call $get_output
                drop
              )
            )
        "#,
        )
        .unwrap();

        assert!(linker.add_module("main", &main_wasm).is_ok());

        let res = linker.link_and_run("main");
        assert!(
            res.is_ok(),
            "failed to link and run with wasi_ephemeral_nn: {:?}",
            res.err()
        );
    }

    #[test]
    fn test_cranelift_compiler_backend_integration() {
        let mut linker = WasmComponentLinker::new();

        // The `test_wasi.wasm` includes the cranelift integration
        let wasm_bytes = include_bytes!("../../../test_wasi/test_wasi.wasm");

        let result = linker.add_module("test_wasi", wasm_bytes);
        // Depending on whether we want to run it, linking itself shouldn't fail due to imports
        assert!(result.is_ok());
    }
}
