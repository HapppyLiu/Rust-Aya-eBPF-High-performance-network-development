# Feature Specification: Rust Foundation

**Feature Branch**: `main`（本项目未为该 Feature 单独开分支）

**Created**: 2026-09-03

**Status**: Draft

**Input**: User description: "建立本学习工程的第一个学习 Feature：Rust Foundation。目标不是学习 Rust 全部语法，而是建立后续学习 Linux Networking、eBPF、Aya、XDP、AF_XDP、zero-copy 和 lock-free programming 所需要的系统级 Rust 能力。"

## Why

后续 Feature（Linux Networking、eBPF、Aya、XDP、TC、AF_XDP、zero-copy、lock-free）全部建立在
系统级 Rust 能力之上。若跳过本 Feature，学习者在读 eBPF packet parsing 代码时会把 raw pointer
边界检查误认为语法噪音，在读 Aya 用户态代码时无法判断 map 访问的并发安全性，在遇到 `#![no_std]`
时无法解释缺失的到底是哪一类运行时能力。Constitution XI（Incremental Complexity）禁止在前置知识
未验收的情况下直接进入复杂框架，因此本 Feature 是整个项目的第一个、也是唯一的无前置 Feature。

本 Feature 的价值不在于"学过 Rust"，而在于把 24 项系统级能力转化为可验证的、可被后续 Feature 依赖
的既有能力。

## Scope

**In scope**：支撑 systems programming 的 Rust 子集，共 24 项核心能力（见 Capability Coverage）。

**Out of scope**：Rust 语言特性的完整覆盖。明确排除 async/await 运行时、过程宏与声明宏的编写、
Web/服务端生态、GUI、cargo workspace 工程化技巧、以及任何 eBPF/Aya 程序的实际编写（属于 Feature
002 及之后）。

**本规格不做的决定**：不指定任何具体 Rust library、crate、Aya API、工具链版本或实现方案。此类选择
由 `/speckit-plan` 阶段决定。

## Capability Coverage

24 项核心能力与学习旅程（User Story）的映射，用于 Constitution XIII 要求的可追踪性：

| ID | Capability | Story |
|----|-----------|-------|
| C-01 | Ownership | US1 |
| C-02 | Move semantics | US1 |
| C-03 | Borrowing | US1 |
| C-04 | Lifetime | US1 |
| C-05 | Struct / Enum | US2 |
| C-06 | Trait | US2 |
| C-07 | Generic | US2 |
| C-08 | Error handling | US3 |
| C-09 | Iterator | US3 |
| C-10 | Closure | US3 |
| C-11 | Smart pointer | US3 |
| C-12 | Send / Sync | US4 |
| C-13 | Concurrency | US4 |
| C-14 | Atomic | US4 |
| C-15 | Unsafe Rust | US5 |
| C-16 | Raw pointer | US5 |
| C-17 | Pointer arithmetic | US5 |
| C-18 | Alignment | US5 |
| C-19 | Aliasing | US5 |
| C-20 | Memory safety | US5 |
| C-21 | FFI | US6 |
| C-22 | no_std | US7 |
| C-23 | core / alloc / std | US7 |
| C-24 | Panic and allocator fundamentals | US7 |

## Clarifications

### Session 2026-09-03

- Q: Feynman 五项检验应该按 24 项能力逐项做，还是按 8 个 Story 模块做？ → A: 混合——Feynman
  教学材料按 8 个 Story 模块产出；24 项能力各自保留独立的 Acceptance Criteria 与实验。
- Q: 当一段 unsafe 实验触发了未定义行为但程序表面正常运行时，凭什么判定实验实际失败？ → A: 以 UB
  检测工具的输出为判定依据，不得以"程序未崩溃"作为无 UB 的证据；具体工具由 Plan 阶段选定。
