# Phase 1 Data Model: Rust Foundation

**Feature**: 001-rust-foundation | **Date**: 2026-09-04 | **Plan**: [plan.md](./plan.md)

本 Feature 的"数据"是**学习产物本身**，没有运行时数据库。本文件定义 Spec `Key Entities` 中 7 个实体
的字段、关系、校验规则与状态迁移，使它们可被机械校验（而非靠人工回忆判断是否完成）。

实体的物理载体全部在版本控制中：`acceptance/capability-matrix.md` 是**单一事实源**，
其余文件是其展开。

---

## Entity Relationship

```text
Capability (24)  ──belongs to──▶  LearningModule (8)  ──has one──▶  FeynmanMaterial (8)
     │                                    │
     ├──has 1..n──▶ Experiment            └──has one──▶ ConceptNote
     │                   │
     │                   └──has one──▶ EnvironmentRecord
     │                   └──has 1..n──▶ StableAssertion
     │                   └──has 0..n──▶ NonAssertionOutput
     ├──has 1..n──▶ SourceReference
     └──has exactly 1──▶ AcceptanceCriterion
```

基数约束（对应 FR-001、FR-003、FR-005、FR-007）：
- 每个 Capability MUST 归属**恰好一个** LearningModule；无归属的 Capability 视为规格缺陷。
- 每个 Capability MUST 有 **≥1** 个 Experiment、**≥1** 条 SourceReference、**恰好 1** 条 AcceptanceCriterion。
- 每个 LearningModule MUST 有**恰好 1** 份 FeynmanMaterial（Clarification：Feynman 按模块产出）。
- 每个 Experiment MUST 有 **≥1** 条 StableAssertion 与**恰好 1** 条 EnvironmentRecord。

---

## 1. Capability（能力项）

本 Feature 的最小可验收单位。

| Field | Type | Rule |
|-------|------|------|
| `id` | `C-01`…`C-24` | 唯一，不可重编号（后续 Feature 会引用） |
| `name` | string | 与 spec.md Capability Coverage 表逐字一致 |
| `module` | `m1`…`m8` | 恰好一个；与 plan.md Capability Gate Matrix 一致 |
| `unsafe_applicable` | bool | 为真时 Constitution VI 生效，产物 MUST 含 Safety Invariant |
| `nostd_applicable` | bool | 为真时 Constitution VII 生效，MUST 区分 core/alloc/std/OS services |
| `ub_tool` | `miri` \| `asan` \| `compile-time` \| `n/a` | FR-019 判定工具，取自 Gate Matrix |
| `downstream_link` | string | FR-014：与后续 Linux/eBPF/Aya 的关联点，或显式标注"仅为理解基础" |
| `status` | State | 见下方状态机 |

**State machine**（`status`）：

```text
planned ──▶ in-progress ──▶ experiment-passed ──▶ accepted
                 ▲                                    │
                 └──────────── regressed ◀────────────┘
```

- `planned → in-progress`：概念材料与源码引用已建立。
- `in-progress → experiment-passed`：实验的全部 StableAssertion 通过，且（若 `ub_tool != n/a`）
  UB 工具输出符合实验的**预期判定**。
- `experiment-passed → accepted`：AcceptanceCriterion 判定通过 **且**所属模块的 FeynmanMaterial
  五项检验全部通过（FR-006：任一项未通过则模块未完成 → 该模块下所有 Capability 不得进入 `accepted`）。
- `accepted → regressed`：重跑时稳定断言未复现，或工具链被迫升级导致记录失效（FR-020）。
  `regressed` MUST 生成补齐任务回写 `tasks.md`（Constitution Review gate）。

**Validation**：`status = accepted` 是唯一可计入进度的状态。"看过""了解过""做过笔记"
MUST NOT 触发任何状态迁移（FR-007）。

---

## 2. LearningModule（学习模块）

与 Story 一一对应，共 8 个。物理载体：`experiments/mN-*/` + `learning/mN-*/` + `feynman/mN-*.md`。

