---
description: "Task list for Rust Foundation (001-rust-foundation)"
---

# Tasks: Rust Foundation

**Input**: Design documents from `/specs/001-rust-foundation/`

**Prerequisites**: [plan.md](./plan.md)、[spec.md](./spec.md)、[research.md](./research.md)、
[data-model.md](./data-model.md)、[contracts/](./contracts/)、[quickstart.md](./quickstart.md)、
[checklists/learning-quality.md](./checklists/learning-quality.md)

**Tests**: 本 Feature 的 `tests/` **不是**可选的开发者测试，而是 FR-003 规定的**验收载体**
（稳定断言只允许出现在 `#[test]` 中，见 R-05 / experiment-contract §C2）。因此每个能力的
`tests/cNN_*.rs` 是强制产物，与 `examples/cNN_*.rs`（可观察、NON-ASSERTION）成对存在。

**Organization**: 任务按 **Story = 学习模块（m1…m8）** 组织，每个 Story 一个 Phase，可独立验证。

## Format: `[ID] [P?] [Story] Description`

- **[P]**: 可并行（不同文件、无未完成依赖）
- **[Story]**: 所属 User Story（US1…US8）；Setup / Foundational / Polish 阶段无 Story 标签
- 每个任务都包含确切文件路径

## Path Conventions

本 Feature 采用 plan.md §Project Structure 的**四分目录 + 单 workspace**：
`learning/`（概念与源码引用）、`experiments/`（可执行实验）、`feynman/`（教学材料）、
`acceptance/`（验收标准），加上 `harness/`（rf-harness 验证设施）与 `tools/`（脚本）。
`experiments/m7-nostd` 被根 workspace `exclude`（R-03）。

外加 **`learner/`（Learner Track 学习者视图）**，见下方 §双轨产物。

## 🔀 双轨产物（Dual-Track Learning Artifact）

每个产出学习内容的 Task 同时产出**两个视图**（契约：learning-artifact-contract §H）：

| 轨道 | 目录 | 内容 | 何时读 |
|-----|------|------|-------|
| **Learner Track** | `learner/mN-*/` | 学习框架、引导问题、预测表、自检清单、提示阶梯 | **先读**，用于自测 |
| **Answer Track** | `learning/mN-*/`、`feynman/mN-*.md`、`experiments/mN-*/` 源码 | 完整答案：原理、源码、实验 | **卡住后**再读 |
| **汇合点** | `acceptance/criteria/cNN.md` | 两轨共用的客观验收 | 任何时候 |

**关键约束**：`learner/` 下 MUST NOT 出现错误码、具体数值答案、Miri UB 类别文本、源码行号、
机制性结论、Answer Track 原文（§H3.1 的 L1–L6 六类）。
本项目为单人工程，"不泄漏"由**提交顺序纪律**保证：Learner Track 三件套 MUST 在对应模块的
实验与 concept 材料投入使用**之前**先行提交（§H3.4）。

双轨任务为 **T151–T161**，见下方 §双轨产物任务。基础设施类 Task（工具链、harness、脚本）
不产出学习内容，不适用双轨。

## ⚠️ 学习顺序即执行顺序（与常规 Feature 的关键差异）

spec.md 明确："Story 按**学习顺序**编号（Constitution XI 要求递进），优先级 tier 表示**阻塞性**
而非顺序"。因此：

- Phase 顺序 = **US1 → US2 → US3 → US4 → US5 → US6 → US7 → US8**（FR-011 强制，禁止跳过）；
- **Story 之间 MUST NOT 并行**——m2 依赖 m1，…，m8 依赖 m1–m7（data-model §2 `prerequisite`）；
- 并行机会只存在于**单个 Story 内部**（不同能力的实验文件互不相交）；
- P1（US1/US5/US7）表示"Feature 002 的硬前置"（FR-012），不表示可以先做。

---

## Phase 1: Setup（共享基础设施 + 门禁前置裁定）

**Purpose**: 消除 learning-quality 清单的未决项（该清单声明为**正式门禁**：未通过项 MUST 在
`/speckit-tasks` 前修订文档或**登记为显式补齐任务**——T001–T004 即该登记），并建立工具链、
workspace 与脚本骨架。

### 1A. 门禁未决项裁定（阻塞后续所有验收判定）

- [X] T001 [P] 修订 `specs/001-rust-foundation/contracts/experiment-contract.md`：补齐 UB 判定的可判定规则——(a) §C5 增加"Miri 报告 UB 但**类别与事前预测不一致**"时判 fail 并重写预测的规则（CHK027）；(b) §C5.3 给出**允许的错误类别子串白名单**（如 `Undefined Behavior` / `memory access failed` / `attempting a read access` / `not sufficiently aligned` / `alias`），禁止事后任选子串（CHK026）；(c) §C3.2"解释"字段增加内容下限（MUST 回答"为什么会这样"与"这**不能**证明什么"两问，仅复述现象即未完成，CHK017）；(d) §C7.2 明确"可推广性判定"对**全部**实验强制，敏感性由实验作者在 OBSERVATIONS 中给出判定与理由（CHK019）；(e) §C4 增加"同一样本触发多个错误码"时的断言方式（断言集合为子集关系，MUST 列出全部实际错误码）（CHK020）
- [X] T002 [P] 修订 `specs/001-rust-foundation/contracts/learning-artifact-contract.md`：(a) §C1 明确"模块 Feynman fail → 能力级 AC **可先判 pass**，但 Capability 状态被模块门禁挡在 `experiment-passed`，MUST NOT 进入 `accepted`"（CHK050）；(b) §D2 为不产生可执行产物的目标（§G 限时阅读、§F 推导依据）给出等效客观判据（复核清单逐条勾选 + 复核人签名日期，CHK003）；(c) §C 检验结果表为五项各写一行可判定合格标准，第 5 节明确"≥5 个问题且每题指向一条实验断言或源码引用"（CHK048）；(d) §F5 与 spec SC-007 的通过线统一表述并声明冲突时以哪条为准（CHK034）；(e) §E 表头增加 **Task** 列（值为本文件的 T-ID），使 FR-013 七段链条不缺环（CHK051）；(f) §E3 给出孤立笔记的**可执行枚举方式**（`learning/`+`feynman/` 下全部条目与矩阵 C-ID 比对的具体命令，CHK052/CHK053）
- [X] T003 修订 `specs/001-rust-foundation/spec.md`：(a) US7 Independent Test 与 SC-006 明确 `no_std` 的"可运行"= **构建成功**（`x86_64-unknown-none` 无 OS，产物不可直接执行，CHK036）；(b) SC-006 的错误集合定义为**固定可枚举清单**（三步递进各自的错误逐条列出，避免编译器只报首错导致分母不确定，CHK038）；(c) SC-002 判定口径补上"`cargo test --workspace` 全绿 **+** `experiments/m7-nostd` 构建成功 + m7 的编译期/静态检查断言复现"（CHK040/CHK041）；(d) FR-011 与 FR-012 的关系明确化：US2/US3/US4/US6 未通过时能否启动 Feature 002 给出唯一答案（CHK058）
- [X] T004 修订 `specs/001-rust-foundation/spec.md` 与 `plan.md`：(a) 为 US5 AS4"与 eBPF verifier 边界要求建立对应关系"给出在 FR-017 约束下**可产出可验收**的形式（限定为 `learning/m5-unsafe/concept.md` 的"与后续学习的关联"条目 + 边界检查移除对照实验的解释，MUST NOT 编写 eBPF 程序，CHK031）；(b) 将"ASan 覆盖面窄于 Miri，`ASan 无报告` 不等价于 `无 UB`"显式登记为 Assumption，并说明 C-21 的判定强度弱于 US5（CHK045）；(c) 在 plan.md Capability Gate Matrix 的 C-19 行补充"Stacked Borrows 与 Tree Borrows 结论不一致时的判定规则"（CHK030）

### 1B. 工具链与工程骨架