- Q: 实验重跑时"结果一致"指逐字节相同还是关键判定条件一致？ → A: 每个实验显式声明"稳定断言"，
  重跑仅比对稳定断言；地址、耗时、线程交错等标注为非断言输出，允许变化。
- Q: 本 Feature 期间是否锁定单一 Rust 工具链版本？ → A: 锁定单一 pinned toolchain 贯穿整个
  Feature，升级推迟至 Feature 完成后；具体版本号由 Plan 阶段指定。
- Q: SC-007 的 90% 正确率在多大的题目集合上统计？ → A: 学习开始前定稿一组 ≥10 个自定义类型的
  判定题，学完后一次性作答，最多错 1 道。

## User Scenarios & Testing *(mandatory)*

本 Feature 的"用户"是学习者本人。Story 按**学习顺序**编号（Constitution XI 要求递进），
优先级 tier 表示**阻塞性**而非顺序：

- **P1** = 后续 Feature 的硬前置，缺失将直接阻塞 eBPF/Aya 学习
- **P2** = 阅读与编写中等复杂度 systems code 所需，可在 P1 之后补齐
- **P3** = 整合与终验收

### User Story 1 - 所有权、移动与生命周期 (Priority: P1)

学习者面对一段陌生的 Rust systems code，能够在不运行程序的前提下，说出每个值的所有者是谁、
在哪一行发生移动或借用、每个引用的生命周期受哪个作用域约束，以及编译器为何接受或拒绝这段代码。

**Why this priority**: 所有权模型是 Rust 与 C 的根本差异，也是后续理解 unsafe 边界（US5）、
Send/Sync（US4）与 eBPF 数据结构生命周期的前提。缺少它，后续所有 Story 都无法验收。

**Independent Test**: 给定一段包含移动、可变借用与显式生命周期标注的代码，学习者独立完成所有权流转
标注并预测编译结果；实际编译后预测与编译器诊断一致，即视为通过。

**Acceptance Scenarios**:

1. **Given** 一段会触发 borrow checker 报错的代码，**When** 学习者在编译前预测错误类型与出错位置，
   **Then** 预测与编译器实际诊断一致，且学习者能解释该规则存在的原因而非仅复述错误信息。
2. **Given** 一个值在函数间传递的最小示例，**When** 学习者分别用移动、不可变借用、可变借用三种方式改写，
   **Then** 三个版本均能编译运行，且学习者能说明各自的语义差异与适用场景。
3. **Given** 一个需要显式生命周期标注才能编译的函数签名，**When** 学习者去掉标注，
   **Then** 学习者能解释省略规则为何在此失效，以及标注表达的约束含义。

---

### User Story 2 - 类型系统与抽象 (Priority: P2)

学习者能够读懂由 struct、enum、trait 与泛型组合而成的抽象层，判断某个调用在编译期解析到哪个具体
实现，并区分静态分发与动态分发在内存布局和调用开销上的差异。

**Why this priority**: 中等复杂度 Rust 代码（含后续 Aya 用户态代码）大量使用 trait 抽象；
读不懂抽象层就无法定位实际执行路径。但它不直接决定内存安全，因此列为 P2。

**Independent Test**: 给定一段使用泛型与 trait 对象混合的代码，学习者独立标注每个调用点的分发方式
与实际执行的实现；通过编译器行为观察验证标注正确。

**Acceptance Scenarios**:

1. **Given** 同一 trait 的泛型参数版本与 trait 对象版本，**When** 学习者对比两者，
   **Then** 能说明单态化与 vtable 的差异，并指出各自在二进制体积与调用开销上的取舍。
2. **Given** 一个用 enum 表达状态机的最小示例，**When** 学习者分析其内存布局，
   **Then** 能解释判别式与变体数据的空间占用，并说明为何该布局在无 GC 环境下可用。

---

### User Story 3 - 组合能力：错误处理、迭代器、闭包与智能指针 (Priority: P2)

学习者能够读懂并写出使用 Result 错误传播、迭代器链、闭包捕获与智能指针的惯用 systems code，
并能说明每种构造在运行时的实际代价。

