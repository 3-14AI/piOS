#!/bin/bash
set -e

echo "Building kernel..."
# We need to build the kernel target
# Use RUSTC_BOOTSTRAP=1 to allow -Z build-std on stable
RUSTC_BOOTSTRAP=1 cargo +1.94.0-x86_64-unknown-linux-gnu build -p kernel --target x86_64-unknown-uefi -Zbuild-std=core,compiler_builtins,alloc,panic_abort --release

# Ensure we have ovmf for QEMU testing
sudo apt-get update || true
sudo apt-get install -y qemu-system-x86 ovmf || true

# Find OVMF
OVMF_PATH=""
if [ -f /usr/share/OVMF/OVMF_CODE.fd ]; then
    OVMF_PATH="/usr/share/OVMF/OVMF_CODE.fd"
elif [ -f /usr/share/qemu/OVMF.fd ]; then
    OVMF_PATH="/usr/share/qemu/OVMF.fd"
else
    echo "Warning: OVMF firmware not found, skipping QEMU test."
    exit 0
fi

# Prepare an ESP directory
ESP_DIR="target/qemu-esp"
mkdir -p "$ESP_DIR/EFI/BOOT"
cp target/x86_64-unknown-uefi/release/kernel.efi "$ESP_DIR/EFI/BOOT/BOOTX64.EFI"

echo "Running QEMU..."
# Run QEMU with serial output redirected to stdio and the isa-debug-exit device on port 0xf4
# We use timeout to avoid hanging forever
set +e
timeout 30s qemu-system-x86_64 \
    -nographic \
    -nodefaults \
    -machine q35 \
    -bios "$OVMF_PATH" \
    -drive format=raw,file=fat:rw:"$ESP_DIR" \
    -serial stdio \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 > qemu_output.log 2>&1

QEMU_EXIT=$?
set -e

cat qemu_output.log

# qemu isa-debug-exit exits with (val << 1) | 1
# Since we write 0x10, exit code is (0x10 << 1) | 1 = 33
if [ $QEMU_EXIT -eq 33 ]; then
    echo "QEMU Integration Test Passed!"
    exit 0
else
    echo "QEMU Integration Test Failed with exit code $QEMU_EXIT"
    exit 1
fi
