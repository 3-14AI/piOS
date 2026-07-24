#!/bin/bash
set -e

# Configuration
IMAGE_NAME="target/disk.img"
ESP_START_MB=1
ESP_OFFSET_BYTES=$((ESP_START_MB * 1024 * 1024))

echo "Running tools/mkimage.sh..."
./tools/mkimage.sh

echo "Verifying image structure..."
if [ ! -f "$IMAGE_NAME" ]; then
    echo "Error: Image $IMAGE_NAME was not created."
    exit 1
fi

echo "Verifying BOOTX64.EFI exists in the image..."
if ! mdir -i "${IMAGE_NAME}@@${ESP_OFFSET_BYTES}" -/ :: | grep -q "BOOTX64"; then
    echo "Error: BOOTX64.EFI not found in the image."
    exit 1
fi

echo "Verifying initrd.img exists in the image..."
if ! mdir -i "${IMAGE_NAME}@@${ESP_OFFSET_BYTES}" -/ :: | grep -qi "initrd.*img"; then
    echo "Error: initrd.img not found in the image."
    exit 1
fi

echo "Image verification tests passed!"