- [X] T005 创建 `rust-toolchain.toml`：锁定 stable `1.98.0`，`components = ["rustfmt","clippy","rust-src"]`，`targets = ["x86_64-unknown-linux-gnu","x86_64-unknown-none"]`（R-01 / FR-020）
- [X] T006 创建根 `Cargo.toml`：`[workspace] members = ["harness","experiments/m*"]`、`exclude = ["experiments/m7-nostd"]`、`resolver`、`[workspace.package] edition = "2024"`、`[workspace.lints.clippy] undocumented_unsafe_blocks = "deny"` 与 `multiple_unsafe_ops_per_block = "deny"`（experiment-contract §C6.4）
- [X] T007 [P] 创建 `rustfmt.toml` 与 `clippy.toml`（宽度/msrv 等最小配置，保证 `cargo fmt --check` 与 `cargo clippy -- -D warnings` 可作为日常门禁，quickstart §1）
- [X] T008 [P] 创建四分目录骨架与 `.gitignore`：`learning/`、`experiments/`、`feynman/`、`acceptance/criteria/`、`tools/`、`harness/`，各目录放置说明其职责的 `README.md`（R-09）
- [X] T009 [P] 创建 `tools/env-record.sh`：输出 data-model §9 全部字段（`rustc_stable`/`rustc_nightly`/`edition`/`kernel`/`arch`/`target`/`command`），格式与 `rf_harness::env` 一致（FR-010）
- [X] T010 [P] 创建 `tools/emit-mir.sh` 与 `tools/emit-llvm-ir.sh`：封装 `cargo rustc -- --emit=mir|llvm-ir`，输出落到 `target/ir/`（R-04 阶梯 3–4）
- [X] T011 [P] 创建 `tools/run-miri.sh` 与 `tools/run-asan.sh`：统一 `MIRIFLAGS`（含 `-Zmiri-many-seeds`、`-Zmiri-tree-borrows` 开关）与 ASan 的 `-Zbuild-std` 参数（R-02 / quickstart §4 §5）
- [X] T012 执行 `quickstart.md` §0 的环境基线校验，并把 `tools/env-record.sh` 的输出存档为 `acceptance/environment-baseline.md`（后续所有 OBSERVATIONS 的环境块以此为基准，FR-010 / FR-018）

**Checkpoint**: 门禁未决项已裁定，工具链锁定，脚本可用。

---

## Phase 2: Foundational（阻塞所有 Story 的前置）

**Purpose**: 建立 `rf-harness` 验证设施、追踪链单一事实源、四类产物模板，以及必须在 US4 之前冻结的
Send/Sync 判定题集。

**⚠️ CRITICAL**: 本阶段完成前，任何 Story 的实验都无法产出**可机械检查**的断言。

- [ ] T013 创建 `harness/Cargo.toml` 与 `harness/src/lib.rs`（package `rf-harness`，**零外部依赖**，声明 `pub mod compile_fail; pub mod counting_alloc; pub mod miri; pub mod env;`，harness-api.md §非目标：MUST NOT 放入任何学习目标代码）
- [ ] T014 [P] 实现 `harness/src/compile_fail.rs`：`expect_errors`、`try_compile`、`CompileOutcome::{has_code,codes}`；以 pinned stable `rustc --edition 2024 --emit=metadata` 编译，完整 stderr 落盘 `target/compile-fail/<stem>.stderr`；失败信息 MUST 同时打印**期望**与**实际**错误码（harness-api §compile_fail / R-06）
- [ ] T015 [P] 实现 `harness/src/counting_alloc.rs`：`CountingAllocator`（`GlobalAlloc` 包装 `System`，`AtomicUsize` 计数）、`AllocStats{allocs,deallocs,reallocs,bytes_allocated,peak_bytes}`、`measure`；字节数仅统计**请求值**，不含分配器内部开销（harness-api §counting_alloc / R-07）
- [ ] T016 [P] 实现 `harness/src/miri.rs`：`run_example`、`MiriOutcome::{reported_ub,stderr_contains,skipped}`；**关键契约**——`skipped()` 为真时 `reported_ub()` MUST **panic**，禁止"没跑工具"被当成"没有 UB"；支持 `RF_SKIP_MIRI=1`（harness-api §miri / FR-019）
- [ ] T017 [P] 实现 `harness/src/env.rs`：`record()` 与 `EnvironmentRecord::to_markdown()`，输出格式 MUST 与 `tools/env-record.sh` 一致（harness-api §env / FR-010）
- [ ] T018 编写 `harness/tests/harness_selfcheck.rs`：验证 harness 自身行为——`expect_errors` 对故意报错样本成功、对可编译样本 panic；`measure` 对已知分配次数的闭包返回确定值；`MiriOutcome::skipped()` 下 `reported_ub()` 确实 panic（设施本身不可靠则全部验收失效）
- [ ] T019 创建 `acceptance/capability-matrix.md`：24 行（C-01…C-24），列为 `C-ID | Capability | Module | Story | Task | Experiment | SourceRef | Criterion | UB Tool | Status`，数据取自 plan.md Capability Gate Matrix，初始 `Status = planned`（FR-013 / learning-artifact §E，含 T002 新增的 Task 列）
- [ ] T020 [P] 创建 `acceptance/criteria/_template.md`：按 learning-artifact §D 的四小节（验证命令 / 通过判据 / 结果 / 环境记录链接），内含 §D1 禁用措辞清单与 §D2 退出码要求
- [ ] T021 [P] 创建 `learning/_templates/concept.md` 与 `learning/_templates/source-refs.md`：按 learning-artifact §A 与 §B 的 REQUIRED 小节与表头（含 `kind = library | reference-fallback` 与"这段源码回答了什么"列）
- [ ] T022 [P] 创建 `feynman/_template.md`：按 learning-artifact §C 的五个 REQUIRED 小节 + 检验结果表（五项合取，FR-006）
- [ ] T023 [P] 创建 `experiments/_templates/OBSERVATIONS.md`：顶部环境块占位 + NON-ASSERTION 记录块（命令 / 输出 / 解释 / 架构相关性），解释字段含 T001 定义的内容下限（experiment-contract §C3.2 §C7）
- [ ] T024 定稿并提交 `acceptance/send-sync-quiz.md`（≥10 个**自定义类型**，每题给出完整定义，要求判定 Send/Sync 并写推导依据）与 `acceptance/send-sync-quiz.answers.md`（作答前 MUST NOT 打开）——**MUST 在 US4 学习开始前完成**，冻结时点以版本控制提交时间为证（SC-007 / R-10 / learning-artifact §F1）
- [ ] T025 运行基线一键验证：`cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`（此时仅含 harness）全绿，作为后续"全量一致性判定"的起点（quickstart §1 / experiment-contract §C8.1）

**Checkpoint**: 验证设施可用、追踪链就位、题集已冻结 —— US1 可以开始。

---

## Phase 3: User Story 1 - 所有权、移动与生命周期（Priority: P1，Feature 002 硬前置）🎯 MVP

**Goal**: 面对陌生 Rust systems code，能在不运行程序的前提下说出每个值的所有者、移动/借用发生的行、
每个引用的生命周期约束，以及编译器为何接受或拒绝该代码。覆盖 C-01…C-04。

**Independent Test**: `cargo test -p m1-ownership` 全绿；且对 `compile_fail/` 样本能在运行断言
**之前**正确预测错误码与出错位置（quickstart §3 的先预测后验证循环）。

- [X] T026 [US1] 创建 `experiments/m1-ownership/Cargo.toml` 与 `src/lib.rs`：dev-dependency 指向 `rf-harness`；`src/lib.rs` 预先声明 `pub mod c01; pub mod c02; pub mod c03; pub mod c04;` 并创建四个空占位文件，使后续能力实验互不冲突（[P] 的前提）
- [X] T027 [US1] 编写 `learning/m1-ownership/concept.md`：按 learning-artifact §A，为 C-01…C-04 各写"一句话定义 / 底层机制 / ≥1 条常见误解 / 对应实验"，并在"与后续学习的关联"为每项写明与 Linux/eBPF/Aya 的关联点或显式标注"仅为理解基础"（FR-014）
- [X] T028 [P] [US1] 编写 `learning/m1-ownership/source-refs.md`：C-01 `core/src/ops/drop.rs` `Drop`、C-02 `core/src/mem/mod.rs` `replace`/`take`/`forget`、C-03 `core/src/cell.rs` `RefCell`/`BorrowFlag` + borrowck 记 `reference-fallback`、C-04 `core/src/marker.rs` `PhantomData` + elision 记 `reference-fallback`；逐条记录路径、符号、**实际行号**与"这段源码回答了什么"（FR-005 / §B2 已知 fallback 项）
- [X] T029 [P] [US1] C-01 实验：`experiments/m1-ownership/examples/c01_ownership.rs`（观察 drop 顺序与作用域结束时机，输出为 NON-ASSERTION）+ `tests/c01_ownership.rs`（稳定断言：用计数器验证 drop 次数与顺序，每个 `#[test]` 带 `CLAIM` 注释）
- [X] T030 [P] [US1] C-02 实验：`experiments/m1-ownership/examples/c02_move.rs`（move / `Copy` / `Clone` 三种改写的语义差异，US1 AS2）+ `tests/c02_move.rs`（断言 `mem::replace`/`take` 后的值、`forget` 后 drop 未发生）
- [X] T031 [P] [US1] C-03 实验：`experiments/m1-ownership/examples/c03_borrow.rs`（不可变 / 可变 / 多重借用与 NLL 缩短借用范围的现象）+ `compile_fail/c03_two_mut_borrows.rs`（首行 `//! EXPECT: E0499` + `//! CLAIM:`）+ `tests/c03_borrow.rs`（`rf_harness::compile_fail::expect_errors` 断言错误码；`RefCell` 的运行期 borrow panic 断言）
- [X] T032 [P] [US1] C-04 实验：`experiments/m1-ownership/examples/c04_lifetime.rs`（去掉显式标注后 elision 为何失效，US1 AS3）+ `compile_fail/c04_dangling_ref.rs`（`//! EXPECT: E0597`）+ `tests/c04_lifetime.rs`（错误码断言 + `PhantomData` 的 `size_of == 0` 断言）
- [X] T033 [US1] 用 `tools/emit-mir.sh` 导出 C-01/C-02 的 MIR，定位 **drop 插入点与移动语义**在 MIR 中的表现，结果作为 NON-ASSERTION 记入 `experiments/m1-ownership/OBSERVATIONS.md`（FR-004 阶梯 3 / experiment-contract §C3.3）
- [X] T034 [US1] 填写 `experiments/m1-ownership/OBSERVATIONS.md`：`tools/env-record.sh` 环境块 + 四个 example 的实际输出 + 完整 stderr 抄录 + 每条"解释"（回答"为什么"与"这不能证明什么"）+ 架构相关性判定
- [X] T035 [US1] 编写 `acceptance/criteria/c01.md`、`c02.md`、`c03.md`、`c04.md`：各含可直接复制的验证命令、≥1 条由**命令退出码**决定的判据（§D2）、源码引用已记录判据；MUST NOT 使用禁用措辞（§D1）
- [X] T036 [US1] 编写 `feynman/m1-ownership.md`：五个 REQUIRED 小节全部完成，`Capabilities covered` 精确等于 C-01…C-04；第 3、4 节每条论断显式引用本模块实验断言名或源码符号（§C2）；填写检验结果表
- [X] T037 [US1] 模块验收：`cargo test -p m1-ownership` 全绿 → 更新 `acceptance/capability-matrix.md` 中 C-01…C-04 的 `Status` 与 `Task` 列，`feynman_status = passed`，并在矩阵中标记 m1 为 **FR-012 硬前置已满足**

