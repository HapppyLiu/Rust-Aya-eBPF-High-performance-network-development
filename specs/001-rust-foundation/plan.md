# Implementation Plan: Rust Foundation

**Branch**: `main`（本 Feature 未单独开分支，沿用 `specs/001-rust-foundation/`）| **Date**: 2026-09-04 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-rust-foundation/spec.md`

## Summary

把 24 项系统级 Rust 能力（C-01…C-24）转化为可机械验证的产物：8 个学习模块，每个模块一个
cargo crate；每项能力一个**可观察的 example**（打印现象）+ 一组**可断言的 test**（稳定断言）
+ 一条源码引用 + 一条 Acceptance Criterion；8 份 Feynman 教学材料按模块产出。

技术路径的三个支点：
1. **stable 1.98.0 为唯一验收工具链**，pinned nightly 仅作为 Miri/sanitizer/MIR 的分析工具（R-01）；
2. **Miri 为 UB 判定的第一依据**，FFI 场景回退到 ASan（R-02）——已实测出"程序正常输出 `2`、
   Miri 判定 UB"的对照，这正是 FR-019 存在的理由；
3. **稳定断言与非断言输出物理隔离**：断言只允许出现在 `tests/`，地址/耗时/线程交错只允许出现在
   `examples/` 与 `OBSERVATIONS.md`（R-05）。

依赖策略为默认零依赖，仅 US6 FFI 允许 `libc` + `cc`（R-08）。全部技术决策与验证记录见
[research.md](./research.md)。

## Technical Context

**Language/Version**: Rust **1.98.0 stable**（commit `88d9e12ae178fab0fb5cc050a94da85685d449ea`,
2026-08-18, LLVM 22.1.8），**Edition 2024**。分析工具链：**nightly 1.100.0**
（commit `17fd5b8a37b6667b6cc137f3cc35f09759768a3b`, 2026-08-28），仅用于 Miri / sanitizer /
`-Z unpretty=mir`，MUST NOT 产出稳定断言。（R-01）

**Primary Dependencies**: 默认零外部 crate。例外仅两项，且限定作用域于 `experiments/m6-ffi`：
`libc`（Linux API 与 C 类型别名）、`cc`（build-dependency，编译配套 C 源码）。（R-08）

**Storage**: N/A —— 学习产物为仓库内的 Markdown 与 Rust 源码，无运行时持久化。

**Testing**: `cargo test --workspace`（稳定断言）；`cargo +nightly miri test` / `miri run`
（UB 判定）；`rf-harness::compile_fail` 的错误码断言（编译失败类实验）。
辅助校验：`cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --check`。

**Target Platform**: Linux x86_64（WSL2 kernel `6.6.114.1-microsoft-standard-WSL2`）。
主机 target `x86_64-unknown-linux-gnu`；`no_std` 实验 target `x86_64-unknown-none`（stable 上已安装）。（R-03）

**Project Type**: Learning project + executable experiments —— 单 cargo workspace（8 个模块 crate
+ 1 个 harness crate）+ 1 个被 workspace 排除的独立 `no_std` crate，配套 learning / feynman /
acceptance 三个文档目录。

**Performance Goals**: 本 Feature **不设性能目标，且不产生任何需要 benchmark 的性能主张**。
"运行时代价"一律改用确定性可测量量表达：分配次数（`CountingAllocator`）、内存布局
（`size_of`/`align_of`/`offset_of`）、分发结构（MIR/LLVM IR 观察）。计时数据 MUST 标注为
NON-ASSERTION。（R-07）

**Constraints**:
- 全程锁定 R-01 指定的工具链版本；Feature 期间 MUST NOT 执行 `rustup update`（FR-020）。
- 稳定断言中 MUST NOT 出现地址、耗时、线程交错、`{:p}` 输出（FR-003 / R-05）。
- "程序未崩溃" MUST NOT 作为无 UB 的证据（FR-019）。
- MUST NOT 编写任何 eBPF / Aya 程序（FR-017）。
- 学习顺序 US1→US8 递进，无跳过（FR-011）。

**Scale/Scope**: 8 个学习模块 / 24 项能力 / ≥24 个 example + ≥24 个 test 文件 /
8 份 Feynman 材料 / 24 条 Acceptance Criteria / 1 个综合实验。实验代码规模按"最小可观察"控制，
单个 example 目标 < 80 行。

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.0.0。**Plan gate** 明确要求："Plan MUST 为每个目标指明源码定位范围（II）与至少
一个可运行实验（III）；涉及 unsafe 或 no_std 的目标 MUST 标注 VI / VII 适用。" 该要求由下方
**Capability Gate Matrix** 逐项满足。

| Principle | 状态 | 本 Plan 中的落点 |
|-----------|------|----------------|
| I. First-Principles Learning | PASS | 每项能力的验证阶梯要求下沉到编译器可观测行为（R-04），而非停留在 API 描述；`CountingAllocator` 要求学习者亲手实现 `GlobalAlloc` 而非引用现成库 |
| II. Source-Code-First | PASS | Capability Gate Matrix 为 24 项能力逐项指定源码定位范围（路径 + 符号），实施时按 FR-005 记录精确行 |
| III. Experiment-Driven | PASS | 每项能力 ≥1 个 `examples/cNN_*.rs`（可观察）+ ≥1 个 `tests/cNN_*.rs`（可复现断言）；命令记录在 quickstart.md |
| IV. Feynman Explanation | PASS | `feynman/mN-*.md` 8 份，模板强制五项检验小节（contracts/learning-artifact-contract.md），任一项缺失即模块未完成 |
| V. Acceptance-Criteria-Driven | PASS | `acceptance/criteria/cNN.md` 24 条，格式强制"可运行命令 + 可观测判据"，禁止"看过/了解过" |
| VI. Unsafe-Rust-Safety | PASS | m5/m6/m8 全部 unsafe 块强制 `// SAFETY:` 覆盖有效性/对齐/别名/provenance/生命周期五要素；由 Miri 与 clippy `undocumented_unsafe_blocks` 双重把关 |
| VII. no_std-Awareness | PASS | m7 用真实裸机 target `x86_64-unknown-none` 而非"仅加 `#![no_std]`"，强制暴露 panic handler 与 OS services 缺席（R-03） |
| VIII. Linux-Kernel-Awareness | PARTIAL（已justify） | 本 Feature 是 Rust 侧地基，内核路径覆盖仅通过 m6-ffi 的真实 syscall（`open`/`close`/`errno`）建立最小接触点；完整内核路径属 Feature 002+。见 Complexity Tracking |
| IX. Performance-Is-Measured | PASS | 不产生需 benchmark 的性能主张；代价论断一律用确定性量（分配次数/布局/IR 结构）；计时一律标 NON-ASSERTION（R-07） |
| X. Reproducibility | PASS | `rust-toolchain.toml` 锁定版本；`tools/env-record.sh` 为每次实验生成环境记录（toolchain/kernel/arch/命令）；一致性判定 = `cargo test --workspace` 全绿 |
| XI. Incremental Complexity | PASS | m1→m8 严格递进，无跳过，m8 依赖 m1–m7 全部通过；US1/US5/US7 为 Feature 002 硬前置（FR-012） |
| XII. Learn → Explain → Build | PASS | 每模块闭环：`learning/` → `source-refs.md` → `examples/`+`tests/` → `feynman/` → `acceptance/` → m8 Build。Benchmark 环节按 IX 以确定性测量替代 |
| XIII. Knowledge Must Be Traceable | PASS | `acceptance/capability-matrix.md` 为单一事实源，逐行串起 C-ID → Story → 实验文件 → 源码引用 → 验收标准；孤立笔记为 0 |
| XIV. Final Capability | PASS | 本 Feature 只承担终态能力清单中的 "Rust systems programming / Unsafe Rust" 两项地基，不降级目标；FR-017 明确禁止在此提前写 eBPF |

