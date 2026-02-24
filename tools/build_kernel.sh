#!/bin/bash
set -e
# Use specific toolchain installed for Verus
TOOLCHAIN="1.93.0-x86_64-unknown-linux-gnu"
# Enable nightly features on stable compiler
export RUSTC_BOOTSTRAP=1
cargo +$TOOLCHAIN build -p kernel --target x86_64-unknown-uefi -Zbuild-std=core,compiler_builtins,alloc,panic_abort --release