**Checkpoint**: m1 完成 —— MVP 达成，US2 可以开始。

---

## Phase 4: User Story 2 - 类型系统与抽象（Priority: P2）

**Goal**: 读懂 struct/enum/trait/泛型组合的抽象层，判断调用解析到哪个实现，区分静态与动态分发在
内存布局与调用开销上的差异。覆盖 C-05…C-07。

**Independent Test**: `cargo test -p m2-types` 全绿；能对泛型版本与 trait 对象版本各自标注分发方式，
并由 `size_of` 断言与 IR 观察验证标注正确。

- [ ] T038 [US2] 创建 `experiments/m2-types/Cargo.toml` 与 `src/lib.rs`（声明 `pub mod c05; pub mod c06; pub mod c07;` 占位）
- [ ] T039 [US2] 编写 `learning/m2-types/concept.md`（C-05…C-07 四要素齐备 + FR-014 关联点）
- [ ] T040 [P] [US2] 编写 `learning/m2-types/source-refs.md`：C-05 `core/src/option.rs`（niche）+ `core/src/mem/mod.rs` `size_of`；C-06 `core/src/fmt/mod.rs` `Display` + `core/src/ops/deref.rs` `Deref`；C-07 `core/src/iter/traits/iterator.rs` + `core/src/cmp.rs` `PartialOrd`
- [ ] T041 [P] [US2] C-05 实验：`experiments/m2-types/examples/c05_layout.rs`（enum 状态机的判别式与变体空间占用，US2 AS2）+ `tests/c05_layout.rs`（`size_of`/`align_of`/`offset_of` 断言 + `size_of::<Option<&u8>>() == size_of::<&u8>()` 的 niche 断言）
- [ ] T042 [P] [US2] C-06 实验：`experiments/m2-types/examples/c06_trait.rs`（同一 trait 的静态分发与 trait 对象两版对照）+ `tests/c06_trait.rs`（`size_of::<&dyn Trait>() == 2 * size_of::<usize>()` 等确定性断言）
- [ ] T043 [P] [US2] C-07 实验：`experiments/m2-types/examples/c07_generic.rs`（单态化的可观察后果）+ `compile_fail/c07_missing_bound.rs`（`//! EXPECT: E0277`）+ `tests/c07_generic.rs`（错误码断言 + 泛型与 trait 对象行为等价性断言）
- [ ] T044 [US2] 用 `tools/emit-llvm-ir.sh` 对照静态分发（直接调用）与动态分发（vtable 间接调用）的 IR 结构，并用 nightly `-Z print-mono-items` 记录单态化实例数量；全部作为 NON-ASSERTION 记入 OBSERVATIONS（FR-004 阶梯 4–5 / R-07）
- [ ] T045 [US2] 填写 `experiments/m2-types/OBSERVATIONS.md`（环境块 + 输出 + 解释 + 架构相关性；IR 文本一律 NON-ASSERTION）
- [ ] T046 [US2] 编写 `acceptance/criteria/c05.md`、`c06.md`、`c07.md`（含退出码判据）
- [ ] T047 [US2] 编写 `feynman/m2-types.md`（五项检验；`Capabilities covered` = C-05…C-07）
- [ ] T048 [US2] 模块验收：`cargo test -p m2-types` 全绿 → 更新 capability-matrix 的 C-05…C-07 状态与 Task 列

**Checkpoint**: m2 完成 —— US3 可以开始。

---

## Phase 5: User Story 3 - 组合能力：错误处理、迭代器、闭包与智能指针（Priority: P2）

**Goal**: 读写惯用的 Result 传播、迭代器链、闭包捕获与智能指针代码，并用**确定性量**（分配次数）
说明其运行时代价。覆盖 C-08…C-11。

**Independent Test**: `cargo test -p m3-composition` 全绿；对多阶段迭代器链的求值顺序与**分配次数**
的预测与 `CountingAllocator` 实测一致（US3 AS1）。

- [ ] T049 [US3] 创建 `experiments/m3-composition/Cargo.toml` 与 `src/lib.rs`：声明 `pub mod c08..c11` 占位，并在 `tests/` 侧启用 `#[global_allocator] static A: CountingAllocator`（harness-api §counting_alloc）
- [ ] T050 [US3] 编写 `learning/m3-composition/concept.md`（C-08…C-11 四要素 + FR-014 关联点；C-10 捕获方式与 US4 Send/Sync 的衔接 MUST 写明，US3 AS2）
- [ ] T051 [P] [US3] 编写 `learning/m3-composition/source-refs.md`：C-08 `core/src/result.rs` + `core/src/convert/mod.rs` `From` + `std/src/error.rs`；C-09 `core/src/iter/traits/iterator.rs` + `core/src/iter/adapters/map.rs`；C-10 `core/src/ops/function.rs` `Fn`/`FnMut`/`FnOnce`；C-11 `alloc/src/boxed.rs`、`alloc/src/rc.rs`、`alloc/src/sync.rs` `Arc`
- [ ] T052 [P] [US3] C-08 实验：`experiments/m3-composition/examples/c08_error.rs`（显式错误码写法 → `Result` + `?` 改写，行为一致）+ `compile_fail/c08_missing_from.rs`（`//! EXPECT: E0277`，`?` 缺少 `From` 转换）+ `tests/c08_error.rs`（错误传播路径断言 + 错误码断言）
- [ ] T053 [P] [US3] C-09 实验：`experiments/m3-composition/examples/c09_iterator.rs`（惰性求值顺序的打印观察）+ `tests/c09_iterator.rs`（用 `rf_harness::counting_alloc::measure` 断言 `collect` 与 `sum` 链路的**确定性分配次数**，US3 AS1）
- [ ] T054 [P] [US3] C-10 实验：`experiments/m3-composition/examples/c10_closure.rs`（按引用 / 按可变引用 / 按值三种捕获）+ `compile_fail/c10_borrow_escapes_thread.rs`（`//! EXPECT: E0373` 或 `E0277`）+ `tests/c10_closure.rs`（`Fn`/`FnMut`/`FnOnce` 分类断言 + 捕获决定能否跨线程移动的错误码断言）
- [ ] T055 [P] [US3] C-11 实验：`experiments/m3-composition/examples/c11_smart_ptr.rs`（`Box`/`Rc`/`Arc` 的所有权语义与误用后果）+ `compile_fail/c11_rc_across_threads.rs`（`//! EXPECT: E0277`）+ `tests/c11_smart_ptr.rs`（`Rc::strong_count` 断言 + 分配次数断言 + 错误码断言）
- [ ] T056 [US3] 串行化分配计数：把使用 `measure` 的断言集中到单个 `#[test]` 或用 crate 内互斥量串行化，并在 `experiments/m3-composition/OBSERVATIONS.md` 中说明 `CountingAllocator` 对**整个 crate 全局生效**（harness-api §并发约束）
- [ ] T057 [US3] 填写 `experiments/m3-composition/OBSERVATIONS.md`（环境块 + 输出 + 解释 + 架构相关性）
- [ ] T058 [US3] 编写 `acceptance/criteria/c08.md`、`c09.md`、`c10.md`、`c11.md`（含退出码判据）
- [ ] T059 [US3] 编写 `feynman/m3-composition.md`（五项检验；`Capabilities covered` = C-08…C-11）
- [ ] T060 [US3] 模块验收：`cargo test -p m3-composition` 全绿 → 更新 capability-matrix 的 C-08…C-11 状态与 Task 列

