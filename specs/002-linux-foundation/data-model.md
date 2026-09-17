# Phase 1 Data Model: Linux Foundation

**Feature**: 002-linux-foundation | **Date**: 2026-09-17 | **Plan**: [plan.md](./plan.md)

本 Feature 的“数据”是学习产物本身。`acceptance/002-linux-foundation/capability-matrix.md`
是单一事实源。

双轨与独立验证：每个 LearningModule 可单独走到 `complete` 的**文件与断言检查**
（不依赖其它模块的运行时状态）；学习顺序仍是 m1→m7。

---

## Entity Relationship

```text
Capability (27)  ──belongs to──▶  LearningModule (7)  ──has one──▶  FeynmanMaterial (7)
     │                                    │
     ├──has exactly 1──▶ Experiment       ├──has one──▶ ConceptNote
     │                   │                ├──has one──▶ LearnerGuide
     │                   └── EnvironmentRecord
     ├──has 1..n──▶ SourceReference       ├──has one──▶ Quiz + AnswerKey
     ├──has exactly 1──▶ AcceptanceCriterion
     └──has exactly 1──▶ DownstreamLink
```

基数约束：
- 每个 Capability 恰好一个 LearningModule；恰好 1 个主 Experiment；≥1 SourceReference；恰好 1 AC；恰好 1 DownstreamLink。
- 每个 LearningModule 恰好 1 份 Feynman 提问版 + 1 份参考回答、1 份 Quiz、1 份 Learning Framework。
- 每个 Experiment 恰好 1 条 EnvironmentRecord；≥1 条 StableAssertion；恰好 1 个主观察通道；自包含 setup/cleanup。

---

## 1. Capability

| Field | Type | Rule |
|-------|------|------|
| `id` | `C-01`…`C-27` | 唯一 |
| `name` | string | 与 spec Capability Coverage 逐字一致 |
| `module` | `m1`…`m7` | 恰好一个 |
| `downstream_link` | string | FR-014，禁止空白 |
| `privilege` | `user` \| `root` | C-04、C-06、C-26 = `root`；其余默认 `user` |
| `host_for_accepted` | `ubuntu-vm` | 最终验收只在 Ubuntu 虚拟机 |
| `status` | State | 五态机 |

```text
planned ──▶ in-progress ──▶ experiment-passed ──▶ accepted
                 ▲                                    │
                 └──────────── regressed ◀────────────┘
```

`experiment-passed → accepted` 要求：AC 在 **Ubuntu VM** 上通过 **且** 模块 Feynman 五项全过
**且** Quiz Answer Key 已分离 **且**（若 privilege=root）以 root 跑通主观察。

偏离宿主上的等价观察最多到 `experiment-passed`，矩阵备注 `off-baseline-fallback`。
基准 VM 上工具缺失 MUST NOT 进入 `experiment-passed`。

---

## 2. LearningModule

| Field | Type | Rule |
|-------|------|------|
| `id` | `m1`…`m7` | |
| `story` | `US1`…`US7` | 一一对应 |
| `priority` | P1/P2/P3 | |
| `capabilities` | `[C-ID]` | 并集精确等于 C-01..C-27 |
| `learn_prerequisite` | m(N-1) | 学习顺序 FR-011 |
| `verify_independent` | bool | **true**：`lf-verify-module.sh` 不依赖其它模块运行时状态 |
| `feynman_status` | pending/passed/failed | 五项合取 |
| `status` | pending/complete | 全部 capability accepted 且 feynman passed |

七个模块：`m1-cli-observe` `m2-process-syscall` `m3-memory` `m4-vfs-io` `m5-sched-ipc` `m6-net` `m7-capstone`。

Composition（FR-002）：Framework / Teaching / Feynman Questions / Experiment Tasks / Quiz / Answer Key / AC。

---

## 3. Experiment

| Field | Rule |
|-------|------|
| `slug` | `cNN_<name>`，与 Gate Matrix Exp 列一致 |
| `size` | example ≤ 80 行 |
| `observe_channel` | 恰好一个主通道（proc / strace / perf / gdb / bpftrace / ip-ss / tcpdump / epoll） |
| `independence` | 单次调用 setup→observe→cleanup；不留下 netns、临时文件、后台进程 |
| `ub_tool` | 一般为 `n/a` |

载体：`examples/cNN_*.rs` + `tests/cNN_*.rs`，CLI 类可附加 `scripts/cNN_*.sh`。

Skip 规则：**仅** `LF_HOST_KIND` ∈ {wsl2, docker, other} 时允许 skip + 等价观察。
基准 VM 上 MUST NOT skip。MUST NOT 用 `#[ignore]` 把能力移出 SC-002 分母。

---

## 4. SourceReference

| Field | Rule |
|-------|------|
| `kernel_series` | `v` + VM `uname -r` 的主.次（6.x） |
| `path` | 相对 linux.git |
| `symbol` | 结构体或函数名 |
| `line` | 本地树或 tree-url 核对后的行号；同系列允许漂移 |
| `kind` | `kernel-tree` \| `tree-url` |
| `answers` | “这段源码回答了什么” |

---

## 5. DownstreamLink

每项能力一行。禁止空白，允许“仅支撑排障”。

---

## 6. Quiz / AnswerKey

每模块 `quiz.md`（Learner）与 `quiz-answers.md`（Answer Track）题量 ≥5。
通过线：错 ≤1 且每题有推导。

---

## 7. ToolProbe

`acceptance/002-linux-foundation/tool-probe.md` 记录：

```text
host_kind: ubuntu-vm | wsl2 | docker | other
uname_r: ...
kernel_series: vX.Y
privilege: root | user
have: strace gdb perf bpftrace ip ss lsns tcpdump gcc|clang
```

基准 VM 上任一 `have=no` → 环境未就绪。

---

## 8. EnvironmentRecord

001 字段 + `kernel` + `kernel_series` + `tools` + `host_kind` + `privilege_notes`。

---

## 9. AcceptanceCriterion

物理文件 `criteria/cNN.md`。至少一条判据由命令退出码决定。
root 类能力的命令 MUST 在文档中标明 `sudo` 或“以 root 执行”。
禁用措辞与 001 相同。

C-26 的判据 MUST 包含：创建两个 namespace、隔离观察、删除成功。
C-18 的判据 MUST 提及 epoll，MUST NOT 提及 io_uring 作为通过条件。
