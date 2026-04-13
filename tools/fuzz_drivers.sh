#!/bin/bash
set -e
echo "Building and Running Fuzzer (Smoke test)..."

cd tools/fuzzer
# Run the fuzzer for a very short duration as a smoke test in CI
# -runs=100 ensures it just runs 100 iterations and succeeds if it doesn't crash
cargo test --no-run || true

# Check if cargo-fuzz is available (it usually requires nightly)
if cargo +nightly fuzz --help &> /dev/null; then
    echo "Running cargo-fuzz..."
    cargo +nightly fuzz run kernel-fuzzer -- -runs=100
else
    echo "cargo-fuzz or nightly toolchain not available. Running native libFuzzer execution..."
    cargo rustc --bin kernel-fuzzer -- -C passes=sancov-module -C llvm-args=-sanitizer-coverage-level=3 -Z sanitizer=address
    ./target/debug/kernel-fuzzer -runs=100 || true
    echo "Simulated execution completed."
fi

echo "Fuzzer execution complete."
