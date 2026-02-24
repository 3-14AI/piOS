#!/bin/bash
set -e
# Verify the kernel library using Verus
VERUS_BIN="tools/verus/verus-x86-linux/verus"

if [ ! -f "$VERUS_BIN" ]; then
    echo "Verus binary not found at $VERUS_BIN. Please run tools/install_verus.sh first."
    exit 1
fi

echo "Verifying kernel..."
# Verify lib.rs which includes verifier.rs
# We verify as a library.
# We need to set 'verus' feature cfg manually for conditional compilation
$VERUS_BIN kernel/src/lib.rs --crate-type=lib --cfg feature=\"verus\"
