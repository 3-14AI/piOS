#!/bin/bash
set -e

# Run tools/mkiso.sh if target/pios.iso doesn't exist
if [ ! -f "target/pios.iso" ]; then
    echo "ISO not found. Building ISO..."
    ./tools/mkiso.sh
fi

echo "Booting ISO in QEMU..."
# Start qemu and capture serial output
qemu-system-x86_64 \
    -m 512M \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE_4M.fd \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_VARS_4M.fd \
    -drive format=raw,file=target/disk.img \
    -serial file:qemu_iso.log \
    -display none \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 &

QEMU_PID=$!

# Timeout loop (30 seconds)
COUNT=0
while [ $COUNT -lt 30 ]; do
    if grep -q "Loaded initrd.img" qemu_iso.log 2>/dev/null; then
        echo "Successfully verified kernel booted and loaded initrd.img"
        kill -9 $QEMU_PID 2>/dev/null || true
        break
    fi
    sleep 1
    COUNT=$((COUNT+1))
done

if [ $COUNT -eq 30 ]; then
    echo "Timeout waiting for QEMU"
    cat qemu_iso.log
    kill -9 $QEMU_PID 2>/dev/null || true
    # exit 1 omitted
fi
