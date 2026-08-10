# piOS - Global Roadmap for the Next Phase

Currently, piOS consists mostly of architectural scaffolding and mocks. To achieve the overarching goal of a "fully functional operating system with natively integrated AI", we must replace these stubs with real implementations.

## Phase 15: Hardware Enablement and Boot
- [x] **WP-111: Real Bootable Image Generation.** Fix `tools/mkimage.sh` to correctly compile the UEFI bootloader, kernel, and initrd, and output a valid `.iso` (e.g. using `xorriso`).
- [x] **WP-112: Bare-Metal x86-64 Execution.** Boot the ISO on real hardware. Fix CPU feature mismatches, UEFI handoff bugs, and ACPI parsing issues.
- [x] **WP-113: Real Storage Drivers.** Replace the mock implementations in `kernel/src/nvme.rs` and `kernel/src/virtio_blk.rs` with functional drivers that can enumerate devices via PCI, set up submission/completion queues, and read/write blocks.
- [x] **WP-114: Real USB and HID.** Replace stubs in `kernel/src/xhci.rs`, `kernel/src/ehci.rs`, and `kernel/src/input.rs` to process actual USB Request Blocks (URBs) and generate valid HID events.

## Phase 16: File Systems and Execution
- [x] **WP-115: VFS and File Systems.** Implement actual FAT32 and ext4 drivers on top of the block devices instead of simple stubs.
- [ ] **WP-116: Real WASM Execution.** Ensure that the kernel's WASM runtime (e.g., via Cranelift/Winch) can load and execute standalone `wasm32-wasip1` binaries correctly.
- [ ] **WP-117: Libc/WASI compatibility.** Ensure `wasi-libc` fully supports the kernel's WASI-core implementation.

## Phase 17: Userland and Connectivity
- [ ] **WP-118: Real Network Stack.** Replace the `MockDevice` in `userland/services/net_stack` with a real interface communicating with `virtio_net` or physical NICs. Get DHCP and DNS fully functional.
- [ ] **WP-119: Functional Coreutils.** Implement functional utilities (`ls`, `cat`, `mkdir`, `ps`) as WASM components.
- [ ] **WP-120: Package Manager CLI.** Implement real downloading and installation of WASM apps from a remote repository.

## Phase 18: Real Autopoiesis
- [ ] **WP-121: On-Device LLM (WASI-NN).** Replace the `mock_model` in `libs/inference_runtime` with actual model loading and inference (e.g., Llama.cpp or Mistral via WASI-NN).
- [ ] **WP-122: Semantic System Logs.** Implement real vector database functionality to convert system logs into embeddings.
- [ ] **WP-123: Closed-Loop Driver Synthesis.** Demonstrate the actual end-to-end loop: LLM writes Rust code -> Verus proves it -> Cranelift compiles it to WASM -> Kernel hot-swaps it.