**Checkpoint**: m3 完成 —— US4 可以开始（T024 的题集必须已冻结）。

---

## Phase 6: User Story 4 - 并发、Send/Sync 与原子操作（Priority: P2）

**Goal**: 判断类型为何是（或不是）Send/Sync，读懂原子操作代码并说明内存序约束，识别数据竞争被
类型系统拦截的位置。覆盖 C-12…C-14。

**Independent Test**: `cargo test -p m4-concurrency` 全绿 +
`MIRIFLAGS="-Zmiri-many-seeds" cargo +nightly miri test -p m4-concurrency` 无意外 UB；
且 T024 冻结题集的一次性作答错 ≤1 题并每题有推导依据（SC-007）。

- [ ] T061 [US4] 创建 `experiments/m4-concurrency/Cargo.toml` 与 `src/lib.rs`（声明 `pub mod c12; pub mod c13; pub mod c14;` 与 quiz 用的自定义类型模块 `pub mod quiz_types;` 占位）
- [ ] T062 [US4] 编写 `learning/m4-concurrency/concept.md`（C-12…C-14 四要素；MUST 分别陈述 Send 与 Sync **各自约束什么**而非混为一谈，US4 AS1；FR-014 写明与 BPF map 并发访问、lock-free 的关联）
- [ ] T063 [P] [US4] 编写 `learning/m4-concurrency/source-refs.md`：C-12 `core/src/marker.rs`（`unsafe auto trait Send`/`Sync` 与负向 impl，含实际行号）；C-13 `std/src/thread/mod.rs`、`std/src/sync/mutex.rs`；C-14 `core/src/sync/atomic.rs`（`Ordering`、`AtomicUsize`）
- [ ] T064 [P] [US4] C-12 实验：`experiments/m4-concurrency/examples/c12_send_sync.rs`（含内部可变性的类型能否跨线程共享）+ `compile_fail/c12_cell_across_threads.rs`（`//! EXPECT: E0277`）+ `tests/c12_send_sync.rs`（`fn assert_send<T: Send>()` / `assert_sync<T: Sync>()` 正向断言 + 错误码负向断言）
- [ ] T065 [P] [US4] C-13 实验：`experiments/m4-concurrency/examples/c13_concurrency.rs`（`thread::scope` / `Mutex` 的线程交错观察，输出标 NON-ASSERTION）+ `compile_fail/c13_data_race.rs`（在安全 Rust 中实现数据竞争被拒绝的具体规则，US4 AS3）+ `tests/c13_concurrency.rs`（与交错顺序**无关**的不变量断言，如累加总和；MUST NOT 断言完成顺序）
- [ ] T066 [P] [US4] C-14 实验：`experiments/m4-concurrency/examples/c14_atomic.rs`（放宽内存序后哪些执行顺序变为可能，US4 AS2）+ `tests/c14_atomic.rs`（`SeqCst` 下的确定性不变量断言 + `Ordering` 语义的可断言事实；unsafe 块如有 MUST 带五要素 SAFETY）
- [ ] T067 [US4] 编写 `experiments/m4-concurrency/tests/c12_send_sync_quiz.rs`：对 `acceptance/send-sync-quiz.md` 中每个自定义类型做正向 `assert_send`/`assert_sync` 断言，并为应当**不**满足的类型在 `compile_fail/quiz_*.rs` 建立 `E0277` 负向条目——编译器是最终裁判（learning-artifact §F4）
- [ ] T068 [US4] 一次性作答 `acceptance/send-sync-quiz.md` 并把作答与推导写入 `acceptance/send-sync-quiz.result.md`（作答**完成前** MUST NOT 打开 answers 文件），随后与 T067 的编译器判定对照，记录正确率（通过线见 T002 统一后的表述，SC-007）
- [ ] T069 [US4] 运行 `MIRIFLAGS="-Zmiri-many-seeds" cargo +nightly miri test -p m4-concurrency`（经 `tools/run-miri.sh`），把 `ub_verdict` 与所用 seed 数记入 OBSERVATIONS；未运行时 MUST 记 `n/a` 而非 `clean`（FR-019 / experiment-contract §C5.2）
- [ ] T070 [US4] 填写 `experiments/m4-concurrency/OBSERVATIONS.md`（环境块 + 线程交错输出标注为 NON-ASSERTION + 解释 + 内存序表现的**架构相关性**判定：x86_64 强序对放宽内存序的掩盖作用，FR-018）
- [ ] T071 [US4] 编写 `acceptance/criteria/c12.md`、`c13.md`、`c14.md`（C-12 判据 MUST 包含 quiz 的客观校验命令）
- [ ] T072 [US4] 编写 `feynman/m4-concurrency.md`（五项检验；MUST 承载 quiz 每题的推导依据，SC-007 要求"给出推导依据而非结论"）
- [ ] T073 [US4] 模块验收：`cargo test -p m4-concurrency` 全绿 + Miri many-seeds 通过 → 更新 capability-matrix 的 C-12…C-14 状态与 Task 列

**Checkpoint**: m4 完成 —— US5 可以开始。

---

## Phase 7: User Story 5 - Unsafe Rust 与裸指针内存模型（Priority: P1，Feature 002 硬前置）

**Goal**: 读 unsafe 代码并写出完整 Safety Invariant，识别裸指针运算/对齐/别名/provenance 的 UB 风险，
解释安全抽象如何在 unsafe 之上重建不变量。覆盖 C-15…C-20。**本 Feature 与后续工作耦合最紧的模块。**

**Independent Test**: `cargo test -p m5-unsafe` 全绿 + `cargo +nightly miri test -p m5-unsafe`
（Stacked 与 Tree Borrows 两轮）结果与每个实验事前声明的 `ub_verdict` **预期值**一致；
每个 unsafe 块的 SAFETY 注释覆盖五要素。

**说明**：每项能力做**成对实验**——"安全侧"（正确用法 + 安全抽象，`ub_verdict = clean`）与
"UB 对照侧"（故意违反约束，`ub_verdict = expected-ub`）。这是 CHK029 要求的双侧覆盖。