**Why this priority**: 这是阅读真实 Rust 代码的日常门槛；其中闭包捕获与智能指针的所有权语义
直接衔接 US4 的并发共享模型。

**Independent Test**: 学习者把一段用显式错误码与手写循环写成的代码，改写为惯用的 Result 与迭代器
版本，两版行为一致，且学习者能说明改写前后运行时开销的变化依据。

**Acceptance Scenarios**:

1. **Given** 一条多阶段迭代器链，**When** 学习者预测其求值顺序与实际发生的分配次数，
   **Then** 预测与实验观测一致。
2. **Given** 三种捕获方式（按引用、按可变引用、按值）的闭包，**When** 学习者分析各自捕获的内容，
   **Then** 能解释捕获方式如何决定闭包能否跨线程移动，为 US4 的 Send/Sync 建立衔接。
3. **Given** 一组智能指针使用场景，**When** 学习者为每种场景选择合适的指针类型，
   **Then** 能说明选择依据是所有权语义而非习惯，并指出误用会导致的具体后果。

---

### User Story 4 - 并发、Send/Sync 与原子操作 (Priority: P2)

学习者能够判断一个类型为何是（或不是）Send 或 Sync，能读懂使用原子操作的代码并说明所选内存序的
约束含义，能识别数据竞争与其在 Rust 类型系统中被拦截的位置。

**Why this priority**: 直接支撑后续 lock-free programming 与 BPF map 并发访问的理解。列为 P2
是因为在 eBPF 程序侧的并发模型与用户态不同，本阶段建立基础即可，深化留给后续 Feature。

**Independent Test**: 给定学习开始前即已定稿的判定题集（≥10 个自定义类型），学习者独立判定其
Send/Sync 属性并说明推导依据；用编译实验验证判定结果。

**Acceptance Scenarios**:

1. **Given** 一个包含内部可变性的类型，**When** 学习者判断其能否跨线程共享，
   **Then** 判断正确，且能解释 Send 与 Sync 各自约束的是什么，而非把两者混为一谈。
2. **Given** 一段使用原子变量的最小并发程序，**When** 学习者尝试放宽内存序，
   **Then** 能说明放宽后哪些执行顺序变为可能，以及该程序是否仍然正确。
3. **Given** 一段存在数据竞争的伪代码，**When** 学习者尝试在安全 Rust 中实现它，
   **Then** 能定位编译器拒绝的具体规则，并说明绕过该规则需要承担的义务。

---

### User Story 5 - Unsafe Rust 与裸指针内存模型 (Priority: P1)

学习者能够阅读 unsafe 代码块，为其写出完整的 Safety Invariant，识别裸指针运算、对齐、别名与
provenance 相关的未定义行为风险，并解释安全抽象如何在 unsafe 之上重建不变量。

**Why this priority**: eBPF packet parsing 本质上是在受限环境中做带边界检查的裸指针运算。
这是本 Feature 中与后续工作耦合最紧的一项，也是 Constitution VI 的直接落点。

**Independent Test**: 给定一段包含 unsafe 的代码，学习者独立写出其 Safety Invariant 清单并指出
至少一处若违反将导致 UB 的调用约束；通过构造违反该约束的实验、并以 UB 检测工具的输出验证其判断。

**Acceptance Scenarios**:

1. **Given** 一段使用裸指针读写内存的代码，**When** 学习者审阅它，**Then** 能写出调用方必须满足的
   全部前置条件（有效性、对齐、别名、生命周期），且不以"Rust 要求 unsafe"作为解释。
2. **Given** 一个未对齐访问的构造实验，**When** 学习者运行它，**Then** 能解释为何该行为是 UB
   而不仅仅是"在某些架构上会崩溃"，并说明观测到的现象与 UB 定义之间的区别。
