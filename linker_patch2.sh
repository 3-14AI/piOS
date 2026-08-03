#!/bin/bash
sed -i 's/engine: Engine::default(),/let mut config = wasmi::Config::default();\n        #[cfg(target_arch = "x86_64")]\n        config.wasm_simd(true);\n        #[cfg(target_arch = "aarch64")]\n        config.wasm_simd(true);\n        #[cfg(target_arch = "riscv64")]\n        config.wasm_simd(false);\n        let engine = Engine::new(\&config);\n        Self { engine, modules: BTreeMap::new() }/g' kernel/src/wasm/linker.rs
sed -i 's/Self {/Self {/g' kernel/src/wasm/linker.rs
