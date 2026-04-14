# TODO 1: Core, Boot, and Drivers

- [x] **Верификация — частично костыль**
  WP-053 прямо описывает задачу: убрать `|| true` из шага `Verify Kernel with Verus` в CI — то есть весь пайплайн верификации искусственно помечался как пройденный, даже при ошибках. [github](https://github.com/3-14AI/piOS/blob/main/todo.md) Это означает, что математические доказательства корректности ядра могут быть неполными.

- [x] **Нет реального железа — только QEMU/VirtIO**
  Все драйверы (WP-016, WP-017, WP-018) написаны под VirtIO — это виртуальные устройства QEMU. [github](https://github.com/3-14AI/piOS/blob/main/todo.md) Запустить систему на физическом x86-64 ПК или ARM — отдельная нерешённая задача.

- [ ] **Документация декларирована, но не написана**
  WP-050 — «финальная стабилизация и документация» — помечен как выполненный [github](https://github.com/3-14AI/piOS/blob/main/todo.md) , но папка `docs/` в репозитории есть, а её содержимое не раскрывается в README. piOS — **амбициозный архитектурный проект-концепт**, написанный с участием AI-агентов (на что намекает сама `AGENTS.md`). Большинство компонентов реализованы как скаффолдинг или заглушки. Для превращения в полноценную ОС не хватает прежде всего: работающей LLM внутри системы, реальных аппаратных драйверов, закрытого цикла самописания, и пользовательских приложений, которые можно запустить без QEMU.

- [ ] **Полная интеграция bootloader и создание bootable-образа**
  WP-001 содержит только UEFI boot stub. Нет готового `mkimage`, EFI-образа для QEMU/real hardware, GRUB-подобного меню или Secure Boot chain с подписью всего образа.

- [x] **Конкретные файловые системы поверх VFS**
  VFS и WASI-filesystem есть, но нет реализации ext4 / btrfs / FAT32 / ZFS. Нет монтирования реального диска, fsck, journaling и snapshot-ов.

- [x] **Драйверы реального железа (не только VirtIO)**
  VirtIO-blk/net/gpu реализованы, PCI-enumerator есть. Отсутствуют: NVMe, USB (xhci/ehci), GPU (amdgpu/intel), Wi-Fi/Bluetooth, sound (HDA), input (HID). Нет ACPI tables parsing и power management.

- [x] **Реальное тестирование и CI/CD для hardware**
  Все тесты — unit + QEMU. Нет hardware CI, stress-тестов, fuzzing драйверов и formal verification coverage reports.

- [ ] **Собрать первый bootable EFI-образ и запустить в QEMU с реальным диском.**

- [ ] **Реализовать хотя бы FAT32 + ext4 поверх VFS.**

- [ ] **Добавить NVMe + USB-драйверы (минимум).**

- [ ] **Выпустить первый pre-release 0.1 с инструкцией «как запустить».**
  По todo.md проект технически «завершён» на бумаге. На практике это очень продвинутый verified microkernel + WASM/AI-фреймворк, но до полноценной ОС (как Redox в 2020-х или Linux в 1995-м) не хватает именно «клея» — bootable-образа, реальных драйверов, installer и user-facing polish. После закрытия этих пунктов piOS действительно сможет претендовать на звание первой self-evolving AI-native ОС.

- [ ] **Полноценного планировщика потоков.**
  Полноценного планировщика потоков.

- [ ] **Безопасного менеджера виртуальной и физической памяти.**
  Безопасного менеджера виртуальной и физической памяти.

- [ ] **Механизмов межпроцессного взаимодействия (IPC) без риска взаимных блокировок (deadlocks) и состояний гонки (race conditions).**
  Механизмов межпроцессного взаимодействия (IPC) без риска взаимных блокировок (deadlocks) и состояний гонки (race conditions).

- [ ] **Драйверов для современных файловых систем.**
  Драйверов для современных файловых систем.

### Fuzzer / CI Fixes to do:
- [ ] Install `cargo-fuzz` via `cargo install cargo-fuzz` directly in the `tools/fuzz_drivers.sh` script or CI environment instead of using native `rustc` compiler flags, since `-fsanitize-coverage=trace-pc-guard` is deprecated in modern libFuzzer versions.
- [ ] Or alternatively, run `cargo +nightly rustc --bin kernel-fuzzer -- -Z sanitizer=address -Z sanitizer=fuzzer` inside the `fuzzer` folder directly rather than passing `-C passes=sancov-module` directly.
