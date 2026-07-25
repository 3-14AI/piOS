#!/bin/bash
set -e

# Run mkimage.sh to get the disk.img (ESP image) first
./tools/mkimage.sh

# Now convert the ESP image into a bootable ISO using xorriso
echo "Building ISO..."
ISO_DIR="target/iso"
ISO_NAME="target/pios.iso"

mkdir -p "$ISO_DIR"
cp target/disk.img "$ISO_DIR/efiboot.img"

# We use the generated disk.img (which contains a FAT32 ESP with BOOTX64.EFI)
# as the El Torito boot image for the ISO.
xorriso -as mkisofs \
    -r -V "piOS" \
    -e efiboot.img \
    -no-emul-boot \
    -isohybrid-gpt-basdat \
    -isohybrid-apm-hfsplus \
    -o "$ISO_NAME" \
    "$ISO_DIR"

# Alternative boot structure if xorriso El Torito doesn't map directly in this QEMU setup:
# We'll just rely on the ESP for UEFI