3. **Given** 一个由 unsafe 实现、对外暴露安全接口的最小抽象，**When** 学习者审查该接口，
   **Then** 能判断该抽象是否真正安全，并在其不安全时构造出触发问题的调用序列。
4. **Given** 一段做指针偏移与边界检查的解析代码，**When** 学习者移除边界检查，
   **Then** 能说明越界读取的后果，并将该模式与后续 eBPF verifier 的边界要求建立对应关系。

---

### User Story 6 - FFI 与 Rust/C/Linux 交界面 (Priority: P2)

学习者能够读写跨 Rust 与 C 的函数与数据结构声明，解释调用约定、内存布局、所有权移交与错误传递
在语言边界上如何被表达，并能说明违反约定时问题在何处显现。

**Why this priority**: Aya 的用户态部分依赖大量 Linux syscall 与 C 结构体交互；这是 Rust 与
Linux 之间最直接的接口层。列为 P2 是因为它建立在 US5 的裸指针理解之上。

**Independent Test**: 学习者独立完成一次 Rust 调用 C 函数与 C 调用 Rust 函数的双向最小实验，
两侧数据结构布局一致且能正确传递；学习者能说明布局一致性是靠什么保证的。

**Acceptance Scenarios**:

1. **Given** 一个 C 结构体定义，**When** 学习者在 Rust 中声明对应类型，**Then** 能说明为何默认
   布局不可依赖，以及需要何种约束才能保证两侧一致。
2. **Given** 一次跨边界的内存分配与释放，**When** 学习者分析所有权流向，**Then** 能指出哪一侧
   负责释放，并说明约定不一致时会产生何种故障。
3. **Given** 一个返回错误码的 Linux API 调用，**When** 学习者将其封装为 Rust 惯用错误类型，
   **Then** 封装保留了原始错误信息，且学习者能解释封装层引入的假设。

---

### User Story 7 - no_std、运行时边界与分配器 (Priority: P1)

学习者能够准确区分 core、alloc 与 std 各自提供的能力，解释 `#![no_std]` 环境下缺失的是哪一类
运行时服务，说明 panic 处理与内存分配在缺少操作系统支持时如何被重新定义。

**Why this priority**: eBPF 程序运行在无标准库、无分配器、无 panic unwinding 的受限环境中。
不理解这层边界，学习者在 Feature 003+ 中将无法解释编译失败的根因。Constitution VII 的直接落点。

**Independent Test**: 学习者独立构建一个 `no_std` 的最小可执行产物并使其成功编译，能逐项说明为使其
编译而必须显式提供的每一样东西及其原因。

**Acceptance Scenarios**:

1. **Given** 一段依赖 std 的代码，**When** 学习者为其加上 `#![no_std]`，**Then** 能对每一条编译
   错误说明缺失的具体能力属于 core、alloc 还是 OS services，而非笼统归因为"不能用标准库"。
2. **Given** 一个 `no_std` 环境，**When** 学习者需要使用堆分配的数据结构，**Then** 能说明需要
   引入什么、由谁提供分配能力，以及在 eBPF 这类环境中该前提是否成立。
3. **Given** panic 在 `no_std` 下必须被显式处理的要求，**When** 学习者实现最小 panic 处理，
   **Then** 能解释 panic 在有无操作系统支持时的行为差异及其对程序正确性的影响。

---

### User Story 8 - 综合实验与能力终验收 (Priority: P3)

学习者完成一个整合前七个 Story 的综合实验：在一个自定义的、面向字节缓冲区的最小解析场景中，
同时用到所有权设计、trait 抽象、错误处理、unsafe 裸指针访问与安全封装，并在受限（no_std 风格）
约束下组织代码。

**Why this priority**: 单项能力通过不等于能组合使用。本 Story 是 Constitution XII 闭环中的
Build 环节，也是进入 Feature 002 的准入检查。它必须在其他 Story 之后进行。