- [ ] T074 [US5] 创建 `experiments/m5-unsafe/Cargo.toml` 与 `src/lib.rs`（声明 `pub mod c15..c20` 占位；crate 级启用 `undocumented_unsafe_blocks` 与 `multiple_unsafe_ops_per_block` 为 deny）
- [ ] T075 [US5] 编写 `learning/m5-unsafe/concept.md`（C-15…C-20 四要素；MUST 说明"UB 不等于崩溃"；FR-014 关联点按 T004 的裁定形式写明 eBPF packet parsing 的边界检查对应关系，US5 AS4）
- [ ] T076 [P] [US5] 编写 `learning/m5-unsafe/source-refs.md`：C-15 `core/src/slice/mod.rs` `get_unchecked` + UB 定义记 `reference-fallback`；C-16 `core/src/ptr/mod.rs`、`core/src/ptr/const_ptr.rs` `read`/`write`；C-17 `const_ptr.rs` `add`/`offset`/`wrapping_add`；C-18 `core/src/mem/mod.rs` `align_of` + `core/src/ptr/mod.rs` `read_unaligned`；C-19 `core/src/cell.rs` `UnsafeCell`；C-20 `core/src/slice/raw.rs` `from_raw_parts` + `alloc/src/vec/mod.rs` `set_len`
- [ ] T077 [P] [US5] C-15 安全侧：`examples/c15_unsafe.rs` + `tests/c15_unsafe.rs`（`get_unchecked` 在已校验边界内的正确用法，unsafe 块带五要素 SAFETY，Miri `clean`）
- [ ] T078 [P] [US5] C-15 UB 对照：`examples/c15_unsafe_ub.rs`（`get_unchecked` 越界）+ `tests/c15_unsafe_ub.rs`（`rf_harness::miri::run_example` → `reported_ub()` 为真 + 类别子串断言，`ub_verdict = expected-ub`）
- [ ] T079 [P] [US5] C-16 安全侧：`examples/c16_raw_ptr.rs` + `tests/c16_raw_ptr.rs`（`addr_of!`、`ptr::read`/`write` 的正确用法；断言可用"地址的关系性质"而非具体数值，§C2.2 例外）
- [ ] T080 [P] [US5] C-16 UB 对照：`examples/c16_raw_ptr_ub.rs`（悬垂指针读取 / use-after-free）+ `tests/c16_raw_ptr_ub.rs`（Miri 类别断言，`expected-ub`）
- [ ] T081 [P] [US5] C-17 安全侧：`examples/c17_ptr_arith.rs` + `tests/c17_ptr_arith.rs`（`add`/`offset` 在分配内的偏移 + `wrapping_add` 语义对照）
- [ ] T082 [P] [US5] C-17 UB 对照：`examples/c17_ptr_arith_ub.rs`（偏移越出分配 / provenance 越界）+ `tests/c17_ptr_arith_ub.rs`（Miri 类别断言，`expected-ub`）
- [ ] T083 [P] [US5] C-18 安全侧：`examples/c18_alignment.rs` + `tests/c18_alignment.rs`（`align_of` 断言 + `read_unaligned` 的正确用法 + `(p as usize) % align_of::<T>() == 0` 关系断言）
- [ ] T084 [P] [US5] C-18 UB 对照：`examples/c18_alignment_ub.rs` + `tests/c18_alignment_ub.rs`——**本 Feature 的核心教学对照**：普通运行正常退出并打印"合理"结果，同一源码在 Miri 下判定 UB；断言只匹配类别文本（`Undefined Behavior`、`memory access failed`），`alloc` 编号/偏移/行号为 NON-ASSERTION（quickstart §4 / research R-02 / US5 AS2）
- [ ] T085 [P] [US5] C-19 安全侧：`examples/c19_aliasing.rs` + `tests/c19_aliasing.rs`（`UnsafeCell` 的合法内部可变性，`clean`）
- [ ] T086 [P] [US5] C-19 UB 对照 + 双别名模型：`examples/c19_aliasing_ub.rs`（重叠可变引用）+ `tests/c19_aliasing_ub.rs`（默认 Stacked Borrows 与 `MIRIFLAGS="-Zmiri-tree-borrows"` 两轮判定，结论不一致时按 T004 写入 plan 的规则处理）
- [ ] T087 [P] [US5] C-20 安全侧：`examples/c20_mem_safety.rs` + `tests/c20_mem_safety.rs`（由 unsafe 实现、对外暴露**安全接口**的最小抽象，`slice::from_raw_parts` 正确用法，Miri `clean`，US5 AS3）
- [ ] T088 [P] [US5] C-20 UB 对照：`examples/c20_mem_safety_ub.rs`（`Vec::set_len` 暴露未初始化内存 / 越界 slice；并构造"该抽象不安全时的调用序列"，US5 AS3 后半）+ `tests/c20_mem_safety_ub.rs`（Miri 类别断言）
- [ ] T089 [US5] SAFETY 五要素审查：`cargo clippy -p m5-unsafe --all-targets -- -D warnings` 零告警（机械兜底）+ 逐块人工核查有效性/对齐/别名/provenance/生命周期**实质**覆盖，不适用项写明原因；结果记入 `acceptance/safety-invariant-audit.md`（FR-008 / SC-010 / §C6）
- [ ] T090 [US5] 边界检查移除对照：在 `examples/c20_bounds_check.rs` 中做"带边界检查的字节解析 → 移除检查"对照，说明越界读取后果，并按 T004 的裁定形式把该模式与 eBPF verifier 的边界要求写入 `learning/m5-unsafe/concept.md` 的关联小节（US5 AS4，MUST NOT 编写 eBPF 程序，FR-017）
- [ ] T091 [US5] 全模块 UB 扫描：`cargo +nightly miri test -p m5-unsafe` 与 `MIRIFLAGS="-Zmiri-tree-borrows" cargo +nightly miri test -p m5-unsafe` 两轮，逐实验记录 `ub_verdict` 实际值并与事前**预期值**比对（不一致按 T001 的规则判 fail 并重写预测）
- [ ] T092 [US5] 填写 `experiments/m5-unsafe/OBSERVATIONS.md`（环境块 + 12 个 example 输出 + 每条解释 MUST 包含"这**不能**证明什么" + 架构相关性：x86_64 容忍未对齐访问、aarch64 可能 SIGBUS，不可跨架构推广）
- [ ] T093 [US5] 编写 `acceptance/criteria/c15.md` … `c20.md`：每条 MUST 含 `cargo test` 与 `cargo +nightly miri test` 两条命令、`ub_verdict` 预期值、以及"每个 unsafe 块 SAFETY 覆盖五要素"判据
- [ ] T094 [US5] 编写 `feynman/m5-unsafe.md`（五项检验；第 5 节的验证性问题 MUST 包含"去掉这个边界检查，Miri 会报告哪一类 UB"这类可暴露理解缺口的问题）
- [ ] T095 [US5] 模块验收：`cargo test -p m5-unsafe` 全绿 + 两轮 Miri 判定与预期一致 + SAFETY 审计通过 → 更新 capability-matrix 的 C-15…C-20 状态，并标记 m5 为 **FR-012 硬前置已满足**

**Checkpoint**: m5 完成 —— US6 可以开始。

---

## Phase 8: User Story 6 - FFI 与 Rust/C/Linux 交界面（Priority: P2）

**Goal**: 读写跨 Rust/C 的函数与数据结构声明，解释调用约定、内存布局、所有权移交与错误传递如何在
语言边界上表达。覆盖 C-21。

**Independent Test**: `cargo test -p m6-ffi` 全绿（双向调用成功 + 两侧布局断言一致）+
`tools/run-asan.sh m6-ffi` 无报告（SC-008）。

- [ ] T096 [US6] 创建 `experiments/m6-ffi/Cargo.toml`：**唯一允许外部依赖的 crate** —— `libc`（dependency）与 `cc`（build-dependency），并在 `src/lib.rs` 声明 `pub mod c21;`（R-08，作用域严格限定于本 crate）
- [ ] T097 [US6] 编写 `experiments/m6-ffi/c/roundtrip.c` 与 `build.rs`：C 侧提供被 Rust 调用的函数、并调用由 Rust 导出的 `#[unsafe(no_mangle)] extern "C"` 函数；`build.rs` 用 `cc` 编译并链接
- [ ] T098 [US6] 编写 `learning/m6-ffi/concept.md`（C-21 四要素；MUST 回答"为何默认布局不可依赖"与"需要何种约束才能保证两侧一致"，US6 AS1；FR-014 写明与 Aya 用户态 syscall 交互的关联）
- [ ] T099 [P] [US6] 编写 `learning/m6-ffi/source-refs.md`：`core/src/ffi/mod.rs`（`c_int`/`c_char`）、`std/src/ffi/c_str.rs`（`CStr`），含行号与"这段源码回答了什么"
- [ ] T100 [P] [US6] C-21 布局一致性实验：`examples/c21_ffi_layout.rs` + `tests/c21_ffi_layout.rs`——Rust 侧 `#[repr(C)]` 结构体与 C 侧同名结构体的 `size_of`/`align_of`/`offset_of` **双侧**断言（C 侧数值由导出函数返回后比对），MUST NOT 以"声明了 `repr(C)`"即认定一致（CHK043）
- [ ] T101 [P] [US6] C-21 双向调用实验：`examples/c21_ffi.rs` + `tests/c21_ffi.rs`——Rust→C 与 C→Rust 两个方向**各自**的断言与通过判据分别成立（CHK042）；每个 unsafe 块带五要素 SAFETY
- [ ] T102 [US6] 跨边界所有权实验：`examples/c21_ffi_ownership.rs` + `tests/c21_ffi_ownership.rs`（一次跨边界分配与释放，断言"哪一侧负责释放"的约定被遵守）；并把"约定不一致会产生何种故障"写入 `learning/m6-ffi/concept.md` 与 OBSERVATIONS 作为可复核产物（US6 AS2 / CHK044）
- [ ] T103 [US6] Linux 错误码封装实验：`examples/c21_errno.rs` + `tests/c21_errno.rs`——用 `libc::open`/`close` 触发失败，把 `errno` 封装为 Rust 惯用错误类型，断言原始错误信息**未丢失**，并在 concept.md 说明封装层引入的假设（US6 AS3；这也是 Constitution VIII 的最小内核接触点）
- [ ] T104 [US6] 运行 `tools/run-asan.sh m6-ffi` 取得 `ub_verdict`，并把 T004 登记的假设"ASan 覆盖面窄于 Miri，无报告不等价于无 UB"抄录到 `experiments/m6-ffi/OBSERVATIONS.md` 的判定说明中（R-02 / CHK045）
- [ ] T105 [US6] 填写 `experiments/m6-ffi/OBSERVATIONS.md`（环境块 MUST 额外含 `libc`/`cc` 的精确版本与 C 编译器版本，data-model §9 `crates`；输出 + 解释 + 架构相关性）
- [ ] T106 [US6] 编写 `acceptance/criteria/c21.md`（验证命令含 `cargo test -p m6-ffi` 与 `tools/run-asan.sh m6-ffi`；判据含双向调用、双侧布局断言、释放责任说明）
- [ ] T107 [US6] 编写 `feynman/m6-ffi.md`（五项检验；`Capabilities covered` = C-21）
- [ ] T108 [US6] 模块验收：`cargo test -p m6-ffi` 全绿 + ASan 无报告 → 更新 capability-matrix 的 C-21 状态与 Task 列

**Checkpoint**: m6 完成 —— US7 可以开始。

---

## Phase 9: User Story 7 - no_std、运行时边界与分配器（Priority: P1，Feature 002 硬前置）

