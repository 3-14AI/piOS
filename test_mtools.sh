#!/bin/bash
set -e
dd if=/dev/zero of=test.img bs=1M count=64
parted -s test.img mklabel gpt
parted -s test.img mkpart ESP fat32 1MiB 63MiB
parted -s test.img set 1 esp on

mformat -i test.img@@1048576 -F ::
mmd -i test.img@@1048576 ::/EFI
mmd -i test.img@@1048576 ::/EFI/BOOT
echo "hello" > test_file.txt
mcopy -i test.img@@1048576 test_file.txt ::/EFI/BOOT/BOOTX64.EFI
mdir -i test.img@@1048576 -/ ::
