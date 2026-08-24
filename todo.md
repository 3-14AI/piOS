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
- [ ] **WP-135: Generative UI Compositor.** Enhance `slint_gui` to construct windows and UI elements completely on the fly based on LLM outputs from `nl_desktop`.
- [ ] **WP-136: Native Hardware Acceleration.** Replace `VirtioGpu` mocks with full `virglrenderer` and `amdgpu`/`intel` backend for actual hardware 3D and compute offload of AI tasks.
- [ ] **WP-137: Audio/Voice Assistant Integration.** Hook up the `sound.rs` driver to a continuous voice recognition loop, allowing pure hands-free operation.

## Phase 23: Complete Developer Loop and Ecosystem
- [ ] **WP-138: Package Repository Expansion.** Create an online repository of WASM components and integrate it fully with the Package Manager CLI to download and load applications dynamically.
- [ ] **WP-139: In-OS IDE.** Build a userland application that allows writing, Verus-verifying, and Cranelift-compiling Rust code directly within piOS without needing a host system.
- [ ] **WP-140: Multi-Agent Collaboration.** Extend `NlShell` and `sys_intent` so multiple agents can collaborate on complex tasks (e.g. one agent searches docs, another writes code, another verifies it).
