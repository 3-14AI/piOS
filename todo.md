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
- [x] **WP-148: Screen & Input Recording Daemon.** Implement a low-overhead service that captures screenshots and input events to create a continuous local history of user actions.
- [x] **WP-149: Semantic Indexing of System State.** Feed the recorded history into the multimodal AI models to generate searchable embeddings, allowing users to query past system states (e.g., "What was that website I was looking at yesterday?").

## Phase 29: Dynamic Application Generation
- [x] **WP-150: JIT WASM Synthesis.** Extend `nl_sh` and the internal compiler so that when a user asks for an app that doesn't exist, the AI generates the Rust code, verifies it with Verus, compiles to WASM, and runs it on the fly.
- [x] **WP-151: Ephemeral Sandboxing.** Ensure dynamically generated apps run in strict, isolated WASM sandboxes that are automatically destroyed after use.

## Phase 30: AI-Native File System
- [x] **WP-152: SemanticFS Implementation.** Implement a new VFS driver where files are organized not just hierarchically, but as a graph of semantic relationships, allowing path resolution via natural language queries (e.g., `/semantic/recent-receipts/`).
- [x] **WP-153: Predictive Prefetching.** Use AI to predict which files the user will need next and preload them into RAM before they are explicitly requested.

## Phase 31: Advanced Hardware & Power Management
- [x] **WP-154: AI Power Governor.** Implement a power management subsystem that learns usage patterns to optimize CPU states, screen brightness, and peripheral power states proactively.
- [x] **WP-155: Bluetooth Stack.** Implement a basic Bluetooth HCI driver and integrate it with the network stack.

## Phase 32: System Stabilization and Polish
- [x] **WP-156: Kernel Fuzzer Polish.** Implement robust automated fuzzing targeting WASM components and driver boundaries to assure code robustness.
- [x] **WP-157: Expand WASM App Ecosystem.** Begin porting standard POSIX and POSIX-adjacent C/C++ libraries and applications into native WASM applications.
- [x] **WP-158: Comprehensive User Documentation.** Finalize end-to-end user-facing documentation to demonstrate the core capabilities and architecture of the newly finalized system.

## Phase 33: Multi-modal Interfaces
- [x] **WP-159: Vision Capabilities Integration.** Integrate the vision capabilities into the generative GUI to process visual inputs (e.g., webcam) directly in the UI.
- [x] **WP-160: Advanced Voice Assistant.** Enhance the continuous voice recognition loop with a dedicated local audio processing model for reliable wake-word detection and intent mapping.

## Phase 34: Proactive Background Agents
- [x] **WP-161: Proactive Task Agents.** Develop background WASM services (agents) that monitor system state and user intent to autonomously execute routine tasks, such as system cleanup, without direct user intervention.
- [x] **WP-162: Agent Context Sharing.** Implement a standard context-sharing mechanism for agents, enabling them to share findings and coordinate tasks seamlessly in the background.

## Phase 35: Formally Verified AI-generated Code
- [x] **WP-163: AI-Verus Integration Loop.** Tighten the loop for JIT WASM synthesis by guaranteeing that AI-generated Rust code strictly passes Verus formal verification checks before it is compiled and executed.
- [ ] **WP-164: Verified Standard Library Enhancements.** Extend the verifiable API surface exposed to the WASM compiler, ensuring that more complex AI-generated applications can be formally proven safe.

## Phase 36: Distributed Swarm Intelligence
- [ ] **WP-165: Swarm Consensus Mechanism.** Develop a consensus protocol within `WasmNetStack` to allow multiple nodes in a swarm to vote and make collective decisions on distributed task scheduling.
- [ ] **WP-166: Cross-Node Agent Migration.** Implement the ability for a running WASM agent (and its memory state) to migrate seamlessly from one physical node to another across the network.
