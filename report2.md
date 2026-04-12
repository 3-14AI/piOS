**Что не хватает проекту piOS, чтобы стать полноценной операционной системой**

(на основе анализа репозитория: архитектура, структура, todo.md с 51 WP — все помечены [x], kernel, userland, WASM-компоненты и AI-слой)

### 🔴 Критические недостающие компоненты (boot → usability)

- [ ] **Полная интеграция bootloader и создание bootable-образа**  
  WP-001 содержит только UEFI boot stub. Нет готового `mkimage`, EFI-образа для QEMU/real hardware, GRUB-подобного меню или Secure Boot chain с подписью всего образа.

- [ ] **Конкретные файловые системы поверх VFS**  
  VFS и WASI-filesystem есть, но нет реализации ext4 / btrfs / FAT32 / ZFS. Нет монтирования реального диска, fsck, journaling и snapshot-ов.

- [ ] **Драйверы реального железа (не только VirtIO)**  
  VirtIO-blk/net/gpu реализованы, PCI-enumerator есть. Отсутствуют: NVMe, USB (xhci/ehci), GPU (amdgpu/intel), Wi-Fi/Bluetooth, sound (HDA), input (HID). Нет ACPI tables parsing и power management.

- [ ] **Система инициализации и сервис-менеджер**  
  Нет systemd-подобного init (или собственного supervisor), unit-файлов, dependency graph, graceful shutdown и watchdog.

- [ ] **Система пакетов и управление приложениями**  
  Нет package manager (аналог apt/pacman/cargo для WASM-компонентов), репозитория, dependency resolution и sandboxed app store.

### 🟠 Недостающие компоненты для «живой» и удобной ОС

- [ ] **Полноценный installer / live-образ**  
  Нет скриптов/приложения для установки на реальное железо, dual-boot, partition manager и recovery mode.

- [ ] **Многопользовательская система + PAM-подобный модуль**  
  Capabilities и UID/GID есть, но нет login manager, sudo/su, session tracking и polkit-подобного авторизатора для AI-агентов.

- [ ] **Сетевой стек в production-ready состоянии**  
  Smoltcp портирован, но нет firewall, nftables-подобного, DNS resolver, DHCP client, VPN и Wi-Fi supplicant.

- [ ] **Поддержка нескольких архитектур**  
  Только x86-64 (UEFI). Для «piOS» логично добавить ARM64 (Raspberry Pi, Apple Silicon) + RISC-V.

- [ ] **Локализация, i18n и accessibility**  
  Нет поддержки кириллицы/unicode в NL-Shell, screen reader, high-contrast themes и input methods.

### 🟡 Пост-автопоэзисные и production-недостающие вещи

- [ ] **Реальное тестирование и CI/CD для hardware**  
  Все тесты — unit + QEMU. Нет hardware CI, stress-тестов, fuzzing драйверов и formal verification coverage reports.

- [ ] **Документация для конечного пользователя**  
  README — только для разработчиков. Нет user guide, man-страниц, quickstart и FAQ.

- [ ] **Экосистема приложений**  
  Есть Slint + генеративный UI, но нет портированных браузера, офисного пакета, media player и native WASM-приложений.

- [ ] **Мониторинг, логирование и observability в production**  
  OpenTelemetry и tracing есть, но нет journald-подобного лога, metrics exporter и dashboard.

- [ ] **Дистрибутивная инфраструктура**  
  Нет mirrors, signing keys, update mechanism (OTA), release notes и ISO-builder.

### Приоритетный TODO-лист (что делать прямо сейчас)

1. Собрать первый bootable EFI-образ и запустить в QEMU с реальным диском.
2. Реализовать хотя бы FAT32 + ext4 поверх VFS.
3. Добавить NVMe + USB-драйверы (минимум).
4. Сделать минимальный init + NL-Shell как default shell.
5. Запустить self-coding loop на реальном примере (генерация и hot-swap простого сервиса).
6. Выпустить первый pre-release 0.1 с инструкцией «как запустить».

**Вывод:**  
По todo.md проект технически «завершён» на бумаге. На практике это очень продвинутый verified microkernel + WASM/AI-фреймворк, но до полноценной ОС (как Redox в 2020-х или Linux в 1995-м) не хватает именно «клея» — bootable-образа, реальных драйверов, installer и user-facing polish. После закрытия этих пунктов piOS действительно сможет претендовать на звание первой self-evolving AI-native ОС.
