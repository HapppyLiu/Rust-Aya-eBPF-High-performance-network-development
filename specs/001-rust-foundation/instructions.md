# Instructions
#创建第一个 Feature
使用 /speckit.specify

建立本学习工程的第一个学习 Feature：

# Rust Foundation

目标不是学习 Rust 全部语法，而是建立后续学习 Linux Networking、eBPF、Aya、XDP、AF_XDP、zero-copy 和 lock-free programming 所需要的系统级 Rust 能力。

学习者需要掌握以下核心能力：

1. Ownership
2. Move semantics
3. Borrowing
4. Lifetime
5. Struct / Enum
6. Trait
7. Generic
8. Error handling
9. Iterator
10. Closure
11. Smart pointer
12. Send / Sync
13. Concurrency
14. Atomic
15. Unsafe Rust
16. Raw pointer
17. Pointer arithmetic
18. Alignment
19. Aliasing
20. Memory safety
21. FFI
22. no_std
23. core / alloc / std
24. Panic and allocator fundamentals

重点学习 systems programming 所需的 Rust，而不是覆盖所有 Rust 语言特性。

学习过程必须包含：

* 概念学习
* 最小 Rust 实验
* 编译器行为观察
* 必要时分析 MIR / LLVM IR
* Rust 源码阅读
* Feynman 教学
* Acceptance Criteria
* 综合实验

特别关注 Rust 与 Linux/eBPF 的交界面。

学习完成后，学习者应该能够：

* 阅读中等复杂度 Rust systems code；
* 阅读 unsafe Rust；
* 理解 raw pointer；
* 理解 Send / Sync；
* 理解 Rust memory model 的核心约束；
* 理解 no_std；
* 理解 Rust 如何与 C/Linux API 交互；
* 为后续 Aya/eBPF 学习做好准备。

不要在 specification 中提前决定具体 Rust library、Aya API 或具体实现方案。

Specification 应关注：

* Why
* What
* Learning outcomes
* Acceptance criteria

而不是具体实现方法。


----------------------------------------
执行 /speckit.clarify 消除歧义


----------------------------------------------------------------------
执行 /speckit.plan 生成计划

以下是约束建议：

为 Rust Foundation 学习 Feature 制定实现计划。

Technical Context：

Language:
Rust

Edition:
使用当前稳定 Rust edition，以实际项目环境为准。

Target Platform:
Linux x86_64

Project Type:
Learning project + executable experiments

Testing:
cargo test

Additional validation:
rustc
cargo clippy
cargo fmt
MIR / LLVM IR（对于需要理解编译器行为的实验）

Performance:
本 Feature 不追求最终网络性能，但需要建立后续 systems programming 所需的性能分析基础。

Documentation:
Markdown + Rust source code + experiment results + Feynman tutorials

Project structure should separate：

* learning material
* executable experiments
* Feynman tutorials
* acceptance tests

重点设计：

1. Rust ownership / borrowing 实验；
2. lifetime 实验；
3. Send / Sync 实验；
4. unsafe Rust 实验；
5. raw pointer 实验；
6. alignment / aliasing 实验；
7. FFI 实验；
8. no_std 实验；
9. core / alloc / std 实验；
10. atomic / concurrency 实验。

每一个核心知识模块都应该能够被独立验证。

不要为了学习 Rust 而构建没有价值的大型应用。

实验应该尽可能小、独立、可运行、可观察。

---------------------------------------------------------------------------------
/speckit-checklist 

检查：Learning Quality
例如：□ 每个学习目标都有 Acceptance Criteria
□ 每个核心知识都有实验
□ unsafe 都有 Safety Invariant
□ no_std 有可运行实验
□ Send/Sync 有编译器验证
□ raw pointer 有 unsafe experiment
□ FFI 有 C/Rust boundary experiment
□ 每个阶段都有 Feynman question
□ 每个实验都有运行命令
□ 每个实验都有预期结果
□ 学习目标可以追溯到 Spec

----------------------------------------------------------------------------------
/speckit-tasks 
得到类似：
Phase 1 — Foundation

- [ ] T001 建立 Rust experiment workspace
- [ ] T002 建立 experiment execution convention
- [ ] T003 建立 acceptance-test convention


Phase 2 — Ownership

- [ ] T004 阅读 ownership 基础
- [ ] T005 创建 move experiment
- [ ] T006 创建 Copy experiment
- [ ] T007 创建 Clone experiment
- [ ] T008 分析编译器错误
- [ ] T009 编写 Feynman tutorial
- [ ] T010 完成 acceptance test


Phase 3 — Borrowing

- [ ] T011 immutable borrow experiment
- [ ] T012 mutable borrow experiment
- [ ] T013 multiple borrow experiment
- [ ] T014 NLL experiment
- [ ] T015 Feynman tutorial
- [ ] T016 acceptance test


Phase 4 — Unsafe Rust

- [ ] T017 raw pointer experiment
- [ ] T018 pointer arithmetic experiment
- [ ] T019 alignment experiment
- [ ] T020 aliasing experiment
- [ ] T021 UB experiment
- [ ] T022 safe abstraction experiment
- [ ] T023 Feynman tutorial
- [ ] T024 acceptance test


Phase 5 — no_std

- [ ] T025 core experiment
- [ ] T026 no_std experiment
- [ ] T027 panic experiment
- [ ] T028 allocator analysis
- [ ] T029 core/alloc/std dependency analysis
- [ ] T030 Feynman tutorial
- [ ] T031 acceptance test

Tasks 应从 spec.md、plan.md 等设计文档推导，并按照 user story / phase 组织、建立依赖关系和可独立验证的任务。

----------------------------------------------------------------------------------------------------------------

