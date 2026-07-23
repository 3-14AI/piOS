# piOS (Self-Evolving AI-Native OS) - Global TODO

This file tracks the overarching goals and next phases for piOS, moving from a fully implemented architectural scaffolding to a functional, bootable operating system with a native AI feedback loop.

## Phase 7: Bootable ISO and Real Hardware Testing
- [ ] **WP-076: Build System for Bootable Image.** Create an automated script (`tools/mkimage.sh`) that compiles the UEFI bootloader, kernel, and an initial ramdisk (initrd) containing WASM components, outputting a bootable `.iso` or `.img`.
- [ ] **WP-077: Initial Ramdisk (initrd) implementation.** Implement parsing of a basic initramfs in the kernel to load critical drivers before the root VFS is mounted.
- [ ] **WP-078: Bare-Metal x86-64 Execution.** Boot the generated ISO on a real physical x86-64 machine. Debug and fix any CPU feature mismatches, UEFI handoff issues, or ACPI parsing panics.
- [ ] **WP-079: Hardware-backed NVMe & USB.** Verify that the NVMe and USB XHCI drivers successfully enumerate and interact with physical storage and input devices on a real machine (not QEMU).

## Phase 8: Core Userland and System Applications
- [ ] **WP-080: Libc/WASI compatibility layer.** Ensure `wasi-libc` fully supports the kernel's WASI-core implementation, allowing standard C/C++ and Rust programs (compiled to `wasm32-wasip1`) to run without modification.
- [ ] **WP-081: Package Manager Implementation.** Build the command-line interface for the `package_manager` service to fetch, install, and resolve dependencies for WASM apps from a remote repository.
- [ ] **WP-082: Minimal Coreutils.** Implement basic system utilities (`ls`, `cat`, `mkdir`, `rm`, `ps`, `kill`) as WASM components.
- [ ] **WP-083: Advanced NL-Shell (Natural Language Shell).** Upgrade the `NL-Shell` to correctly parse complex commands, pipe data between WASM instances, and effectively utilize the `sys_intent` semantic layer.

## Phase 9: AI Autopoiesis & Verus Self-Verification
- [ ] **WP-084: On-Device LLM (WASI-NN Execution).** Integrate a lightweight local model (e.g., Llama.cpp or Mistral via WASI-NN) that can run entirely within the piOS userland using the VirtIO-GPU driver.
- [ ] **WP-085: Semantic System Logs.** Implement the vector database logger, converting kernel panic and warning logs into embeddings for the AI to query when debugging itself.
- [ ] **WP-086: The Self-Coding Loop (Driver Synthesis).** Demonstrate a closed-loop scenario: System detects unknown USB device -> LLM writes a basic Rust driver -> Verus (running in WASM) proves memory safety -> Driver is compiled via Cranelift WASM backend -> Driver is hot-loaded.
- [ ] **WP-087: SMT Solver Integration.** Successfully port and execute an SMT solver (like Z3 or CVC5) in the WASM userland to support the on-device Verus verifier.

## Phase 10: GUI, Compositor and Daily-Driver Polish
- [ ] **WP-088: WGPU Compositor and Wayland-like protocol.** Mature the `wgpu_compositor` to handle multiple overlapping application windows, input routing, and damage tracking.
- [ ] **WP-089: Generative UI (Slint).** Implement a dynamic desktop environment using Slint where the AI can generate or modify UI layouts based on user context.
- [ ] **WP-090: Networking & Web.** Implement a basic DNS resolver and HTTP client in userland, eventually paving the way for a WASM-based web browser.
- [ ] **WP-091: User Documentation & Installer.** Write a comprehensive user guide, and create a live-USB GUI installer that formats disks, sets up secure boot, and installs piOS.
