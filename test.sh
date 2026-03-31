#!/bin/bash
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --engine llvm --fail-under 87 --exclude-files 'tools/verus/*' --out Xml
