# piOS - Global Roadmap for the Next Phase: Swarm Intelligence & Physical Adoption

Currently, piOS has completed its base architecture and the integration of AI-native loops within a single node context. We have successfully implemented:
- Bootable ISO, hardware enumeration, and base drivers
- Full network stack with smoltcp, WASM package manager, and userland coreutils
- Foundational AI capabilities: Inference runtime, sys optimizer, LLM inference abstraction
- Next-Gen OS features: Natural language desktop simulation, AI predictive app preloader, AI IDS/IPS, and autonomous closed-loop driver synthesis.
- Complete Developer Ecosystem: Multi-agent collaboration, In-OS IDE, Package repository connections.

To achieve the overarching goal of a "fully functional operating system with natively integrated AI", we must transition from single-node mock systems to physical hardware integration and multi-node swarm intelligence.

## Phase 24: Real Hardware Adoption and Stabilization
- [ ] **WP-141: Bare Metal Bootloader Integration.** Replace UEFI QEMU stubs with a fully signed GRUB or systemd-boot configuration that supports booting piOS natively on modern x86_64 motherboards.
- [ ] **WP-142: Physical Device Drivers.** Implement true hardware interaction for the `AmdGpu` and `Intel` graphics backends instead of mock interfaces in `virtio_gpu.rs`, including MMIO memory mapping and PCIe register initialization.
- [ ] **WP-143: Native File System Mounts.** Expand `vfs` to perform actual disk mounting of physical NVMe/SATA drives using the ext4 or btrfs driver, transitioning away from in-memory arrays.

## Phase 25: Distributed Swarm Intelligence
- [ ] **WP-144: Swarm State Synchronization.** Introduce an agent in `nl_sh` that synchronizes system state (processes, memory limits) continuously with other piOS instances on the LAN via `A2AMessage` broadcasts.
- [ ] **WP-145: Distributed AI Inference.** Modify `inference_runtime` to split tensor computation across multiple network nodes using `wasi_ephemeral_nn` and RPC if the local node lacks GPU power.
- [ ] **WP-146: Global Anomaly Defense.** Enhance `ids_ips` to build a shared network-wide anomaly graph, blocking attacks on the entire swarm when one node detects a threat.
