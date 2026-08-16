# piOS - Global Roadmap for the Next Phase

Currently, piOS consists mostly of architectural scaffolding and mocks. To achieve the overarching goal of a "fully functional operating system with natively integrated AI", we must replace these stubs with real implementations.

## Phase 15: Hardware Enablement and Boot
- [x] **WP-111: Real Bootable Image Generation.** Fix `tools/mkimage.sh` to correctly compile the UEFI bootloader, kernel, and initrd, and output a valid `.iso` (e.g. using `xorriso`).
- [x] **WP-112: Bare-Metal x86-64 Execution.** Boot the ISO on real hardware. Fix CPU feature mismatches, UEFI handoff bugs, and ACPI parsing issues.
- [x] **WP-113: Real Storage Drivers.** Replace the mock implementations in `kernel/src/nvme.rs` and `kernel/src/virtio_blk.rs` with functional drivers that can enumerate devices via PCI, set up submission/completion queues, and read/write blocks.
- [x] **WP-114: Real USB and HID.** Replace stubs in `kernel/src/xhci.rs`, `kernel/src/ehci.rs`, and `kernel/src/input.rs` to process actual USB Request Blocks (URBs) and generate valid HID events.

## Phase 16: File Systems and Execution
- [x] **WP-115: VFS and File Systems.** Implement actual FAT32 and ext4 drivers on top of the block devices instead of simple stubs.
- [x] **WP-116: Real WASM Execution.** Ensure that the kernel's WASM runtime (e.g., via Cranelift/Winch) can load and execute standalone `wasm32-wasip1` binaries correctly.
- [x] **WP-117: Libc/WASI compatibility.** Ensure `wasi-libc` fully supports the kernel's WASI-core implementation.

## Phase 17: Userland and Connectivity
- [x] **WP-118: Real Network Stack.** Replace the `MockDevice` in `userland/services/net_stack` with a real interface communicating with `virtio_net` or physical NICs. Get DHCP and DNS fully functional.
- [x] **WP-119: Functional Coreutils.** Implement functional utilities (`ls`, `cat`, `mkdir`, `ps`) as WASM components.
- [x] **WP-120: Package Manager CLI.** Implement real downloading and installation of WASM apps from a remote repository.

## Phase 18: Real Autopoiesis
- [ ] **WP-121: On-Device LLM (WASI-NN).** Replace the `mock_model` in `libs/inference_runtime` with actual model loading and inference (e.g., Llama.cpp or Mistral via WASI-NN).
- [x] **WP-122: Semantic System Logs.** Implement real vector database functionality to convert system logs into embeddings.
- [ ] **WP-123: Closed-Loop Driver Synthesis.** Demonstrate the actual end-to-end loop: LLM writes Rust code -> Verus proves it -> Cranelift compiles it to WASM -> Kernel hot-swaps it.

## Phase 19: AI Agent Loop and Next-Gen IPC
- [x] **WP-124: IPC Refactoring.** Replace current rendezvous channels with a more robust message-passing interface that supports WASM components efficiently.
- [x] **WP-125: Dynamic Agent Spawning.** Enable the natural language shell (NL-Shell) to spawn agent WASM components on demand and manage their lifecycles.
- [x] **WP-126: Continuous Learning Loop.** Integrate telemetry and semantic logs deeply into the LLM context to allow agents to analyze faults and generate Rust code fixes autonomously.
- [x] **WP-127: System Calls for Autopoiesis.** Add `wasi_ephemeral_compiler` or similar syscalls so an agent can invoke Cranelift dynamically from userland.

## Phase 20: Next Generation AI OS Features
- [ ] **WP-128: Autonomous System Optimization.** Implement a daemon that continuously monitors system performance and dynamically adjusts scheduling, memory allocation, and power management using AI.
- [ ] **WP-129: Zero-Trust Security with AI.** Develop an AI-driven intrusion detection and prevention system (IDS/IPS) that learns normal system behavior and blocks anomalous activities in real-time.
- [ ] **WP-130: Natural Language Desktop Environment.** Build a GUI desktop environment where all interactions (window management, file operations, settings) can be controlled entirely through natural language commands and multimodal inputs.
- [ ] **WP-131: Predictive Application Preloading.** Implement a system that predicts which applications the user is likely to launch next based on historical usage patterns and preloads them into memory for instant startup.
