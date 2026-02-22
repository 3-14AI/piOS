default:
    @just --list

build:
    cargo build --workspace

# Build the kernel specifically for UEFI target
build-kernel:
    cargo build -p kernel --target x86_64-unknown-uefi -Zbuild-std=core,compiler_builtins,alloc --release

test:
    cargo test --workspace --exclude kernel

clean:
    cargo clean

verify:
    @echo "Running verification..."
    # ./tools/verus-runner/verify.sh

run: build-kernel
    @echo "Running QEMU..."
    # Placeholder: In Phase 1 we will create a proper disk image with ESP
    @echo "To run, we need to create a disk image containing the EFI application."
    @echo "Build artifact is at target/x86_64-unknown-uefi/release/kernel.efi"

setup:
    ./tools/install_deps.sh
    ./tools/setup_verus.sh