**Independent Test**: 学习者独立完成该综合实验，产物可重复构建与运行，并能对照 24 项能力清单逐项
指出其在实验中的体现位置。

**Acceptance Scenarios**:

1. **Given** 综合实验完成，**When** 对照 24 项能力清单逐项检查，**Then** 每一项都能在实验产物或
   配套说明中定位到具体体现，无遗漏项。
2. **Given** 实验中的全部 unsafe 代码，**When** 审查其安全性文档，**Then** 每个 unsafe 块都有
   对应的 Safety Invariant 说明。
3. **Given** 实验的完整环境记录，**When** 在记录的环境中重新执行构建与运行命令，
   **Then** 全部稳定断言复现，非断言输出的差异不影响判定。

### Edge Cases

- 某能力项的实验在不同 CPU 架构（如 x86_64 与 aarch64）上表现不同时，如何判定该能力是否已掌握？
  规格要求：能解释差异来源者视为掌握，仅记录现象者视为未完成。
- unsafe 实验触发了未定义行为但程序表面正常运行时，学习者如何得知实验实际失败？
  规格要求：以 UB 检测工具的输出为判定依据（FR-019），程序未崩溃 MUST NOT 被当作无 UB 的证据。
- 编译器版本升级导致中间表示输出或诊断信息与既有学习材料不一致时，既有验收记录是否仍然有效？
  规格要求：本 Feature 全程锁定单一工具链版本（FR-020），因此该情况不应在 Feature 进行期间出现；
  若因不可避免的原因升级，受影响实验的验收记录 MUST 重新验证。
- 学习者在 US5（unsafe）验收未通过的情况下要求直接进入 Feature 002（eBPF）时，应如何处理？
  规格要求：US1、US5、US7 是硬前置（FR-012），未通过即阻塞 Feature 002 启动，不接受"边学边补"。
- 某能力项在 Rust 源码中找不到清晰的对应实现（属于编译器内建而非库代码）时，Source-Code-First
  要求如何满足？规格要求：改为记录对应的语言参考或编译器实现位置（FR-005）。
- Feynman 五项检验中仅"常见误区"一项无法完成时，该模块判定为完成还是未完成？
  规格要求：判定为未完成，并生成补齐任务（FR-006）；五项检验不接受部分通过。

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: 本 Feature MUST 覆盖 Capability Coverage 表中全部 24 项能力（C-01 至 C-24），
  每项 MUST 归属到恰好一个 Story，且 MUST NOT 存在无归属的能力项。
- **FR-002**: 每个学习模块 MUST 包含以下全部环节：概念学习、最小 Rust 实验、编译器行为观察、
  Rust 源码阅读、Feynman 教学材料、Acceptance Criteria。缺任一环节的模块 MUST NOT 标记为完成。
- **FR-003**: 每项能力 MUST 至少对应一个可重复执行的最小实验，实验 MUST 记录执行命令、输入与实际
  输出，并 MUST 显式声明其**稳定断言**——即重跑时必须复现的判定条件。输出中因地址随机化、耗时、
  线程交错等原因天然可变的部分 MUST 被标注为非断言输出，MUST NOT 计入一致性判定。
- **FR-004**: 涉及内存布局、优化行为或抽象代价的能力项，MUST 通过编译器中间表示（MIR / LLVM IR）
  或等效的编译器可观测行为进行验证，SHOULD 仅在概念性解释不足以判定时才引入中间表示分析。
- **FR-005**: 每项能力 MUST 至少定位一处 Rust core / std / 编译器源码中的实际结构体、函数或
  调用路径，并记录其文件路径与符号名。若该能力属于编译器内建而无库代码对应，MUST 记录对应的语言
  参考或编译器实现位置以替代。
- **FR-006**: Feynman 五项检验（自述概念、最小示例、底层机制、常见误区、回答验证性问题）MUST 以
  学习模块为单位执行，即 8 个 Story 各产出一份覆盖其全部所属能力项的 Feynman 教学材料；模块通过
  五项检验方可标记完成，任一项未通过 MUST 记为未完成并生成补齐任务。能力项级别的验收由 FR-003 的
  实验与 FR-007 的 Acceptance Criteria 承担，MUST NOT 因模块级检验而省略。