**Goal**: 区分 core/alloc/std 各自提供的能力，解释 `#![no_std]` 下缺失的是哪一类运行时服务，说明
panic 与内存分配在无 OS 支持时如何被重新定义。覆盖 C-22…C-24。

**Independent Test**: `cd experiments/m7-nostd && cargo build` 成功（"可运行"= **构建成功**，见 T003
的裁定）；且对三步递进过程中的**每一条**编译错误都能正确归属到 core / alloc / OS services（SC-006）。

- [ ] T109 [US7] 创建 `experiments/m7-nostd/`（**独立于根 workspace**，由 T006 的 `exclude` 保证）：`Cargo.toml`（`panic = "abort"`）、`.cargo/config.toml`（`target = "x86_64-unknown-none"`）、`src/main.rs`（`#![no_std]` `#![no_main]` + `#[panic_handler]`）（R-03）
- [ ] T110 [US7] 编写 `learning/m7-nostd/concept.md`（C-22…C-24 四要素；MUST 逐一定义 core / alloc / std / allocator / panic / runtime / OS services **七类边界**，使"归属哪一层"无二义（CHK039）；MUST NOT 把 `#![no_std]` 表述为"不能使用标准库"（FR-009）；FR-014 写明与 eBPF 受限环境的对应）
- [ ] T111 [P] [US7] 编写 `learning/m7-nostd/source-refs.md`：C-22 `core/src/lib.rs`（`#![no_std]`）+ `std/src/lib.rs`；C-23 `alloc/src/lib.rs` + `std/src/lib.rs` 的 re-export 关系；C-24 `core/src/panicking.rs`、`core/src/alloc/global.rs` `GlobalAlloc`、`alloc/src/alloc.rs`
- [ ] T112 [P] [US7] C-22 实验：`experiments/m7-nostd/src/c22_nostd.rs` + 构建脚本入口，断言载体为**编译期**（`const` 断言 / `compile_error!` 对照 / 构建退出码）与静态检查，按 T003 认定为合法稳定断言（CHK041）
- [ ] T113 [P] [US7] C-23 实验：`experiments/m7-nostd/src/c23_core_alloc_std.rs`——同一段逻辑分别只用 `core`、加 `alloc`、加 `std` 三个版本，记录各自在裸机 target 上的构建结果与失败归属（US7 AS1）
- [ ] T114 [P] [US7] C-24 实验：`experiments/m7-nostd/src/c24_panic_alloc.rs`——最小 `#[panic_handler]` 实现 + 自己实现的 `#[global_allocator]`（静态数组 bump allocator），使 `extern crate alloc` 的 `Vec` 可用；说明"分配能力由谁提供"以及该前提在 eBPF 环境是否成立（US7 AS2/AS3）
- [ ] T115 [US7] 三步递进错误归属实验：按 quickstart §6 依次（1）移除 `#[panic_handler]`（2）引入一个 std 类型（3）在无 allocator 时使用 `Vec`，每步构建并把**每一条**错误逐条归属到 core / alloc / OS services，记入 `experiments/m7-nostd/OBSERVATIONS.md`；错误清单按 T003 定稿为固定可枚举集合（SC-006 / CHK037/CHK038）
- [ ] T116 [US7] 产物静态检查：`nm target/x86_64-unknown-none/debug/m7-nostd` 与 `readelf -h`，断言/记录关键符号与节区（无 `__libc_start_main`、无 `eh_personality` 等），把可判定项写成脚本 `tools/check-nostd-artifact.sh` 以获得退出码判据（R-04 阶梯 6 / §D2）
- [ ] T117 [US7] host 侧分配器校验：在 `harness` 或 m3 的 host target 上用 `cargo +nightly miri test` 校验 C-24 自实现 `GlobalAlloc` 的 unsafe 实现（裸机产物无法跑 Miri，故在 host 侧取得 `ub_verdict`），结果记入 OBSERVATIONS（plan Gate Matrix C-24 的 "Miri（host 侧 allocator）"）
- [ ] T118 [US7] 填写 `experiments/m7-nostd/OBSERVATIONS.md`（环境块 MUST 含 `target = x86_64-unknown-none` + 三步递进的完整错误抄录 + 每条归属解释 + 架构相关性）
- [ ] T119 [US7] 编写 `acceptance/criteria/c22.md`、`c23.md`、`c24.md`（验证命令为 `cd experiments/m7-nostd && cargo build` 与 `tools/check-nostd-artifact.sh`；判据含"每条错误的归属正确率 100%"与"'不能用标准库'式笼统归因判为未通过"）
- [ ] T120 [US7] 编写 `feynman/m7-nostd.md`（五项检验；第 4 节 MUST 收录"`no_std` = 不能用标准库"这一误区及其证据）
- [ ] T121 [US7] 模块验收：`cd experiments/m7-nostd && cargo build` 成功 + `tools/check-nostd-artifact.sh` 退出码 0 + 三步归属正确率 100% → 更新 `acceptance/capability-matrix.md` 中 C-22…C-24 的状态与 Task 列，并标记 m7 为 **FR-012 硬前置已满足**

**Checkpoint**: m1/m5/m7 三个硬前置全部完成 —— US8 可以开始。

---

## Phase 10: User Story 8 - 综合实验与能力终验收（Priority: P3）

**Goal**: 在一个面向字节缓冲区的最小解析场景中同时用到所有权设计、trait 抽象、错误处理、unsafe 裸指针
访问与安全封装，并在 `no_std` 风格约束下组织代码；24 项能力逐项可定位。

**Independent Test**: `cargo test -p m8-capstone` 全绿 + `cargo +nightly miri test -p m8-capstone`
判定 `clean` + 24 项能力在产物或配套说明中**逐项**定位无遗漏（SC-009）。

- [ ] T122 [US8] 创建 `experiments/m8-capstone/Cargo.toml` 与 `src/lib.rs`：`#![no_std]` + `extern crate alloc`（host target 上可 `cargo test`，`std` 仅在 `#[cfg(test)]` 下启用），声明各分层模块
- [ ] T123 [US8] 编写 `learning/m8-capstone/concept.md`：综合场景设计（字节缓冲区报文解析器）+ 该场景为何能同时承载七个模块的能力 + 每一分层对应哪些 C-ID
- [ ] T124 [P] [US8] 编写 `learning/m8-capstone/source-refs.md`：综合实验中借鉴/对照的 core/alloc 源码位置（如 `core/src/slice/iter.rs`、`alloc/src/vec/mod.rs`）
- [ ] T125 [US8] 实现所有权与生命周期分层 `src/buffer.rs`：零拷贝的 `&'a [u8]` 视图类型与生命周期标注，体现 C-01…C-04
- [ ] T126 [US8] 实现 trait 抽象与泛型分层 `src/parse.rs`：`trait Parse` + 泛型解析组合器 + 一处 trait 对象用法，体现 C-05…C-07
- [ ] T127 [US8] 实现错误处理/迭代器/智能指针分层 `src/error.rs` 与 `src/iter.rs`：`Result` 错误传播、`Iterator` 实现、`alloc::boxed::Box` 的所有权语义，体现 C-08…C-11
- [ ] T128 [US8] 实现并发/原子分层 `src/stats.rs`：`AtomicUsize` 解析计数器 + 显式的 `Send`/`Sync` 论证（注释中写明为何成立），体现 C-12…C-14
- [ ] T129 [US8] 实现 unsafe 解析核心与安全封装 `src/raw.rs`：带边界检查的裸指针偏移读取 + 对外**安全**接口；每个 unsafe 块的 SAFETY 覆盖五要素，体现 C-15…C-20（FR-008 / SC-010）
- [ ] T130 [US8] 实现 `#[repr(C)]` 报文头 `src/wire.rs`：布局断言 + C-21 在综合实验中的体现（若不做真实 C 调用，则在配套说明中定位并说明理由，SC-009 允许"产物**或配套说明**"）
- [ ] T131 [US8] 在 `no_std` + `alloc` 约束下组织代码并验证：`cargo build -p m8-capstone --no-default-features`（不启用 std）成功；说明哪些能力因缺少 OS services 而必须换实现，体现 C-22…C-24
- [ ] T132 [US8] 编写 `experiments/m8-capstone/tests/capstone.rs`（必要时按分层拆为 `tests/parse.rs`、`tests/raw.rs`、`tests/wire.rs`）稳定断言集：解析正确性、错误路径、布局、分配次数（`CountingAllocator`）、`Send`/`Sync` 静态断言；每个 `#[test]` 带 `CLAIM`（FR-015）
- [ ] T133 [US8] 运行 `cargo +nightly miri test -p m8-capstone` 取得 `ub_verdict = clean`——综合实验的安全抽象必须**真的**安全（quickstart §7）
- [ ] T134 [US8] 编写 `acceptance/capability-location-map.md`：24 项能力逐项定位到综合实验的**文件 + 函数**（粒度按 T001/CHK010 的裁定），无遗漏项（SC-009）
- [ ] T135 [US8] 填写 `experiments/m8-capstone/OBSERVATIONS.md`（环境块 + 输出 + 解释 + 架构相关性）
- [ ] T136 [US8] 编写 `feynman/m8-capstone.md`（五项检验；MUST 解释"单项能力通过 ≠ 能组合使用"这一模块存在的理由）
- [ ] T137 [US8] 模块验收：`cargo test -p m8-capstone` 全绿 + Miri `clean` + 24 项定位无遗漏 → 更新 capability-matrix，m8 `status = complete`

