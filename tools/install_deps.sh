#!/bin/bash
set -e

echo "Installing dependencies..."

if command -v apt-get &> /dev/null; then
    sudo apt-get update
    sudo apt-get install -y qemu-system-x86 ovmf llvm clang lld curl git build-essential
else
    echo "Warning: apt-get not found. Please ensure qemu-system-x86_64, ovmf, llvm, clang are installed."
fi

echo "Installing rust components..."
rustup component add llvm-tools-preview rust-src

if ! command -v just &> /dev/null; then
    echo "Installing just..."
    curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/.cargo/bin || echo "Failed to install just. Please install manually."
else
    echo "just is already installed."
fi

echo "Dependencies installed successfully."