- **FR-007**: 每项能力 MUST 具备可验证的 Acceptance Criteria；"看过""了解过""做过笔记"
  MUST NOT 被接受为完成标准。
- **FR-008**: 所有 unsafe 代码产出 MUST 附带 Safety Invariant 说明，覆盖有效性、对齐、别名、
  provenance 与生命周期中所有适用项；MUST NOT 以"Rust 要求 unsafe"作为说明。
- **FR-009**: 涉及 `no_std` 的产出 MUST 明确区分 core、alloc、std、allocator、panic、runtime
  与 OS services 各自的边界，MUST NOT 将 `#![no_std]` 表述为"不能使用标准库"。
- **FR-010**: 每个实验 MUST 附带环境记录，至少包含 Rust toolchain 版本、kernel 版本、CPU 架构
  与执行命令。
- **FR-011**: 学习顺序 MUST 遵循 US1 → US2 → US3 → US4 → US5 → US6 → US7 → US8 的递进关系；
  任何跳过 MUST 在 Plan 中显式记录理由与补齐计划。
- **FR-012**: US5（Unsafe）、US7（no_std）与 US1（所有权）MUST 在 Feature 002 启动前完成验收，
  三者构成后续 Feature 的硬前置。
- **FR-013**: 每个学习目标 MUST 可追踪到 Spec → Plan → Task → Learning Material → Experiment
  → Source Code → Acceptance Criteria 的完整链条；MUST NOT 产生无归属的孤立笔记。
- **FR-014**: 每项能力 MUST 说明其与后续 Linux / eBPF / Aya 学习的关联点，或显式标注为"仅为
  理解基础，不直接对应后续内容"。
- **FR-015**: US8 综合实验 MUST 整合前七个 Story 的能力，产出 MUST 可重复构建与运行。
- **FR-016**: 本规格 MUST NOT 指定具体 Rust library、crate、Aya API 或实现方案；此类决策
  MUST 推迟至 Plan 阶段。
- **FR-017**: 本 Feature MUST NOT 包含实际 eBPF 或 Aya 程序的编写；此类工作属于后续 Feature。
- **FR-018**: 当某能力项的实验结果依赖 CPU 架构或编译器版本时，产出 MUST 记录所用架构与版本，
  并说明该结果是否可跨环境推广。
- **FR-019**: 涉及未定义行为的实验 MUST 以 UB 检测工具的输出作为通过或失败的判定依据；
  "程序未崩溃"或"输出符合预期" MUST NOT 被作为不存在 UB 的证据。具体检测工具由 Plan 阶段选定。
- **FR-020**: 本 Feature 全程 MUST 锁定单一 Rust 工具链版本，工具链升级 MUST 推迟至本 Feature
  完成之后；若因不可避免的原因在进行期间升级，受影响实验的验收记录 MUST 重新验证后方可继续沿用。
  具体版本号由 Plan 阶段指定。

### Key Entities

- **Capability（能力项）**：24 项系统级 Rust 能力之一，具有唯一 ID（C-01 至 C-24）、所属 Story、
  以及完成状态。是本 Feature 的最小可验收单位。
- **Learning Module（学习模块）**：与 Story 一一对应的学习单元，共 8 个，包含概念说明、实验、
  源码引用、Feynman 材料与 Acceptance Criteria 五类内容。Feynman 检验以此为单位执行。
- **Experiment（实验）**：可重复执行的最小验证单元，包含执行命令、预期输出、实际输出、环境记录，
  以及一组**稳定断言**（重跑必须复现的判定条件）与被标注为非断言的可变输出。
- **Source Reference（源码引用）**：指向 Rust core / std / 编译器源码中具体位置的记录，
  包含文件路径与符号名。