**Checkpoint**: 八个模块全部完成 —— 进入终验收。

---

## Phase 11: Polish & Cross-Cutting Concerns（Feature 终验收）

**Purpose**: 完成不属于单一模块的 Success Criteria（SC-004/SC-005/SC-010/SC-011）、全量一致性
重跑，以及 Feature 002 的准入判定。

- [ ] T138 编写 `acceptance/unfamiliar-code-reading.md` 的**素材选定**部分：为 SC-004（200–400 行中等复杂度 Rust systems code）与 SC-005（含 unsafe 的陌生代码）各选定素材并记录来源与选取日期；素材 MUST NOT 来自本 Feature 自己的产出（learning-artifact §G1）
- [ ] T139 执行 SC-004 限时评估：60 分钟内产出所有权流转 / 抽象分发方式 / 错误传播路径三份说明，按 T002 定义的复核清单逐条勾选并签署日期，结果写入 `acceptance/unfamiliar-code-reading.md`
- [ ] T140 执行 SC-005 限时评估：30 分钟内写出 Safety Invariant 清单 + ≥1 处"调用方违反即 UB"的位置，并用 Miri 或书面推导验证所指位置确实构成 UB 风险，结果写入 `acceptance/unfamiliar-code-reading.md`
- [ ] T141 [P] 全仓 unsafe SAFETY 覆盖核查（SC-010）：`cargo clippy --workspace --all-targets -- -D warnings` 零 `undocumented_unsafe_blocks` 告警 + 按 T001 界定的范围（含 `learning/`、`feynman/` 文档中的示例代码，CHK021）人工核查五要素，更新 `acceptance/safety-invariant-audit.md` 至覆盖率 100%
- [ ] T142 [P] 孤立笔记审计（SC-011）：按 T002 §E3 定义的枚举方式列出 `learning/` 与 `feynman/` 下全部条目并与 `acceptance/capability-matrix.md` 比对，把结果与"孤立笔记数量 = 0"的结论写入 `acceptance/traceability-audit.md`
- [ ] T143 `acceptance/capability-matrix.md` 终态核对：24 行与 spec.md Capability Coverage 表逐行一致（无遗漏、无重复、无无归属，FR-001）；`Task` / `Experiment` / `SourceRef` / `Criterion` 四列全部非空（SC-003）；`Status` 全部为 `accepted`
- [ ] T144 全量一致性重跑（SC-002）：`cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`、`cd experiments/m7-nostd && cargo build`、`cargo +nightly miri test -p m5-unsafe -p m8-capstone`；全部稳定断言复现，非断言输出差异不计入（FR-003）
- [ ] T145 按 `quickstart.md` §8 的 Definition of Done 表逐条核对 SC-001…SC-011，把逐条判定结果写入 `acceptance/definition-of-done.md`
- [ ] T146 [P] 环境记录归档核查（FR-010 / FR-018）：8 个 `OBSERVATIONS.md` 顶部环境块齐备且字段完整（stable + nightly 双版本、kernel、arch、target、命令），架构敏感实验均有可推广性判定行
- [ ] T147 [P] 更新根 `README.md`：指向四分目录（`learning/` / `experiments/` / `feynman/` / `acceptance/`）与 `specs/001-rust-foundation/quickstart.md`，说明一键验证命令与"不要执行 `rustup update`"的约束
- [ ] T148 由学习者（本清单的 reviewer）复核并标记 `specs/001-rust-foundation/checklists/learning-quality.md` 的 60 项——T001–T004 已修订的条目方可标 `[x]`；本任务只负责标记与记录理由，MUST NOT 反向修改实现（清单声明为 reviewer-owned）
- [ ] T149 编写 `acceptance/feature-002-readiness.md`：声明 m1 / m5 / m7 三个模块 `status = complete`（FR-012 硬前置），并按 T003 对 FR-011/FR-012 关系的裁定给出 Feature 002 可否启动的结论
- [ ] T150 回写机制核查：确认 data-model §1 的 `accepted → regressed` 迁移与 Constitution Review gate 的"未通过项 MUST 作为新任务回写 tasks.md"有可执行入口——在本文件末尾建立"补齐任务（Remediation）"小节并说明写入规则

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup（Phase 1）**：无依赖，可立即开始。其中 T001–T004（门禁裁定）**阻塞**所有验收判定类任务，
  因为它们定义了"通过/失败"的判定规则。
- **Foundational（Phase 2）**：依赖 Phase 1 完成 —— **阻塞所有 Story**。
- **User Stories（Phase 3–10）**：**严格串行** US1 → US2 → US3 → US4 → US5 → US6 → US7 → US8
  （FR-011 / data-model §2 `prerequisite`）。这与常规 Feature 的"Story 可并行"**不同**：
  跳过或并行会违反 Constitution XI，任何跳过 MUST 在 plan.md 显式记录理由与补齐计划。
- **Polish（Phase 11）**：依赖全部 Story 完成。例外：T138（素材选定）可提前，但 T139/T140 的
  评估 MUST 在相应能力学完之后执行。

### User Story Dependencies

| Story | 模块 | 依赖 | 说明 |
|-------|------|------|------|
| US1 (P1) | m1 | Foundational | 无 Story 依赖；MVP |
| US2 (P2) | m2 | m1 | 抽象层的所有权语义依赖 C-01…C-04 |
| US3 (P2) | m3 | m2 | 迭代器/闭包/智能指针依赖 trait 与泛型 |
| US4 (P2) | m4 | m3 + **T024（题集冻结）** | Send/Sync 依赖闭包捕获与智能指针的所有权语义 |
| US5 (P1) | m5 | m4 | Safety Invariant 依赖所有权与并发共享模型 |
| US6 (P2) | m6 | m5 | FFI 建立在裸指针理解之上（spec US6 Why） |
| US7 (P1) | m7 | m6 | 运行时边界依赖 allocator 与 FFI 的 ABI 认知 |
| US8 (P3) | m8 | m1–m7 **全部通过** | Constitution XII 的 Build 环节 |

### Within Each User Story

0. **`learner/mN-*/` 三件套（Learner Track）—— MUST 先于下列全部步骤提交**（§H3.4）→
1. crate 骨架（预声明各能力子模块，使能力实验互不冲突）→
2. `concept.md`（概念，可与 `source-refs.md` 并行）→
3. 各能力的 `examples/` + `tests/`（**能力之间可并行**）→
4. 编译器行为观察（MIR / LLVM IR / 静态检查）→
5. `OBSERVATIONS.md`（模块级单文件，不可并行）→
6. `acceptance/criteria/cNN.md` →
7. `feynman/mN-*.md`（依赖前面全部产出作为论断依据，§C2）→
8. 模块验收 + capability-matrix 更新。

FR-002 的六环节在此顺序中一一对应：概念学习(2) / 最小实验(3) / 编译器行为观察(4) /
源码阅读(2 的 source-refs) / Feynman(7) / Acceptance Criteria(6)。缺任一环节模块
MUST NOT 标记完成。

### Parallel Opportunities

- Phase 1：T001–T002 可并行；T007–T011 可并行（不同文件）。
- Phase 2：T014–T017（harness 四个模块）可并行；T020–T023（四类模板）可并行。
- 每个 Story 内：`source-refs.md` 与各能力实验可并行；同一模块内**不同能力**的
  `examples/`+`tests/`+`compile_fail/` 完全不相交，可并行。
- Phase 11：T141、T142、T146、T147 可并行。
- **不可并行**：Story 之间（FR-011）；同一模块的 `concept.md` / `OBSERVATIONS.md` /
  `feynman/mN-*.md`（单文件）；使用 `CountingAllocator` 的测试（进程全局计数器，
  harness-api §并发约束）。

---

## Parallel Example: User Story 5（并行度最高的模块）

```bash
# m5 crate 骨架（T074）完成后，六项能力的 12 个成对实验可并行推进：
Task: "C-15 安全侧 examples/c15_unsafe.rs + tests/c15_unsafe.rs"          # T077
Task: "C-15 UB 对照 examples/c15_unsafe_ub.rs + tests/c15_unsafe_ub.rs"   # T078
Task: "C-16 安全侧 examples/c16_raw_ptr.rs + tests/c16_raw_ptr.rs"        # T079
Task: "C-16 UB 对照 examples/c16_raw_ptr_ub.rs + tests/..."               # T080
Task: "C-17 安全侧 / UB 对照"                                              # T081 / T082
Task: "C-18 安全侧 / UB 对照（核心教学对照）"                                 # T083 / T084
Task: "C-19 安全侧 / UB 对照 + Tree Borrows"                               # T085 / T086
Task: "C-20 安全侧 / UB 对照"                                              # T087 / T088

# 随后串行收口（共享文件或依赖全部实验）：
cargo test -p m5-unsafe                                    # T095
cargo +nightly miri test -p m5-unsafe                      # T091
MIRIFLAGS="-Zmiri-tree-borrows" cargo +nightly miri test -p m5-unsafe
```

