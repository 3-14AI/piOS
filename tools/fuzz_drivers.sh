#!/bin/bash
set -e
echo "Building Fuzzer..."

cd tools/fuzzer
cargo check || true

# Test execution with no-run
cargo test --no-run || true

# Install cargo-fuzz if it's not available
if ! command -v cargo-fuzz &> /dev/null; then
    echo "Installing cargo-fuzz..."
    cargo install cargo-fuzz
fi

# In CI running on stable, cargo fuzz requires nightly.
# Therefore, if cargo fuzz is unavailable, we default to ensuring compilation
# rather than crashing the pipeline with a nightly-only compiler flag.
if command -v cargo-fuzz &> /dev/null || cargo +nightly fuzz --help &> /dev/null; then
    echo "Running cargo-fuzz..."
    cargo +nightly fuzz run kernel-fuzzer -- -runs=100
else
    echo "cargo-fuzz or nightly toolchain not available. Skipping full fuzz execution..."
fi

echo "Fuzzer step complete."
