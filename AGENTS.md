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
