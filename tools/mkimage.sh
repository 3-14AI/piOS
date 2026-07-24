#!/bin/bash
set -e

# Build the kernel
echo "Building kernel..."
./tools/build_kernel.sh

# Build hello_world WASM app
echo "Building hello_world WASM app..."
cargo build -p hello_world --target wasm32-wasip1 --release

# Create initrd
echo "Creating initrd..."
INITRD_DIR="target/initrd"
INITRD_IMG="target/initrd.img"
mkdir -p "$INITRD_DIR"
cp target/wasm32-wasip1/release/hello_world.wasm "$INITRD_DIR/"
cd "$INITRD_DIR"
find . | cpio -o -H newc > "../initrd.img"
cd ../..

# Configuration
IMAGE_NAME="target/disk.img"
IMAGE_SIZE_MB=64
ESP_START_MB=1
ESP_SIZE_MB=63
ESP_OFFSET_BYTES=$((ESP_START_MB * 1024 * 1024))
KERNEL_EFI="target/x86_64-unknown-uefi/release/kernel.efi"

echo "Creating raw GPT disk image of ${IMAGE_SIZE_MB}MB..."
dd if=/dev/zero of="$IMAGE_NAME" bs=1M count="$IMAGE_SIZE_MB" status=none

echo "Partitioning image with GPT and ESP..."
parted -s "$IMAGE_NAME" mklabel gpt
parted -s "$IMAGE_NAME" mkpart ESP fat32 "${ESP_START_MB}MiB" "${ESP_SIZE_MB}MiB"
parted -s "$IMAGE_NAME" set 1 esp on

echo "Formatting ESP as FAT32..."
# Format the partition at the offset
mformat -i "${IMAGE_NAME}@@${ESP_OFFSET_BYTES}" -F ::

echo "Creating EFI directory structure..."
mmd -i "${IMAGE_NAME}@@${ESP_OFFSET_BYTES}" ::/EFI
mmd -i "${IMAGE_NAME}@@${ESP_OFFSET_BYTES}" ::/EFI/BOOT

echo "Copying kernel.efi to BOOTX64.EFI..."
mcopy -i "${IMAGE_NAME}@@${ESP_OFFSET_BYTES}" "$KERNEL_EFI" ::/EFI/BOOT/BOOTX64.EFI

echo "Copying initrd.img to ESP..."
mcopy -i "${IMAGE_NAME}@@${ESP_OFFSET_BYTES}" "$INITRD_IMG" ::/initrd.img

echo "Bootable image created successfully at $IMAGE_NAME"
