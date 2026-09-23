# Инструкции для Агентов (AGENTS.md)

## Работа с задачами (TODO)
1. Все задачи находятся в файле `todo.md` в корне репозитория.
2. После выполнения пункта, отметьте его как выполненный (например, заменив `[ ]` на `[x]`, добавив `[x]` в начало строки, или зачеркнув).
3. **ВАЖНО:** После завершения текущего пункта, прочитайте следующий пункт. Если необходимо, скорректируйте его описание или требования на основе полученного опыта и текущего состояния проекта.

## Система заметок (Memo)
1. Используйте папку `memo/` для сохранения важных заметок, архитектурных решений и наблюдений.
2. Обновляйте `memo/mapping.md`, чтобы отразить, какие темы покрываются в каких файлах заметок.
3. Перед началом работы проверяйте `memo/`, чтобы получить контекст от предыдущих этапов.

## Глобальные цели (AI-Native OS)
1. **Фокус на реальном железе и загрузке:** Базовая архитектура и скаффолдинг завершены. Приоритетом является создание загрузочного образа (ISO), выполнение на реальном железе и отладка драйверов (NVMe, USB, GPU).
2. **Замыкание петли Autopoiesis:** Реализуйте и стабилизируйте инструменты для самонаписания ОС (Cranelift/Winch WASM backend, WASI-NN inference, on-device Verus).
3. **Развитие Userland:** Создавайте полноценные WASM-приложения (coreutils, package manager) и развивайте графический интерфейс (wgpu_compositor, Slint).

## Test Coverage Constraints (Tarpaulin)
1. GitHub CI uses `cargo tarpaulin` to enforce strict code coverage limits (e.g. `87%` for the entire workspace).
2. If `cargo tarpaulin` fails locally due to coverage dropping below the threshold after you add new code, **DO NOT** attempt to bypass the check by writing duplicate or empty tests.
3. Instead, either write extensive real mock tests to hit the target coverage, or adjust the `--fail-under` flag in `.github/workflows/ci.yml` temporarily if approved.

## The Next Phase: Implementing Real Functionality
1. **Transitioning from Mocks to Real Code**: The project has reached a point where most architectural components are in place as mock structures or stubs. The new primary goal is to implement functional code for these components.
2. **Prioritize Boot and Hardware**: Start with making the OS bootable (ISO generation) and getting storage/input drivers to work on real hardware.
3. **Continuous Testing on Real Infrastructure**: As mocks are removed, ensure that tests simulate real hardware interactions (e.g., using `[u32; N]` arrays for MMIO) rather than just toggling booleans.

## Работа с внешними модулями (Third-Party)
1. **Verus**: When compiling kernel with `verus_builtin`, ensure the `/tools/verus` folder exists and contains compiled files, or run `./tools/install_verus.sh`.
2. **Candle**: When testing features using `candle-core` in inference_runtime, remember to enable the feature during cargo test `cargo test --features candle-core`.
# Updated AI/OS Goals
We have now implemented all current features and reached the state of a fully functional OS with natively integrated AI. We should proceed to the next stage.

## Phase 24 and 25 Focus
For Phases 24 and 25 (Swarm and Real Hardware), focus on real hardware integration and actual network packet passing, moving away from simulated interfaces.

## Swarm Implementation Details
1. **A2AMessage Network Serialization:** When dispatching `A2AMessage` across the network, they must be properly serialized to a `&[u8]` slice and broadcast over the `WasmNetStack` UDP capabilities (port 9999) using `add_udp_broadcast_socket` and `send_udp_broadcast`.

## Phases 28-31 Focus: AI-Native Paradigms
1. **System-wide AI Observer (Phase 28):** Focus on building low-overhead daemons for continuous system state recording (inputs, screen bounds) to feed into the multimodal AI. Ensure privacy boundaries are maintained.
2. **Dynamic Application Generation (Phase 29):** Agents must focus on integrating `wasi_compiler` and `sys_intent` deeply so that missing capabilities trigger live code synthesis and ephemeral sandbox execution.
3. **AI-Native File System (Phase 30):** Shift from POSIX tree structures to graph-based, semantic representations of files. Paths like `/semantic/...` should invoke LLM embedding similarity searches instead of simple inode lookups.
4. **Hardware Continuity (Phase 31):** Continue pushing for real hardware support (Power Management, Bluetooth HCI) and replace remaining stub drivers with functional MMIO/DMA code.

## Phases 33-36 Focus: Advanced AI-Native Paradigms
1. **Multi-modal Interfaces (Phase 33):** When implementing vision and advanced voice integrations, prioritize real-time local model execution using `inference_runtime` with `#![no_std]` constraints. Ensure that memory usage is bounded for these large models.
2. **Proactive Background Agents (Phase 34):** Agents should be built as lightweight WASM components. Ensure they use non-blocking scheduling via `set_scheduler_quantum` and securely share context via the isolated semantic VFS namespaces rather than direct memory access.
3. **Formally Verified AI Code (Phase 35):** When an agent generates code dynamically, the validation step **must** invoke Verus checks. If the generated code fails formal proofs, the loop must iterate, correct the logic based on Verus error output, and retry. Do not bypass the verification step.
4. **Distributed Swarm Intelligence (Phase 36):** Testing swarm consensus and state migration requires spinning up multiple test environments passing `A2AMessage` via `WasmNetStack` UDP capabilities. Ensure state serialization uses zero-copy abstractions where possible to reduce network latency during cross-node migration.

## Swarm Consensus and Migration

*   **Network Integration:** New features that require interaction across multiple OS instances must utilize the `WasmNetStack`. Multi-node tasks, agent elections, and distributed state use the Raft-inspired Swarm Consensus (`userland/services/net_stack/src/swarm_consensus.rs`).
*   **Cross-Node Migration:** Agent migration is supported using `MigrationMessage` within `WasmNetStack` (`userland/services/net_stack/src/agent_migration.rs`), sending agent memory state across the UDP broadcast network for decentralized scaling.

## Verifiable Standard Library
*   **WASM Enhancements:** The kernel exposes verifiable standard library wrappers (e.g., `VerifiableVec`, `VerifiableMap`) in `kernel/src/wasm/verus_std_env.rs` for AI-generated code. Use these abstractions when generating applications to ensure formal verifiability with Verus before compiling with Cranelift.

* The `userland/services/net_stack` module has been expanded to include Distributed Shared Memory (`distributed_memory.rs`), Zero Trust Agent Sandboxing (`zero_trust.rs`), AI-driven Malware Analysis (`malware_analysis.rs`), Collaborative IDE (`collaborative_ide.rs`), and Natural Language Debugger (`nl_debugger.rs`).
* Always ensure that custom WASM services utilizing AI models or natural language processing correctly implement `#![no_std]` paradigms and verify dependencies before exposing new capabilities to userland applications.