---

## Implementation Strategy

### MVP First（User Story 1 Only）

1. 完成 Phase 1: Setup（含 T001–T004 门禁裁定）
2. 完成 Phase 2: Foundational（harness 是所有断言的地基）
3. 完成 Phase 3: User Story 1（m1-ownership）
4. **STOP and VALIDATE**：`cargo test -p m1-ownership` 全绿 + `feynman/m1-ownership.md`
   五项检验通过 + C-01…C-04 在 capability-matrix 中为 `accepted`
5. 此时已获得一个**完整闭环样板**（Learn → Source → Experiment → Feynman → Acceptance），
   后续七个模块复用同一形状

### Incremental Delivery

1. Setup + Foundational → 验证设施与追踪链就位
2. + US1（m1）→ 独立验证 → **MVP**，同时满足 FR-012 的第一个硬前置
3. + US2（m2）→ 独立验证
4. + US3（m3）→ 独立验证
5. + US4（m4）→ 独立验证（含 SC-007 一次性作答）
6. + US5（m5）→ 独立验证 → **第二个硬前置**（本 Feature 与 eBPF 耦合最紧的模块）
7. + US6（m6）→ 独立验证（SC-008）
8. + US7（m7）→ 独立验证 → **第三个硬前置**（SC-006）
9. + US8（m8）→ 综合验证（SC-009）
10. Polish → Feature 终验收 → Feature 002 准入判定

每个模块交付后，前面模块的验收状态 MUST 保持有效；`cargo test --workspace` 是该不变量的守卫。

### 单人学习工程的执行提示

- 本项目为单人学习工程，"并行"的实际含义是"这些任务之间无顺序约束，可按精力自由选序"，
  而非多人同时施工。
- 每个任务完成后立即提交（Constitution X 可复现性 + T024 的冻结时点需要提交时间为证）。
- 任何模块的 Feynman 五项检验若出现 fail，MUST 停止推进下一个 Story，并按 §Remediation
  把补齐任务写回本文件（FR-006 / Constitution Review gate）。

---

## 双轨产物任务（Dual-Track，T151–T161）

按 learning-artifact-contract §H 产出 Learner Track。这些任务**穿插**在既有 Phase 中执行，
不构成独立阶段 —— 每个模块的 T15x MUST 在该模块的 Answer Track 任务**之前**完成并提交
（§H3.4 的提交顺序纪律）。

### 归属 Phase 1（Setup）

- [X] T151 创建 `learner/README.md`：说明双轨读法、两个目录的职责区别（`learner/` 提问 / `learning/` 回答）、§H4 打开 Answer Track 的三个条件、以及 §H3 的六类禁止内容清单；MUST 说明"打开答案不影响验收判定"（§H4.3）

### 归属 Phase 2（Foundational）

- [ ] T152 [P] 创建 `learner/_templates/guide.md`、`predictions.md`、`selfcheck.md`：分别按 §H2.1 / §H2.2 / §H2.3 的 REQUIRED 小节；`guide.md` 模板含提示阶梯三级骨架，`predictions.md` 模板含"未命中复盘"块与 §H2.2a 的先填后跑规则声明

### 归属各 Story（MUST 先于该模块的 Answer Track 任务提交）

- [X] T153 [US1] 创建 `learner/m1-ownership/{guide,predictions,selfcheck}.md`：覆盖 C-01…C-04；引导问题针对 drop 时机、移动与 `Copy` 的区别、两次 `&mut` 被拒的规则、elision 何时失效；源码定位只给 `core/src/ops/`、`core/src/mem/`、`core/src/cell.rs`、`core/src/marker.rs` 的**目录/文件范围**，不给行号（L4）；预测表含错误码、drop 顺序、`size_of` 三类预测项，值留空（L1/L2）
- [ ] T154 [US2] 创建 `learner/m2-types/{guide,predictions,selfcheck}.md`：覆盖 C-05…C-07；预测项含 enum 布局、`Option<&T>` 是否与 `&T` 同宽、`&dyn Trait` 宽度、单态化实例数
- [ ] T155 [US3] 创建 `learner/m3-composition/{guide,predictions,selfcheck}.md`：覆盖 C-08…C-11；预测项以**分配次数**为核心（US3 AS1），MUST NOT 写出任何实测次数
- [ ] T156 [US4] 创建 `learner/m4-concurrency/{guide,predictions,selfcheck}.md`：覆盖 C-12…C-14；MUST 与 `acceptance/send-sync-quiz.md` 交叉引用但 MUST NOT 复述题目答案；预测项含 Send/Sync 判定、放宽内存序后哪些顺序变为可能
- [ ] T157 [US5] 创建 `learner/m5-unsafe/{guide,predictions,selfcheck}.md`：覆盖 C-15…C-20；预测表 MUST 为 12 个成对实验各留一行 **UB 类别事前预测**（填 W 编号，见 experiment-contract §C5.3），且 MUST NOT 列出白名单文本本身（L3）；提示阶梯 MUST 引导学习者自行写出 SAFETY 五要素而非给出范文
- [ ] T158 [US6] 创建 `learner/m6-ffi/{guide,predictions,selfcheck}.md`：覆盖 C-21；预测项含两侧 `size_of`/`align_of`/`offset_of` 是否一致、`errno` 封装后原始信息是否丢失
- [ ] T159 [US7] 创建 `learner/m7-nostd/{guide,predictions,selfcheck}.md`：覆盖 C-22…C-24；预测表 MUST 为 SC-006 固定清单的 6 条错误各留一行"归属哪一层"的事前预测，MUST NOT 给出归属答案
- [ ] T160 [US8] 创建 `learner/m8-capstone/{guide,predictions,selfcheck}.md`：综合场景的设计问题（不给设计方案）；自检 MUST 含"24 项能力你能各自定位到哪一层"的空白表

### 归属 Phase 11（Polish）

- [ ] T161 双轨完整性与不泄漏核查：(a) 8 个模块各有 `learner/mN-*/` 三件套；(b) 逐文件人工核查 §H3.1 的 L1–L6 六类内容零出现，结果写入 `acceptance/dual-track-audit.md`；(c) 核查每个模块 Learner Track 的提交时点早于该模块 Answer Track（`git log --diff-filter=A --format='%ad %H' -- <path>` 比对），违反者记录理由；(d) 抽查 `predictions.md` 的预测列提交时点早于对应验证命令的产物提交时点（§H2.2a）

---

## Remediation（补齐任务回写区）

**写入规则**（T150 建立）：当出现下列情形时，MUST 在本小节追加任务，ID 从 `T151` 起连续编号，
并在 `acceptance/capability-matrix.md` 中把相关 Capability 的 `Status` 置为 `regressed` 或
保持在 `experiment-passed`：

1. 某模块 Feynman 五项检验中任一项 fail（FR-006：不接受部分通过）；
2. 某能力的 AcceptanceCriterion 判定为 `fail`（data-model §1）；
3. 重跑时稳定断言未复现（`accepted → regressed`，FR-003 / SC-002）；
4. 工具链因不可避免的原因升级，受影响实验的验收记录需重新验证（FR-020）；
5. Miri 报告的 UB 类别与事前预测不一致（按 T001 裁定判 fail 并重写预测）。

追加格式与正文一致：`- [ ] T1NN [Story?] 描述 + 文件路径 + 触发原因（引用被违反的 FR/SC 编号）`。

_（当前为空——尚无补齐任务）_

---

## Notes

- `[P]` 任务 = 不同文件、无未完成依赖；本 Feature 中 Story 之间**永不**并行（FR-011）。
- `[Story]` 标签把任务映射到 US1…US8，与 `acceptance/capability-matrix.md` 的 `Task` 列
  共同构成 FR-013 七段追踪链的 **Task** 环节。
- 稳定断言只允许出现在 `tests/`；地址、耗时、线程交错、IR 文本、诊断全文只允许出现在
  `examples/` 与 `OBSERVATIONS.md`（R-05 / experiment-contract §C2.2）。
- "程序未崩溃" MUST NOT 作为无 UB 的证据；未运行 UB 工具时 `ub_verdict` 只能是 `n/a`（FR-019）。
- 每个任务完成后 MUST 可通过 experiment-contract §C8 的命令契约独立验证。
- MUST NOT 在本 Feature 编写任何 eBPF / Aya 程序（FR-017）；MUST NOT 执行
  `rustup update`（FR-020）。