| Field | Type | Rule |
|-------|------|------|
| `id` | `m1`…`m8` | 唯一 |
| `story` | `US1`…`US8` | 一一对应 |
| `priority` | `P1` \| `P2` \| `P3` | 取自 spec.md |
| `capabilities` | `[C-ID]` | 非空；并集必须精确等于 C-01..C-24（无重复、无遗漏） |
| `crate_path` | path | `experiments/mN-*`（m7 为独立 workspace） |
| `prerequisite` | `mN` \| null | FR-011 递进：m2 依赖 m1，…，m8 依赖 m1–m7 |
| `feynman_status` | `pending` \| `passed` \| `failed` | 五项检验的合取，不接受部分通过（FR-006） |
| `status` | `pending` \| `complete` | `complete` 要求：所属 Capability 全部 `accepted` **且** `feynman_status = passed` |

**Composition rule**（FR-002）：模块 MUST 同时具备六个环节，缺一不可 ——
概念学习 / 最小 Rust 实验 / 编译器行为观察 / Rust 源码阅读 / Feynman 教学材料 / Acceptance Criteria。

**Hard prerequisite**（FR-012）：`m1`、`m5`、`m7` 三个模块 `status = complete` 是 Feature 002
启动的准入条件，不接受"边学边补"。

---

## 3. Experiment（实验）

可重复执行的最小验证单元。物理载体：一个 `examples/cNN_*.rs`（可观察）+ 一个
`tests/cNN_*.rs`（可断言），必要时加 `compile_fail/*.rs`。

| Field | Type | Rule |
|-------|------|------|
| `id` | `cNN_<slug>` | 与文件名一致 |
| `capability` | `C-ID` | 所属能力 |
| `kind` | `runtime` \| `compile-fail` \| `ir-observation` \| `ub-demo` | 决定验证方式 |
| `command` | string | 可直接复制执行的完整命令（FR-003） |
| `stable_assertions` | `[StableAssertion]` | 非空 |
| `non_assertion_outputs` | `[NonAssertionOutput]` | 可空 |
| `env` | EnvironmentRecord | 必填（FR-010） |
| `ub_verdict` | `clean` \| `expected-ub` \| `unexpected-ub` \| `n/a` | FR-019 判定结果 |

**`kind` 语义**：
- `runtime` —— 编译通过并运行，断言来自 `#[test]`。
- `compile-fail` —— MUST NOT 编译成功；断言是 stderr 中出现的**错误码**（如 `E0502`）。
- `ir-observation` —— 断言来自 `size_of`/`align_of`/分配计数等确定性量；MIR/LLVM IR 文本本身
  记为非断言输出。
- `ub-demo` —— 实验意图就是触发 UB。此时 `ub_verdict = expected-ub` 才算**通过**，
  且 Miri 报告的错误类别文本是稳定断言的一部分。

**Critical rule**（FR-019）：`ub_verdict` MUST 取自 UB 工具输出。程序正常退出
MUST NOT 使 `ub_verdict` 被置为 `clean`——未运行 UB 工具时该字段只能是 `n/a`。

---

## 4. StableAssertion（稳定断言）

重跑时**必须复现**的判定条件。这是 Clarification"重跑仅比对稳定断言"的落地实体。

| Field | Type | Rule |
|-------|------|------|
| `location` | path::fn | MUST 位于 `tests/` 或 `#[cfg(test)]` 中的 `#[test]` 函数 |
| `claim` | string | 一句话陈述该断言验证的事实 |
| `forbidden_content` | — | 断言表达式 MUST NOT 包含：指针地址、`{:p}` 输出、时间测量、线程调度顺序、哈希遍历顺序、进程 PID |

**Enforcement**：断言与观测的分离是**物理的**（`tests/` vs `examples/`），不依赖自律。
详见 [contracts/experiment-contract.md](./contracts/experiment-contract.md)。

---

## 5. NonAssertionOutput（非断言输出）

天然可变、MUST NOT 计入一致性判定的输出。

| Field | Type | Rule |
|-------|------|------|
| `source` | `examples/cNN_*.rs` | MUST NOT 来自 `tests/` |
| `category` | `address` \| `timing` \| `thread-interleaving` \| `ir-text` \| `diagnostic-text` | |
| `recorded_value` | text | 抄录到 `OBSERVATIONS.md`，标注 `NON-ASSERTION` |
| `interpretation` | string | **必填**：该现象说明了什么。仅记录现象而无解释者，按 Spec Edge Case 视为未完成 |

---

## 6. SourceReference（源码引用）