**Gate 判定**：通过。唯一偏离项（VIII 部分覆盖、FR-020 双工具链）已在 Complexity Tracking 登记理由。

### Capability Gate Matrix

Constitution Plan gate 的逐项落实。**Exp** 列为该能力的主实验（`examples/` 与 `tests/` 同名）；
**VI/VII** 标注 unsafe / no_std 原则适用性；**UB 判定**列给出 FR-019 的判定工具。

| C-ID | Capability | 模块 | Exp（`cNN_*`） | 源码定位范围（II） | VI | VII | UB 判定 |
|------|-----------|------|---------------|------------------|----|----|--------|
| C-01 | Ownership | m1 | `c01_ownership` | `core/src/ops/drop.rs` (`Drop`)、`core/src/marker.rs` (`Copy`) | – | – | – |
| C-02 | Move semantics | m1 | `c02_move` | `core/src/mem/mod.rs` (`replace`/`take`/`forget`)、`core/src/marker.rs` (`Copy`) | – | – | – |
| C-03 | Borrowing | m1 | `c03_borrow` | `core/src/cell.rs` (`RefCell`/`BorrowFlag`)；borrowck 属编译器内建 → 按 FR-005 记 Reference | – | – | – |
| C-04 | Lifetime | m1 | `c04_lifetime` | `core/src/marker.rs` (`PhantomData`)；elision 规则按 FR-005 记 Reference | – | – | – |
| C-05 | Struct / Enum | m2 | `c05_layout` | `core/src/option.rs` (`Option` niche)、`core/src/mem/mod.rs` (`size_of`) | – | – | – |
| C-06 | Trait | m2 | `c06_trait` | `core/src/fmt/mod.rs` (`Display`)、`core/src/ops/deref.rs` (`Deref`) | – | – | – |
| C-07 | Generic | m2 | `c07_generic` | `core/src/iter/traits/iterator.rs`、`core/src/cmp.rs` (`PartialOrd`) | – | – | – |
| C-08 | Error handling | m3 | `c08_error` | `core/src/result.rs`、`core/src/convert/mod.rs` (`From`)、`std/src/error.rs` | – | – | – |
| C-09 | Iterator | m3 | `c09_iterator` | `core/src/iter/traits/iterator.rs`、`core/src/iter/adapters/map.rs` | – | – | – |
| C-10 | Closure | m3 | `c10_closure` | `core/src/ops/function.rs` (`Fn`/`FnMut`/`FnOnce`) | – | – | – |
| C-11 | Smart pointer | m3 | `c11_smart_ptr` | `alloc/src/boxed.rs`、`alloc/src/rc.rs`、`alloc/src/sync.rs` (`Arc`) | – | – | – |
| C-12 | Send / Sync | m4 | `c12_send_sync` | `core/src/marker.rs` (`Send`/`Sync` unsafe auto trait + 负向 impl) | – | – | Miri |
| C-13 | Concurrency | m4 | `c13_concurrency` | `std/src/thread/mod.rs`、`std/src/sync/mutex.rs` | – | – | Miri `-Zmiri-many-seeds` |
| C-14 | Atomic | m4 | `c14_atomic` | `core/src/sync/atomic.rs` (`Ordering`、`AtomicUsize`) | 适用 | – | Miri `-Zmiri-many-seeds` |
| C-15 | Unsafe Rust | m5 | `c15_unsafe` | `core/src/slice/mod.rs` (`get_unchecked`)；UB 定义按 FR-005 记 Reference | **适用** | – | **Miri** |
| C-16 | Raw pointer | m5 | `c16_raw_ptr` | `core/src/ptr/mod.rs`、`core/src/ptr/const_ptr.rs` (`read`/`write`) | **适用** | – | **Miri** |
| C-17 | Pointer arithmetic | m5 | `c17_ptr_arith` | `core/src/ptr/const_ptr.rs` (`add`/`offset`/`wrapping_add`) | **适用** | – | **Miri** |
| C-18 | Alignment | m5 | `c18_alignment` | `core/src/mem/mod.rs` (`align_of`)、`core/src/ptr/mod.rs` (`read_unaligned`) | **适用** | – | **Miri** |
| C-19 | Aliasing | m5 | `c19_aliasing` | `core/src/cell.rs` (`UnsafeCell`) | **适用** | – | **Miri** (Stacked + Tree Borrows 对照) |
| C-20 | Memory safety | m5 | `c20_mem_safety` | `core/src/slice/raw.rs` (`from_raw_parts`)、`alloc/src/vec/mod.rs` (`set_len`) | **适用** | – | **Miri** |
| C-21 | FFI | m6 | `c21_ffi` | `core/src/ffi/mod.rs` (`c_int`/`c_char`)、`std/src/ffi/c_str.rs` (`CStr`) | **适用** | – | **ASan**（Miri 不支持真实 C 调用，R-02） |
| C-22 | no_std | m7 | `c22_nostd` | `core/src/lib.rs` (`#![no_std]`)、`std/src/lib.rs` | 适用 | **适用** | 编译期 + `nm`/`readelf` |
| C-23 | core / alloc / std | m7 | `c23_core_alloc_std` | `alloc/src/lib.rs`、`std/src/lib.rs`（re-export 关系） | – | **适用** | 编译期 |
| C-24 | Panic & allocator | m7 | `c24_panic_alloc` | `core/src/panicking.rs`、`core/src/alloc/global.rs` (`GlobalAlloc`)、`alloc/src/alloc.rs` | **适用** | **适用** | 编译期 + Miri（host 侧 allocator） |

