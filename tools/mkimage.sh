#!/bin/bash
set -e

# Build the kernel
echo "Building kernel..."
./tools/build_kernel.sh

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

echo "Bootable image created successfully at $IMAGE_NAME"