- **Feynman Material（Feynman 教学材料）**：面向"讲给他人听"的解释性产出，覆盖五项检验。
- **Acceptance Criterion（验收标准）**：针对某能力项的可验证完成条件，判定结果为通过或未通过。
- **Environment Record（环境记录）**：实验的可复现上下文，含 toolchain 版本、kernel 版本、
  架构与依赖信息。

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 8 个 Story 模块 100% 具备已通过 Feynman 五项检验的教学材料，且每份材料覆盖其所含
  能力项的全部条目（24 项能力在模块材料中的覆盖率为 100%）。
- **SC-002**: 24 项能力中 100% 至少对应一个实验，且这些实验在记录的环境中重新执行时，
  **稳定断言**全部复现的比例达到 100%（非断言输出的差异不计入）。
- **SC-003**: 24 项能力中 100% 至少关联一处带文件路径与符号名的源码引用（编译器内建项按 FR-005
  的替代方式记录）。
- **SC-004**: 给定一段 200 至 400 行、此前未读过的中等复杂度 Rust systems code，学习者能在
  60 分钟内说明其所有权流转、抽象分发方式与错误传播路径，且经复核关键判断无误。
- **SC-005**: 给定一段包含 unsafe 的陌生代码，学习者能在 30 分钟内写出其 Safety Invariant 清单，
  并至少识别出一处若调用方违反约定即导致未定义行为的位置。
- **SC-006**: 学习者能独立完成一个 `no_std` 最小产物的构建，并对构建过程中出现的每一条错误
  逐条说明缺失能力的归属层（core / alloc / OS services），说明正确率 100%。
- **SC-007**: 面对一组在学习开始前即已定稿的、不少于 10 个自定义类型的判定题，学习者一次性作答的
  Send / Sync 判定正确率不低于 90%（即最多错 1 道），且能对每次判定给出推导依据而非结论。
  题目 MUST 在学习开始前定稿，MUST NOT 在验收时另行挑选。
- **SC-008**: 学习者能完成一次双向跨语言边界调用实验，两侧数据结构布局一致且数据传递正确。
- **SC-009**: US8 综合实验完成，且 24 项能力全部能在实验产物中定位到具体体现位置，无遗漏项。
- **SC-010**: 所有 unsafe 代码产出中，附带 Safety Invariant 说明的比例为 100%。
- **SC-011**: 所有学习目标均可追溯到完整的 Spec → … → Acceptance Criteria 链条，孤立笔记数量为 0。

## Assumptions

- 学习者已具备至少一门系统级或通用编程语言的经验，不需要从"什么是变量"开始。
- 学习者拥有可用的 Linux 环境与本机编译能力；实验默认在 x86_64 上执行，架构相关差异（如对齐、
  内存序表现）以 x86_64 为基准记录，其他架构作为对照说明而非必需实验。
- 每项能力的学习深度以"能支撑 systems programming 与后续 eBPF/Aya 学习"为界，不追求语言律师级别的
  完备性；深度边界的具体判定在 Plan 阶段结合 Acceptance Criteria 细化。
- 本 Feature 无固定日历工期，完成与否由 Acceptance Criteria 判定而非时间投入（Constitution V）。
- 本项目为单人学习工程，"评审"由学习者对照可验证产物自评完成，产出物的可验证性是评审有效性的前提。
- async/await、宏编写、Web 生态等不在本 Feature 范围内；若后续 Feature 需要，另行立项。
- 具体工具链版本号、UB 检测工具、实验组织形式与所用 crate 由 Plan 阶段确定，本规格只约束"必须锁定
  单一版本"与"必须使用 UB 检测工具"，不预设具体选择。
- 本规格沿用仓库中已存在的 `specs/001-rust-foundation/` 目录，而非按顺序编号新建 `003-`；
  该目录已由使用者预先创建并按本 Feature 命名。
