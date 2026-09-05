# piOS - Global Roadmap for the Next Phase: Post-1.0 AI Native Capabilities

Currently, piOS has completed its base architecture. We have successfully implemented:
- Bootable ISO, hardware enumeration, and base drivers
- Full network stack with smoltcp, WASM package manager, and userland coreutils
- Foundational AI capabilities: Inference runtime, sys optimizer, LLM inference abstraction
- Next-Gen OS features: Natural language desktop simulation, AI predictive app preloader, AI IDS/IPS, and autonomous closed-loop driver synthesis.

To achieve the overarching goal of a "fully functional operating system with natively integrated AI", we must replace these stubs with real implementations and scale up.

## Phase 21: Full System Autonomy and Optimization
- [x] **WP-132: Real-time Kernel Parameter Tuning.** Expand `sys_optimizer` to actually hook into kernel scheduler APIs instead of mocks.
- [x] **WP-133: Deep Neural Scheduler.** Completely replace the round-robin/CFS scheduler in the kernel with an inference-based predictor that prioritizes threads based on user intent.
- [x] **WP-134: Self-Healing Memory Management.** Use the AI monitoring to predict out-of-memory errors and proactively compact or swap memory in advance.

## Phase 22: Generative GUI and Desktop Experience
- [x] **WP-135: Generative UI Compositor.** Enhance `slint_gui` to construct windows and UI elements completely on the fly based on LLM outputs from `nl_desktop`.
- [x] **WP-136: Native Hardware Acceleration.** Replace `VirtioGpu` mocks with full `virglrenderer` and `amdgpu`/`intel` backend for actual hardware 3D and compute offload of AI tasks.
- [x] **WP-137: Audio/Voice Assistant Integration.** Hook up the `sound.rs` driver to a continuous voice recognition loop, allowing pure hands-free operation.

## Phase 23: Complete Developer Loop and Ecosystem
- [x] **WP-138: Package Repository Expansion.** Create an online repository of WASM components and integrate it fully with the Package Manager CLI to download and load applications dynamically.
- [x] **WP-139: In-OS IDE.** Build a userland application that allows writing, Verus-verifying, and Cranelift-compiling Rust code directly within piOS without needing a host system.
- [x] **WP-140: Multi-Agent Collaboration.** Extend `NlShell` and `sys_intent` so multiple agents can collaborate on complex tasks (e.g. one agent searches docs, another writes code, another verifies it).

## Phase 24: Real Hardware Integration
- [x] **WP-141: Real Hardware Boot.** Boot piOS on a physical x86-64 machine and debug NVMe, USB, and GPU driver issues.

## Phase 25: Swarm Synchronization
- [x] **WP-142: Swarm Packet Passing.** Implement true multi-node packet passing and agent broadcasting for A2A communication.

## Phase 26: Hardware Peripheral Expansion
- [x] **WP-143: USB Mass Storage Support.** Implement drivers for reading and writing to USB flash drives.
- [x] **WP-144: WiFi Capabilities.** Implement a WiFi driver and integrate it with the network stack.
- [x] **WP-145: Audio Framework.** Complete the audio subsystem and implement a basic AC97 or HDA driver.

## Phase 27: Persistent AI Core
- [x] **WP-146: Model Fine-tuning on Device.** Allow local models to learn from telemetry data by saving updated weights to disk.
- [x] **WP-147: Multi-User Contextual Memory.** Implement separate VectorDB spaces for different users based on capabilities and permissions.

## Phase 28: System-wide AI Observer
- [ ] **WP-148: Screen & Input Recording Daemon.** Implement a low-overhead service that captures screenshots and input events to create a continuous local history of user actions.
- [ ] **WP-149: Semantic Indexing of System State.** Feed the recorded history into the multimodal AI models to generate searchable embeddings, allowing users to query past system states (e.g., "What was that website I was looking at yesterday?").

## Phase 29: Dynamic Application Generation
- [ ] **WP-150: JIT WASM Synthesis.** Extend `nl_sh` and the internal compiler so that when a user asks for an app that doesn't exist, the AI generates the Rust code, verifies it with Verus, compiles to WASM, and runs it on the fly.
- [ ] **WP-151: Ephemeral Sandboxing.** Ensure dynamically generated apps run in strict, isolated WASM sandboxes that are automatically destroyed after use.

## Phase 30: AI-Native File System
- [ ] **WP-152: SemanticFS Implementation.** Implement a new VFS driver where files are organized not just hierarchically, but as a graph of semantic relationships, allowing path resolution via natural language queries (e.g., `/semantic/recent-receipts/`).
- [ ] **WP-153: Predictive Prefetching.** Use AI to predict which files the user will need next and preload them into RAM before they are explicitly requested.

## Phase 31: Advanced Hardware & Power Management
- [ ] **WP-154: AI Power Governor.** Implement a power management subsystem that learns usage patterns to optimize CPU states, screen brightness, and peripheral power states proactively.
- [ ] **WP-155: Bluetooth Stack.** Implement a basic Bluetooth HCI driver and integrate it with the network stack.