| Field | Type | Rule |
|-------|------|------|
| `capability` | `C-ID` | |
| `path` | string | 相对 `$(rustc --print sysroot)/lib/rustlib/src/rust/library/` 的路径 |
| `symbol` | string | 具体的结构体 / 函数 / trait 名 |
| `line` | int | 实施时记录的实际行号（随 pinned 工具链固定） |
| `kind` | `library` \| `reference-fallback` | |
| `note` | string | 该符号回答了什么问题 |

**Fallback rule**（FR-005）：当能力属编译器内建而无库代码对应（如 borrow checker、lifetime elision），
`kind = reference-fallback`，`path` 改记 Rust Reference 章节或编译器实现位置。已知需 fallback 的
能力：C-03（borrowck）、C-04（elision）、C-15（UB 定义）。

---

## 7. FeynmanMaterial（Feynman 教学材料）

按模块产出，共 8 份。物理载体：`feynman/mN-*.md`。

| Field | Type | Rule |
|-------|------|------|
| `module` | `m1`…`m8` | 一一对应 |
| `covered_capabilities` | `[C-ID]` | MUST 精确等于该模块的 `capabilities`（SC-001 要求 100% 覆盖） |
| `check_1_self_explanation` | pass/fail | 用自己的语言解释概念 |
| `check_2_minimal_example` | pass/fail | 给出最小示例 |
| `check_3_mechanism` | pass/fail | 解释底层机制 |
| `check_4_misconceptions` | pass/fail | 解释常见误区 |
| `check_5_questions` | pass/fail | 回答验证性问题 |

**Rule**（FR-006 / Constitution IV）：五项为**合取**，任一项 fail 则模块判定为**未完成**，
并 MUST 生成补齐任务。不接受部分通过。

---

## 8. AcceptanceCriterion（验收标准）

| Field | Type | Rule |
|-------|------|------|
| `capability` | `C-ID` | 一一对应，共 24 条 |
| `verification_command` | string | 可执行命令；MUST NOT 为空 |
| `observable_verdict` | string | 可观测的通过判据 |
| `result` | `pass` \| `fail` \| `not-evaluated` | |

**Forbidden**（FR-007）：`observable_verdict` MUST NOT 使用"看过""了解过""做过笔记""熟悉"
一类不可判定的措辞。合格形式参照 Constitution V：能解释 / 能画图 / 能写代码 / 能运行实验 /
能分析输出 / 能定位源码 / 能完成测试。

---

## 9. EnvironmentRecord（环境记录）

| Field | Source | Example |
|-------|--------|---------|
| `rustc_stable` | `rustc -Vv` | `1.98.0 (88d9e12ae 2026-08-18)` |
| `rustc_nightly` | `rustup run nightly rustc -Vv` | `1.100.0-nightly (17fd5b8a3 2026-08-28)` |
| `edition` | Cargo.toml | `2024` |
| `kernel` | `uname -r` | `6.6.114.1-microsoft-standard-WSL2` |
| `arch` | `uname -m` | `x86_64` |
| `target` | cargo | `x86_64-unknown-linux-gnu` / `x86_64-unknown-none` |
| `command` | — | 实际执行的完整命令 |
| `crates` | Cargo.lock | 仅 m6：`libc`、`cc` 的精确版本 |

**Rule**（FR-010 / FR-018 / Constitution X）：每个实验必填。当实验结果依赖 CPU 架构或编译器版本时
（典型：C-14 内存序在 x86_64 上的表现、C-18 未对齐访问），MUST 额外说明该结果**是否可跨架构推广**。
由 `tools/env-record.sh` 自动生成，避免手工遗漏。

---

## Traceability（FR-013 / SC-011）

`acceptance/capability-matrix.md` 是追踪链的单一事实源，每行一个 Capability：

```text
C-ID | Capability | Module | Story | Experiment | SourceRef | AcceptanceCriterion | UB Tool | Status
```

**完整链条**：`spec.md (FR/US)` → `plan.md (Gate Matrix)` → `tasks.md (task id)` →
`learning/mN/concept.md` → `experiments/mN/{examples,tests}/cNN_*` →
`learning/mN/source-refs.md` → `acceptance/criteria/cNN.md`。

**孤立笔记检查**：任何 `learning/` 或 `feynman/` 下的内容若无法在 capability-matrix 中找到归属，
即为孤立笔记，MUST 在下一次 Review 中并入链条或删除（Constitution XIII，SC-011 要求数量为 0）。