源码根路径：`$(rustc --print sysroot)/lib/rustlib/src/rust/library/`（`rust-src` 组件已安装）。

### Post-Design Re-check（Phase 1 完成后）

Phase 1 产出 data-model.md、contracts/×3、quickstart.md 后重新核对，**结论：仍然通过**，
且以下三项由"计划意图"变为"契约中的可检查条款"：

| 原则 | Phase 1 强化点 |
|------|--------------|
| IV. Feynman | `learning-artifact-contract.md` §C 把五项检验固化为五个 REQUIRED 小节，缺一即模块 `failed`；论断需可追溯到实验或源码引用 |
| V. Acceptance-Criteria | 同上 §D 规则 D1 列出禁用措辞清单，D2 要求"至少一条判据由命令退出码决定"，使验收不能完全依赖自评 |
| VI. Unsafe-Rust-Safety | `experiment-contract.md` §C6 把 Safety Invariant 拆为五要素逐项覆盖，并用 clippy `undocumented_unsafe_blocks` 做机械兜底 |
| X. Reproducibility | §C2.2 给出稳定断言的**禁止内容清单**（地址/耗时/线程交错/哈希顺序/诊断全文），使 FR-003 可机械检查 |
| — FR-019 强化 | `harness-api.md` 规定 `MiriOutcome::reported_ub()` 在 Miri 未运行时 **panic** 而非返回 `false`，在类型层面阻止"没跑工具"被当成"没有 UB" |

