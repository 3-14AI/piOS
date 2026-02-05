# piOS - Self-Evolving AI OS

1. Введение и архитектурная философия
Настоящий отчет представляет собой детальный план разработки (Todo List) и архитектурное обоснование для создания операционной системы нового поколения, условно именуемой "piOS" .

Данная система объединяет в себе четыре передовые парадигмы, которые до настоящего момента развивались изолированно: безопасное системное программирование на Rust , формальная верификация корректности кода с использованием инструментария Verus , архитектура AI-Native (AIOS) , где искусственный интеллект является не приложением, а системным ресурсом, и концепция самонаписания кода (Self-coding) , позволяющая системе автономно развиваться и оптимизировать собственные компоненты.

**Цель разработки —** разрешить фундаментальное противоречие между надежностью (Reliability) и адаптивностью (Adaptability).

Традиционные операционные системы, такие как Linux или Windows, полагаются на огромные базы унаследованного кода на C/C++, где безопасность обеспечивается дисциплиной разработки и тестированием, а не математическим доказательством.

В предлагаемой архитектуре доверенная вычислительная база (Trusted Computing Base, TCB) математически верифицируется, а гибкость обеспечивается агентами ИИ, работающими в изолированных средах WebAssembly (WASM), способными генерировать, верифицировать и внедрять новый код без остановки системы.



## 1.1 Архитектурные столпы


- **1. Верифицированное ядро** (Verified Microkernel) : Основано на Rust и Verus. В
отличие от проектов типа Redox или Theseus, здесь ставится задача не просто использовать memory-safe язык, но и математически доказать отсутствие логических ошибок (deadlocks, race conditions, integer overflows) и корректность выполнения спецификаций.

Используется методология "framekernel", где верифицируется только критически важный код, управляющий unsafe операциями.

- **2. WebAssembly Component** Model : Все драйверы, системные сервисы и
пользовательские приложения исполняются в среде WASM.

Это обеспечивает аппаратную независимость и строгую изоляцию ("песочницу").

Компонентная модель (Component Model) позволяет динамически связывать модули, что критически важно для горячей замены кода при самообновлении.

- **3. AI-Native Orchestration** (AIOS) : Языковые модели (LLM) интегрированы в ядро
через интерфейс WASI-NN. Операционная система предоставляет "Шину Интеллекта" (Intelligence Bus), где агенты управляют планированием процессов, распределением памяти и оптимизацией ввода-вывода на основе семантического понимания задач пользователя, а не эвристик.

- **4. Цикл Автопоэзиса** (Self-Coding Loop) : Система включает в себя встроенный
компилятор (на базе Cranelift/Winch) и верификатор Verus как сервисы.

ИИ может сгенерировать улучшенную версию драйвера, автоматически отправить её на верификацию, и, в случае успеха доказательства безопасности, ядро произведет горячую замену компонента.



## 1.2 Структура плана работ

План разбит на 50 рабочих пакетов (Work Packages, WP). Каждый пакет оценивается в 80 человеко-часов (стандартный двухнедельный спринт для одного высококвалифицированного инженера).

Общая трудоемкость составляет 4000 человеко-часов, что соответствует примерно году работы команды из 2-3 ключевых разработчиков.

Каждый пункт содержит не просто задачу, но (что именно мы доказываем), необходимые исследования и ожидаемые артефакты.

## Conclusion
8. Заключение
Предложенный план описывает амбициозный, но технически реализуемый путь к созданию операционной системы, которая является не просто набором утилит, а живым, адаптирующимся организмом.

Сочетание математической строгости Verus с креативностью генеративного ИИ создает уникальную архитектуру, где надежность ядра гарантирует, что саморазвитие системы никогда не приведет к самоуничтожению.

Это следующий логический шаг в эволюции системной инженерии.

Ссылки на использованные материалы Интегрированы в текст отчета:.... Источники
1. verus-lang/verus: Verified Rust for low-level systems code - GitHub,
https://github.com/verus-lang/verus
2.
Verus:

A Practical Foundation for Systems Verification
-
Tej Chajed, https://www.chajed.io/papers/verus:sosp2024.pdf
3.
Asterinas:

A Linux ABI-Compatible, Rust-Based Framekernel OS with a Small and Sound TCB
-
arXiv, https://arxiv.org/html/2506.03876v1
4.
The WebAssembly Component Model:

Introduction, https://component-model.bytecodealliance.org/
5.
What is the WebAssembly Component Model?

-
F5, https://www.f5.com/company/blog/what-is-the-webassembly-component-model
6.
LLM as OS, Agents as Apps:

Envisioning AIOS, Agents and the AIOS-Agent Ecosystem
-
arXiv, https://arxiv.org/html/2312.03815v2
7.
agiresearch/AIOS:

AIOS:

AI Agent Operating System
-
GitHub, https://github.com/agiresearch/AIOS
8.
Building self-evolving AI systems: exploring the architecture | by Pavel Buchnev | Jan, 2026, https://butschster.medium.com/building-self-evolving-ai-systems-exploring-the-architecture-a63912fd72c4
9.
Hyperlight Wasm:

Fast, secure, and OS-free
-
Microsoft Open Source Blog, https://opensource.microsoft.com/blog/2025/03/26/hyperlight-wasm-fast-secure-and-os-free
10.
Theseus is a new OS written from scratch in Rust to experiment with novel OS structure, better state management, and how to shift OS responsibilities like resource management into the compiler.

-
Reddit, https://www.reddit.com/r/rust/comments/jpfuwe/theseus_is_a_new_os_written_from_scratch_in_rust/