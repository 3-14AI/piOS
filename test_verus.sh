#!/bin/bash
cd kernel
RUSTC_BOOTSTRAP=1 RUSTFLAGS="-Zbuild-std=core,compiler_builtins,alloc,panic_abort" ../tools/verus/verus-x86-linux/verus src/sync.rs --crate-type=lib --cfg 'feature="verus"' --extern vstd=../tools/verus/verus-x86-linux/libvstd.rlib