设计过程中未引入新的原则偏离；Complexity Tracking 的三条登记项保持不变。

## Project Structure

### Documentation (this feature)

```text
specs/001-rust-foundation/
├── plan.md                  # This file (/speckit-plan command output)
├── research.md              # Phase 0 output — R-01..R-10 技术决策
├── data-model.md            # Phase 1 output — 7 个实体的字段/关系/状态机
├── quickstart.md            # Phase 1 output — 可运行的验证场景
├── contracts/               # Phase 1 output
│   ├── experiment-contract.md        # 实验产物的结构与稳定断言规则
│   ├── harness-api.md                # rf-harness 公开 API 契约
│   └── learning-artifact-contract.md # learning/feynman/acceptance 文档 schema
├── spec.md
├── instructions.md
├── checklists/requirements.md
└── tasks.md                 # Phase 2 output (/speckit-tasks — NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
rust-toolchain.toml                     # 锁定 stable 1.98.0 + components + targets (R-01)
Cargo.toml                              # workspace; exclude = ["experiments/m7-nostd"]
rustfmt.toml / clippy.toml

harness/                                # rf-harness：共享验证设施，不含学习内容
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── compile_fail.rs                 # 错误码级断言器（零依赖，R-06）
    ├── counting_alloc.rs               # CountingAllocator：确定性分配计数（R-07）
    └── env.rs                          # 环境记录生成

experiments/                            # 可执行实验：一模块一 crate
├── m1-ownership/                       # C-01..C-04  (US1, P1)
│   ├── Cargo.toml
│   ├── src/lib.rs                      # 被 example/test 复用的最小类型与函数
│   ├── examples/c01_ownership.rs       # 可观察：打印现象（NON-ASSERTION 输出）
│   ├── examples/c02_move.rs
│   ├── examples/c03_borrow.rs
│   ├── examples/c04_lifetime.rs
│   ├── tests/c01_ownership.rs          # 可断言：稳定断言（验收单位）
│   ├── tests/c02_move.rs
│   ├── tests/c03_borrow.rs
│   ├── tests/c04_lifetime.rs
│   ├── compile_fail/                   # MUST NOT 编译的样本 + 期望错误码
│   │   ├── c03_two_mut_borrows.rs
│   │   └── c04_dangling_ref.rs
│   └── OBSERVATIONS.md                 # 实际输出 + 环境记录 + NON-ASSERTION 标注
├── m2-types/                           # C-05..C-07  (US2, P2)
├── m3-composition/                     # C-08..C-11  (US3, P2)
├── m4-concurrency/                     # C-12..C-14  (US4, P2)
├── m5-unsafe/                          # C-15..C-20  (US5, P1) —— Miri 重点区
├── m6-ffi/                             # C-21        (US6, P2)
│   ├── build.rs                        # cc 编译 c/ 下的 C 源
│   └── c/roundtrip.c                   # C 调 Rust / Rust 调 C 的双向最小实验
├── m7-nostd/                           # C-22..C-24  (US7, P1) —— 独立 workspace
│   ├── .cargo/config.toml              # target = "x86_64-unknown-none"
│   └── src/main.rs                     # #![no_std] #![no_main] + panic_handler
└── m8-capstone/                        # 综合实验    (US8, P3) —— #![no_std] + alloc 风格

learning/                               # 学习材料（概念 + 源码引用）
├── m1-ownership/{concept.md,source-refs.md}
└── ... m2 .. m8

feynman/                                # Feynman 教学材料：按模块 8 份，强制五项检验
├── m1-ownership.md
└── ... m2 .. m8

acceptance/                             # 验收
├── capability-matrix.md                # C-01..C-24 单一事实源（FR-013 追踪链）
├── criteria/c01.md .. c24.md           # 24 条 Acceptance Criteria
├── send-sync-quiz.md                   # ≥10 题，US4 开始前定稿（SC-007 / R-10）
├── send-sync-quiz.answers.md           # 作答前 MUST NOT 打开
└── unfamiliar-code-reading.md          # SC-004 / SC-005 的计时评估素材与记录

tools/
├── env-record.sh                       # 生成 Environment Record
├── emit-mir.sh / emit-llvm-ir.sh       # 编译器中间表示导出（R-04 阶梯 3–4）
├── run-miri.sh                         # 统一 MIRIFLAGS 的 UB 判定入口
└── run-asan.sh                         # US6 的 FFI UB 判定入口
```

**Structure Decision**：采用**四分目录 + 单 workspace**（R-09）。
`learning/`（概念与源码引用）、`experiments/`（可执行实验）、`feynman/`（教学材料）、
`acceptance/`（验收标准与判定题）四个顶层目录一一对应用户要求的四类产物，物理分离避免混杂。

实验 crate 粒度取 **8**（对齐 8 个 Feynman 模块与模块级验收），实验文件粒度取 **24**
（对齐能力级 Acceptance Criteria），精确落实 Clarification"Feynman 按 8 个模块产出、24 项能力
各自保留独立 AC 与实验"的裁定。每个 example 与 test 都是独立的 cargo 目标，
`cargo run -p m5-unsafe --example c18_alignment` 可单独执行，满足"小、独立、可运行、可观察"。

`experiments/m7-nostd` 被根 workspace `exclude`，因为 `#![no_main]` + 自定义 `#[panic_handler]`
的 crate 无法用 host target 构建，会污染 `cargo test --workspace` 的一键验证（R-03）。

## Complexity Tracking

> 已知的原则偏离，按 Constitution Governance 要求显式登记理由与补救计划。

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| **双工具链**（stable 1.98.0 构建 + nightly 1.100.0 分析），与 FR-020"锁定单一版本"的字面表述偏离 | FR-019 强制要求 UB 检测工具作为判定依据，而 Miri 仅以 nightly 组件发布，stable 无法安装（已实测）。二者不可能由单一 stable 版本同时满足 | *统一 pin nightly*：字面合规，但 nightly 静默允许 unstable 特性，会让学习者在建立**稳定 Rust 语义**基线时误用不稳定特性而不自知，污染整个后续知识链。*放弃 Miri*：直接违反 FR-019，且 x86_64 硬件容忍未对齐访问，US5 AS2 将完全无法观测。**补救**：nightly 版本同样按 commit-hash 精确锁定，且限定用途——MUST NOT 产出稳定断言、MUST NOT 编译交付产物；两个版本号均写入每份环境记录 |
| **Constitution VIII（Linux-Kernel-Awareness）仅部分覆盖** | 本 Feature 是 Rust 侧地基，Spec 的 Scope 明确排除 eBPF/Aya 实作（FR-017），Out of scope 亦未包含内核子系统学习 | 在本 Feature 强行加入内核路径学习会违反 Constitution XI（前置未验收即进入复杂系统）与 FR-017。**补救**：通过 m6-ffi 的真实 syscall（`open`/`close`/`errno`）建立最小内核接触点，完整的 syscall / skb / NAPI / XDP 路径覆盖由 Feature 002+ 承担；本偏离不影响 FR-012 的硬前置判定 |
| **Constitution XII 闭环中的 Benchmark 环节以确定性测量替代** | Technical Context 明确本 Feature 不追求性能；WSL2 上的计时抖动会使计时结果既无法作稳定断言（FR-003），又会诱导错误的性能结论 | *引入 criterion 微基准*：与"不追求性能"冲突，引入重依赖与长运行时间，且 Constitution IX 要求"未测量的性能主张 MUST 标注为假设"——正确做法是**不产生**此类主张。**补救**：用分配次数、内存布局、IR 结构等确定性量承担"代价可测量"职责（R-07）；真正的 throughput/latency benchmark 推迟到有性能目标的后续 Feature |
