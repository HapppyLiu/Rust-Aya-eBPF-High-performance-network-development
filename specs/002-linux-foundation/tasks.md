---
description: "Task list for Linux Foundation (002-linux-foundation)"
---

# Tasks: Linux Foundation

**Input**: Design documents from `/specs/002-linux-foundation/`

**Prerequisites**: [plan.md](./plan.md)、[spec.md](./spec.md)、[research.md](./research.md)、
[data-model.md](./data-model.md)、[contracts/](../../contracts/002-linux-foundation/)、
[quickstart.md](./quickstart.md)

**Tests**: 每个能力的 `tests/cNN_*.rs`（或 `scripts/cNN_*.sh` 退出码）是 FR-003 的**验收载体**，
不是可选开发者测试。与 `examples/cNN_*.rs`（可观察、NON-ASSERTION）成对存在。
实验必须小、独立、可运行、可观察（R-15）：一次 setup→观察→cleanup，失败能归因到单一 C-ID。

**Organization**: 任务按 **Story = 学习模块（m1…m7）** 组织。每个 Story 一个 Phase，
`./tools/lf-verify-module.sh mN-<name>` 可独立验证，不依赖其它模块的运行时状态。

## Format: `[ID] [P?] [Story] Description`

- **[P]**: 可并行（不同文件、无未完成依赖）
- **[Story]**: US1…US7；Setup / Foundational / Polish 无 Story 标签
- 每个任务都包含确切文件路径

## Path Conventions

| 类型 | 本 Feature 路径 |
|------|----------------|
| Spec / Plan / Tasks | `specs/002-linux-foundation/` |
| Learner Track | `learner/002-linux-foundation/` |
| Answer Track（概念） | `learning/002-linux-foundation/` |
| Feynman 参考回答 | `feynman/002-linux-foundation/` |
| 实验 | `experiments/002-linux-foundation/` |
| 验收 | `acceptance/002-linux-foundation/` |
| 契约 | `contracts/002-linux-foundation/` |
| harness | `linux-harness/` |

文档中的 `mN-<module>` 指各类型目录下本 Feature 子目录中的模块。

## 双轨产物（每个产出学习内容的 Task 同时写两轨）

```text
Task → Learner Track（先） + Answer Track（后） → Acceptance
```

| 轨道 | 目录 | 内容 |
|------|------|------|
| Learner | `learner/002-linux-foundation/mN-*/` | `guide.md` `predictions.md` `selfcheck.md` `quiz.md` |
| Answer | `learning/…` `feynman/…` `experiments/…` 源码 | 机制、行号、答案、实现 |
| 汇合 | `acceptance/002-linux-foundation/` | AC 与矩阵 |

`learner/` MUST NOT 泄漏稳定断言预期值、内核行号、机制性结论、Quiz/Feynman 答案（契约 §G L1–L7）。
基础设施 Task（harness、脚本）不走双轨。

## 学习顺序即执行顺序

spec.md：Story 编号是学习顺序，P1/P2/P3 是阻塞性不是开工顺序。

- Phase 顺序 = **US1 → US2 → US3 → US4 → US5 → US6 → US7**（FR-011，禁止跳过）
- **Story 之间 MUST NOT 并行**（m2 学完再学 m3）；**验证**仍按模块独立（R-15）
- 并行只在**单个 Story 内部**（不同 `cNN` 文件）
- US1/US2/US3/US6 是进入 003 的硬前置；`可启动 003` ⟺ US1…US7 全部 complete（FR-012）

最终 `accepted` 只在 Ubuntu x86_64 虚拟机上判定。C-04 / C-06 / C-26 必须 root。

---

## Phase 1: Setup（共享基础设施）

**Purpose**: 把 002 接入已有 workspace，建立双轨目录与 VM 探测脚本。不改 001 的 pin 工具链。

- [ ] T001 Add `linux-harness` and `experiments/002-linux-foundation/m*` to workspace `members` and `lf-harness` to `[workspace.dependencies]` in `Cargo.toml`
- [ ] T002 Create `linux-harness/Cargo.toml` and `linux-harness/src/lib.rs` (package `lf-harness`, zero extra crates, declare `host` `tool_probe` `strace` `proc` `env` modules per `contracts/002-linux-foundation/harness-api.md`)
- [ ] T003 [P] Create Feature directory READMEs in `learner/002-linux-foundation/README.md`, `learning/002-linux-foundation/README.md`, `feynman/002-linux-foundation/README.md`, `experiments/002-linux-foundation/README.md`, and `acceptance/002-linux-foundation/README.md`
- [ ] T004 [P] Create `tools/linux-tool-probe.sh` writing `host_kind`, `uname_r`, `kernel_series`, `privilege`, and have-bits for strace/gdb/perf/bpftrace/ip/ss/lsns/tcpdump into stdout (quickstart §0)
- [ ] T005 [P] Create `tools/lf-run.sh` wrapping command + env-record; honor `# PRIV: root` via sudo when not already root in `tools/lf-run.sh`
- [ ] T006 Create `tools/lf-verify-module.sh` that builds only `lf-harness` plus one `mN-*` crate, runs `cargo test -p <crate>`, checks that module's seven artifact files exist per `contracts/002-linux-foundation/learning-artifact-contract.md` §H
- [ ] T007 [P] Create Answer Track templates `learning/002-linux-foundation/_templates/concept.md` and `learning/002-linux-foundation/_templates/source-refs.md`
- [ ] T008 [P] Create Learner Track templates `learner/002-linux-foundation/_templates/guide.md`, `predictions.md`, `selfcheck.md`, and `quiz.md` (no answers)
- [ ] T009 [P] Create `feynman/002-linux-foundation/_template.md`, `experiments/002-linux-foundation/_templates/OBSERVATIONS.md`, and `acceptance/002-linux-foundation/criteria/_template.md`

**Checkpoint**: 目录与脚本就位，001 toolchain 未改。

---

## Phase 2: Foundational（阻塞所有 Story）

**Purpose**: `lf-harness` 可测、矩阵为空表、模板可用。本阶段完成前不得开始模块实验。

**⚠️ CRITICAL**: No user story work until this phase is complete.

- [ ] T010 Implement `linux-harness/src/host.rs` (`HostKind`, `host_kind()`, `is_acceptance_host()` reading `LF_HOST_KIND`)
- [ ] T011 [P] Implement `linux-harness/src/tool_probe.rs` (`have`, `path`, `require` — acceptance host missing tool panics)
- [ ] T012 [P] Implement `linux-harness/src/strace.rs` (`run`, `syscall_names` — no PID parsing)
- [ ] T013 [P] Implement `linux-harness/src/proc.rs` named fields from `/proc/self/status|maps|stat`
- [ ] T014 [P] Implement `linux-harness/src/env.rs` markdown matching `tools/env-record.sh` plus Linux fields in `linux-harness/src/env.rs`
- [ ] T015 Write `linux-harness/tests/harness_selfcheck.rs` covering probe, syscall_names on a fixture string, and skipped-host behavior
- [ ] T016 Create `acceptance/002-linux-foundation/capability-matrix.md` with 27 rows (C-01…C-27) columns from learning-artifact-contract §E, `Status = planned`
- [ ] T017 Run `cargo test -p lf-harness -- --test-threads=1` until green and archive `./tools/linux-tool-probe.sh` output as `acceptance/002-linux-foundation/tool-probe.md` plus `acceptance/002-linux-foundation/environment-baseline.md`

**Checkpoint**: harness 可用、追踪表就位 —— US1 可以开始。

---

## Phase 3: User Story 1 - CLI、诊断与观测工具 (Priority: P1) 🎯 MVP

**Goal**: 命令行诊断进程/文件/套接字；先预测再核对本机系统调用与观测工具。覆盖 C-01…C-06。

**Independent Test**: `./tools/lf-verify-module.sh m1-cli-observe` 退出码 0。在 Ubuntu VM 上 C-04/C-06 以 root 跑通；Learner 四文件不含答案。

### Learner Track first（无答案）

- [ ] T018 [US1] Create crate `experiments/002-linux-foundation/m1-cli-observe/Cargo.toml` and `src/lib.rs` (dev-dep `lf-harness`; declare `c01`…`c06` placeholders)
- [ ] T019 [US1] Write Learning Framework `learner/002-linux-foundation/m1-cli-observe/guide.md`
- [ ] T020 [P] [US1] Write Experiment Tasks `learner/002-linux-foundation/m1-cli-observe/predictions.md`
- [ ] T021 [P] [US1] Write Feynman Questions `learner/002-linux-foundation/m1-cli-observe/selfcheck.md`
- [ ] T022 [P] [US1] Write Quiz `learner/002-linux-foundation/m1-cli-observe/quiz.md` (≥5 questions, no answers)

### Tests first（必须先红）

- [ ] T023 [P] [US1] Write failing test `experiments/002-linux-foundation/m1-cli-observe/tests/c01_cli.rs` (`CLAIM`: can parse `/proc/self` identity fields)
- [ ] T024 [P] [US1] Write failing test `experiments/002-linux-foundation/m1-cli-observe/tests/c02_diag.rs` (`CLAIM`: `ss`/`ip` observe local sockets)
- [ ] T025 [P] [US1] Write failing test `experiments/002-linux-foundation/m1-cli-observe/tests/c03_strace.rs` (`CLAIM`: syscall name set contains `write`)
- [ ] T026 [P] [US1] Write failing test `experiments/002-linux-foundation/m1-cli-observe/tests/c04_perf.rs` plus `scripts/c04_perf.sh` (`# PRIV: root`; `CLAIM`: `perf stat` exit 0)
- [ ] T027 [P] [US1] Write failing test `experiments/002-linux-foundation/m1-cli-observe/tests/c05_gdb.rs` (`CLAIM`: `gdb -batch` breaks on a user symbol)
- [ ] T028 [P] [US1] Write failing test `experiments/002-linux-foundation/m1-cli-observe/tests/c06_bpftrace.rs` plus `scripts/c06_bpftrace.sh` (`# PRIV: root`; one-liner `BEGIN`/`exit` only)

### Track B examples + Track A teaching

- [ ] T029 [P] [US1] Implement example `experiments/002-linux-foundation/m1-cli-observe/examples/c01_cli.rs` (≤80 lines, `/proc` observe)
- [ ] T030 [P] [US1] Implement example `experiments/002-linux-foundation/m1-cli-observe/examples/c02_diag.rs`
- [ ] T031 [P] [US1] Implement example `experiments/002-linux-foundation/m1-cli-observe/examples/c03_strace.rs`
- [ ] T032 [P] [US1] Implement example `experiments/002-linux-foundation/m1-cli-observe/examples/c04_perf.rs`
- [ ] T033 [P] [US1] Implement example `experiments/002-linux-foundation/m1-cli-observe/examples/c05_gdb.rs`
- [ ] T034 [P] [US1] Implement example `experiments/002-linux-foundation/m1-cli-observe/examples/c06_bpftrace.rs` (no `.bpf.c`)
- [ ] T035 [US1] Write Teaching Material `learning/002-linux-foundation/m1-cli-observe/concept.md` (C-01…C-06 + FR-014 + depth notes)
- [ ] T036 [P] [US1] Write `learning/002-linux-foundation/m1-cli-observe/source-refs.md` (ptrace / proc / perf_event / bpf_trace symbols; `KERNEL_SERIES` from VM)
- [ ] T037 [US1] Fill `experiments/002-linux-foundation/m1-cli-observe/OBSERVATIONS.md` (env block + why / cannot-prove)
- [ ] T038 [US1] Write AC `acceptance/002-linux-foundation/criteria/c01.md` through `c06.md` (exit-code criteria; c04/c06 mark root)
- [ ] T039 [US1] Write Answer Key `learning/002-linux-foundation/m1-cli-observe/quiz-answers.md` and Feynman answers `feynman/002-linux-foundation/m1-cli-observe.md` (five sections, cover C-01…C-06)
- [ ] T040 [US1] Run `./tools/lf-verify-module.sh m1-cli-observe` and set C-01…C-06 `Status` in `acceptance/002-linux-foundation/capability-matrix.md`

**Checkpoint**: m1 独立通过 —— MVP。勿开始 US2 直到本 checkpoint 完成。

---

## Phase 4: User Story 2 - 用户态/内核态、进程模型与系统调用路径 (Priority: P1)

**Goal**: 说明用户/内核边界、`task_struct`、上下文切换、syscall 进入路径。覆盖 C-07…C-10。

**Independent Test**: `./tools/lf-verify-module.sh m2-process-syscall` 退出码 0。

- [ ] T041 [US2] Create crate `experiments/002-linux-foundation/m2-process-syscall/Cargo.toml` and `src/lib.rs`
- [ ] T042 [US2] Write `learner/002-linux-foundation/m2-process-syscall/guide.md`
- [ ] T043 [P] [US2] Write `learner/002-linux-foundation/m2-process-syscall/predictions.md`
- [ ] T044 [P] [US2] Write `learner/002-linux-foundation/m2-process-syscall/selfcheck.md`
- [ ] T045 [P] [US2] Write `learner/002-linux-foundation/m2-process-syscall/quiz.md`
- [ ] T046 [P] [US2] Write failing test `experiments/002-linux-foundation/m2-process-syscall/tests/c07_user_kernel.rs`
- [ ] T047 [P] [US2] Write failing test `experiments/002-linux-foundation/m2-process-syscall/tests/c08_task.rs`
- [ ] T048 [P] [US2] Write failing test `experiments/002-linux-foundation/m2-process-syscall/tests/c09_ctxsw.rs`
- [ ] T049 [P] [US2] Write failing test `experiments/002-linux-foundation/m2-process-syscall/tests/c10_syscall.rs`
- [ ] T050 [P] [US2] Implement `experiments/002-linux-foundation/m2-process-syscall/examples/c07_user_kernel.rs`
- [ ] T051 [P] [US2] Implement `experiments/002-linux-foundation/m2-process-syscall/examples/c08_task.rs`
- [ ] T052 [P] [US2] Implement `experiments/002-linux-foundation/m2-process-syscall/examples/c09_ctxsw.rs`
- [ ] T053 [P] [US2] Implement `experiments/002-linux-foundation/m2-process-syscall/examples/c10_syscall.rs` (optional C contrast only as extra file, not required)
- [ ] T054 [US2] Write `learning/002-linux-foundation/m2-process-syscall/concept.md` and `source-refs.md` (`do_syscall_64`, `task_struct`, `context_switch`)
- [ ] T055 [US2] Fill `experiments/002-linux-foundation/m2-process-syscall/OBSERVATIONS.md`
- [ ] T056 [US2] Write `acceptance/002-linux-foundation/criteria/c07.md` through `c10.md`
- [ ] T057 [US2] Write `learning/002-linux-foundation/m2-process-syscall/quiz-answers.md` and `feynman/002-linux-foundation/m2-process-syscall.md`
- [ ] T058 [US2] Run `./tools/lf-verify-module.sh m2-process-syscall` and update `acceptance/002-linux-foundation/capability-matrix.md` for C-07…C-10

**Checkpoint**: m2 独立通过。

---

## Phase 5: User Story 3 - 虚拟内存、页表、缺页与 mmap (Priority: P1)

**Goal**: 虚存 ≠ 物理页；匿名 mmap 触碰前后缺页。覆盖 C-11…C-14。

**Independent Test**: `./tools/lf-verify-module.sh m3-memory` 退出码 0；minflt 方向断言稳定。

- [ ] T059 [US3] Create crate `experiments/002-linux-foundation/m3-memory/Cargo.toml` and `src/lib.rs`
- [ ] T060 [US3] Write `learner/002-linux-foundation/m3-memory/guide.md`
- [ ] T061 [P] [US3] Write `learner/002-linux-foundation/m3-memory/predictions.md`
- [ ] T062 [P] [US3] Write `learner/002-linux-foundation/m3-memory/selfcheck.md`
- [ ] T063 [P] [US3] Write `learner/002-linux-foundation/m3-memory/quiz.md`
- [ ] T064 [P] [US3] Write failing test `experiments/002-linux-foundation/m3-memory/tests/c11_vmem.rs`
- [ ] T065 [P] [US3] Write failing test `experiments/002-linux-foundation/m3-memory/tests/c12_page.rs`
- [ ] T066 [P] [US3] Write failing test `experiments/002-linux-foundation/m3-memory/tests/c13_fault.rs`
- [ ] T067 [P] [US3] Write failing test `experiments/002-linux-foundation/m3-memory/tests/c14_mmap.rs`
- [ ] T068 [P] [US3] Implement `experiments/002-linux-foundation/m3-memory/examples/c11_vmem.rs`
- [ ] T069 [P] [US3] Implement `experiments/002-linux-foundation/m3-memory/examples/c12_page.rs`
- [ ] T070 [P] [US3] Implement `experiments/002-linux-foundation/m3-memory/examples/c13_fault.rs`
- [ ] T071 [P] [US3] Implement `experiments/002-linux-foundation/m3-memory/examples/c14_mmap.rs` (SAFETY comments if `unsafe` slices)
- [ ] T072 [US3] Write `learning/002-linux-foundation/m3-memory/concept.md` and `source-refs.md` (`mm_struct`, `struct page`, `handle_mm_fault`, `do_mmap`)
- [ ] T073 [US3] Fill `experiments/002-linux-foundation/m3-memory/OBSERVATIONS.md`
- [ ] T074 [US3] Write `acceptance/002-linux-foundation/criteria/c11.md` through `c14.md`
- [ ] T075 [US3] Write `learning/002-linux-foundation/m3-memory/quiz-answers.md` and `feynman/002-linux-foundation/m3-memory.md`
- [ ] T076 [US3] Run `./tools/lf-verify-module.sh m3-memory` and update `acceptance/002-linux-foundation/capability-matrix.md` for C-11…C-14

**Checkpoint**: m3 独立通过。

---

## Phase 6: User Story 4 - 文件描述符、VFS 与 I/O 模型 (Priority: P2)

**Goal**: fd / file / inode；open/read/write 路径；阻塞 vs 非阻塞 vs **epoll**（禁止 io_uring）。覆盖 C-15…C-18。

**Independent Test**: `./tools/lf-verify-module.sh m4-vfs-io` 退出码 0。

- [ ] T077 [US4] Create crate `experiments/002-linux-foundation/m4-vfs-io/Cargo.toml` and `src/lib.rs`
- [ ] T078 [US4] Write `learner/002-linux-foundation/m4-vfs-io/guide.md`
- [ ] T079 [P] [US4] Write `learner/002-linux-foundation/m4-vfs-io/predictions.md`
- [ ] T080 [P] [US4] Write `learner/002-linux-foundation/m4-vfs-io/selfcheck.md`
- [ ] T081 [P] [US4] Write `learner/002-linux-foundation/m4-vfs-io/quiz.md`
- [ ] T082 [P] [US4] Write failing test `experiments/002-linux-foundation/m4-vfs-io/tests/c15_fd.rs`
- [ ] T083 [P] [US4] Write failing test `experiments/002-linux-foundation/m4-vfs-io/tests/c16_vfs.rs`
- [ ] T084 [P] [US4] Write failing test `experiments/002-linux-foundation/m4-vfs-io/tests/c17_rw.rs`
- [ ] T085 [P] [US4] Write failing test `experiments/002-linux-foundation/m4-vfs-io/tests/c18_io.rs` (epoll, not io_uring)
- [ ] T086 [P] [US4] Implement `experiments/002-linux-foundation/m4-vfs-io/examples/c15_fd.rs`
- [ ] T087 [P] [US4] Implement `experiments/002-linux-foundation/m4-vfs-io/examples/c16_vfs.rs`
- [ ] T088 [P] [US4] Implement `experiments/002-linux-foundation/m4-vfs-io/examples/c17_rw.rs`
- [ ] T089 [P] [US4] Implement `experiments/002-linux-foundation/m4-vfs-io/examples/c18_io.rs`
- [ ] T090 [US4] Write `learning/002-linux-foundation/m4-vfs-io/concept.md` and `source-refs.md` (`fdtable`, `file`/`inode`, `vfs_read`, `fs/eventpoll.c`)
- [ ] T091 [US4] Fill `experiments/002-linux-foundation/m4-vfs-io/OBSERVATIONS.md`
- [ ] T092 [US4] Write `acceptance/002-linux-foundation/criteria/c15.md` through `c18.md`
- [ ] T093 [US4] Write `learning/002-linux-foundation/m4-vfs-io/quiz-answers.md` and `feynman/002-linux-foundation/m4-vfs-io.md`
- [ ] T094 [US4] Run `./tools/lf-verify-module.sh m4-vfs-io` and update `acceptance/002-linux-foundation/capability-matrix.md` for C-15…C-18

**Checkpoint**: m4 独立通过。

---

## Phase 7: User Story 5 - 调度、IPC 与同步 (Priority: P2)

**Goal**: 运行队列/唤醒/时间预算（禁止调度器公式）；pipe vs 共享内存直觉；futex 竞争路径。覆盖 C-19…C-21。

**Independent Test**: `./tools/lf-verify-module.sh m5-sched-ipc` 退出码 0。

- [ ] T095 [US5] Create crate `experiments/002-linux-foundation/m5-sched-ipc/Cargo.toml` and `src/lib.rs`
- [ ] T096 [US5] Write `learner/002-linux-foundation/m5-sched-ipc/guide.md`
- [ ] T097 [P] [US5] Write `learner/002-linux-foundation/m5-sched-ipc/predictions.md`
- [ ] T098 [P] [US5] Write `learner/002-linux-foundation/m5-sched-ipc/selfcheck.md`
- [ ] T099 [P] [US5] Write `learner/002-linux-foundation/m5-sched-ipc/quiz.md`
- [ ] T100 [P] [US5] Write failing test `experiments/002-linux-foundation/m5-sched-ipc/tests/c19_sched.rs`
- [ ] T101 [P] [US5] Write failing test `experiments/002-linux-foundation/m5-sched-ipc/tests/c20_ipc.rs`
- [ ] T102 [P] [US5] Write failing test `experiments/002-linux-foundation/m5-sched-ipc/tests/c21_sync.rs`
- [ ] T103 [P] [US5] Implement `experiments/002-linux-foundation/m5-sched-ipc/examples/c19_sched.rs`
- [ ] T104 [P] [US5] Implement `experiments/002-linux-foundation/m5-sched-ipc/examples/c20_ipc.rs`
- [ ] T105 [P] [US5] Implement `experiments/002-linux-foundation/m5-sched-ipc/examples/c21_sync.rs`
- [ ] T106 [US5] Write `learning/002-linux-foundation/m5-sched-ipc/concept.md` and `source-refs.md` (`__schedule`, `pipe_read`, `futex`)
- [ ] T107 [US5] Fill `experiments/002-linux-foundation/m5-sched-ipc/OBSERVATIONS.md`
- [ ] T108 [US5] Write `acceptance/002-linux-foundation/criteria/c19.md` through `c21.md`
- [ ] T109 [US5] Write `learning/002-linux-foundation/m5-sched-ipc/quiz-answers.md` and `feynman/002-linux-foundation/m5-sched-ipc.md`
- [ ] T110 [US5] Run `./tools/lf-verify-module.sh m5-sched-ipc` and update `acceptance/002-linux-foundation/capability-matrix.md` for C-19…C-21

**Checkpoint**: m5 独立通过。

---

## Phase 8: User Story 6 - 网络栈、socket、TCP/IP、NIC 与 netns (Priority: P1)

**Goal**: 回环上的发送路径五个命名对象；TCP 连接（禁止拥塞控制）；virtio/NAPI（禁止物理卸载）；**仅 C-26** 创建两个 netns 并删除。覆盖 C-22…C-26。

**Independent Test**: `./tools/lf-verify-module.sh m6-net` 退出码 0。C-26 以 root 在 Ubuntu VM 上创建 `lf-c26-a`/`lf-c26-b` 后清理干净。

- [ ] T111 [US6] Create crate `experiments/002-linux-foundation/m6-net/Cargo.toml` and `src/lib.rs`
- [ ] T112 [US6] Write `learner/002-linux-foundation/m6-net/guide.md`
- [ ] T113 [P] [US6] Write `learner/002-linux-foundation/m6-net/predictions.md`
- [ ] T114 [P] [US6] Write `learner/002-linux-foundation/m6-net/selfcheck.md`
- [ ] T115 [P] [US6] Write `learner/002-linux-foundation/m6-net/quiz.md`
- [ ] T116 [P] [US6] Write failing test `experiments/002-linux-foundation/m6-net/tests/c22_netstack.rs`
- [ ] T117 [P] [US6] Write failing test `experiments/002-linux-foundation/m6-net/tests/c23_socket.rs`
- [ ] T118 [P] [US6] Write failing test `experiments/002-linux-foundation/m6-net/tests/c24_tcp.rs`
- [ ] T119 [P] [US6] Write failing test `experiments/002-linux-foundation/m6-net/tests/c25_nic.rs`
- [ ] T120 [P] [US6] Write failing test `experiments/002-linux-foundation/m6-net/tests/c26_netns.rs` plus `scripts/c26_netns.sh` (`# PRIV: root`; create two ns, isolate, delete)
- [ ] T121 [P] [US6] Implement `experiments/002-linux-foundation/m6-net/examples/c22_netstack.rs` (loopback only)
- [ ] T122 [P] [US6] Implement `experiments/002-linux-foundation/m6-net/examples/c23_socket.rs`
- [ ] T123 [P] [US6] Implement `experiments/002-linux-foundation/m6-net/examples/c24_tcp.rs`
- [ ] T124 [P] [US6] Implement `experiments/002-linux-foundation/m6-net/examples/c25_nic.rs` (virtio/sysfs, no offload)
- [ ] T125 [P] [US6] Implement `experiments/002-linux-foundation/m6-net/examples/c26_netns.rs` (self-contained cleanup)
- [ ] T126 [US6] Write `learning/002-linux-foundation/m6-net/concept.md` and `source-refs.md` (`sk_buff`, `__sys_socket`, `tcp_sendmsg`, `net_device`/NAPI, `struct net`)
- [ ] T127 [US6] Fill `experiments/002-linux-foundation/m6-net/OBSERVATIONS.md`
- [ ] T128 [US6] Write `acceptance/002-linux-foundation/criteria/c22.md` through `c26.md` (c26 requires create+isolate+delete)
- [ ] T129 [US6] Write `learning/002-linux-foundation/m6-net/quiz-answers.md` and `feynman/002-linux-foundation/m6-net.md`
- [ ] T130 [US6] Run `./tools/lf-verify-module.sh m6-net` and update `acceptance/002-linux-foundation/capability-matrix.md` for C-22…C-26

**Checkpoint**: m6 独立通过。C-22…C-25 不依赖残留 netns。

---

## Phase 9: User Story 7 - 综合实验：Rust 验证 Linux 关键机制 (Priority: P3)

**Goal**: 单一自包含 Rust 程序覆盖映射 + fd + 本地 TCP；对照 27 项能力定位表。覆盖 C-27。

**Independent Test**: `./tools/lf-verify-module.sh m7-capstone` 退出码 0；capstone 自己 setup/cleanup，不要求 m1–m6 example 仍在跑。

- [ ] T131 [US7] Create crate `experiments/002-linux-foundation/m7-capstone/Cargo.toml` and `src/lib.rs`
- [ ] T132 [US7] Write `learner/002-linux-foundation/m7-capstone/guide.md`
- [ ] T133 [P] [US7] Write `learner/002-linux-foundation/m7-capstone/predictions.md`
- [ ] T134 [P] [US7] Write `learner/002-linux-foundation/m7-capstone/selfcheck.md`
- [ ] T135 [P] [US7] Write `learner/002-linux-foundation/m7-capstone/quiz.md`
- [ ] T136 [US7] Write failing tests `experiments/002-linux-foundation/m7-capstone/tests/c27_capstone.rs` and `tests/capstone.rs`
- [ ] T137 [US7] Implement `experiments/002-linux-foundation/m7-capstone/examples/capstone.rs` and supporting `src/*.rs` (self-contained)
- [ ] T138 [US7] Write `learning/002-linux-foundation/m7-capstone/concept.md` and `source-refs.md` (points at prior symbols, no new eBPF)
- [ ] T139 [US7] Write `acceptance/002-linux-foundation/capability-location-map.md` locating all 27 capabilities in capstone or module experiments
- [ ] T140 [US7] Fill `experiments/002-linux-foundation/m7-capstone/OBSERVATIONS.md`
- [ ] T141 [US7] Write `acceptance/002-linux-foundation/criteria/c27.md`
- [ ] T142 [US7] Write `learning/002-linux-foundation/m7-capstone/quiz-answers.md` and `feynman/002-linux-foundation/m7-capstone.md`
- [ ] T143 [US7] Run `./tools/lf-verify-module.sh m7-capstone` and set C-27 `accepted` in `acceptance/002-linux-foundation/capability-matrix.md`

**Checkpoint**: US7 完成；003 准入检查的 Build 环闭合。

---

## Phase 10: Polish & Cross-Cutting

**Purpose**: 泄漏检查、追溯、quickstart、禁止 eBPF 产物。

- [ ] T144 [P] Scan `learner/002-linux-foundation/` for L1–L7 leaks (expected assertion values, kernel line numbers, quiz answers) and fix in those learner files
- [ ] T145 [P] Run learning-artifact-contract §E isolated-notes enum into `acceptance/002-linux-foundation/traceability-audit.md` (`comm` both sides empty)
- [ ] T146 Confirm zero `*.bpf.c` / Aya / XDP sources under `experiments/002-linux-foundation/` (SC-010)
- [ ] T147 Execute `specs/002-linux-foundation/quickstart.md` sections 0–6 on the Ubuntu VM and attach outputs under `acceptance/002-linux-foundation/`
- [ ] T148 Run `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` including new 002 crates
- [ ] T149 Sync contract copies `contracts/002-linux-foundation/*.md` → `specs/002-linux-foundation/contracts/*.md` if the root copies changed
- [ ] T150 Mark remaining `planned` rows in `acceptance/002-linux-foundation/capability-matrix.md` only if truly incomplete; otherwise all C-01…C-27 `accepted` on Ubuntu VM

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: 无依赖
- **Foundational (Phase 2)**: 依赖 Setup；**阻塞全部 Story**
- **US1…US7 (Phases 3–9)**: 依赖 Foundational；**学习上串行** US1→US7；验证命令按模块独立
- **Polish (Phase 10)**: 依赖打算交付的全部 Story

### User Story Dependencies

- **US1 (P1)**: Foundational 之后即可；MVP
- **US2 (P1)**: 学完 US1 再开始（FR-011）
- **US3 (P1)**: 学完 US2
- **US4 (P2)**: 学完 US3
- **US5 (P2)**: 学完 US4
- **US6 (P1)**: 学完 US5（P1 表示阻塞 003，不表示可插到 US2 之前）
- **US7 (P3)**: 学完 US1–US6；capstone 运行时不依赖其它 crate 的残留状态

### Within Each User Story

1. Crate + Learner 四件套（无答案）
2. 先写失败的 `tests/cNN_*.rs`（及 root `scripts/`）
3. 再写 `examples/cNN_*.rs`（≤80 行，自清理）
4. Teaching Material + source-refs + OBSERVATIONS
5. AC + Quiz answers + Feynman answers
6. `lf-verify-module.sh` + 矩阵

### Parallel Opportunities

- Phase 1 中标 [P] 的模板/脚本
- Phase 2 中 harness 各模块文件
- **同一 Story 内**不同 `tests/cNN_*.rs` / `examples/cNN_*.rs`
- Polish 的泄漏扫描与 eBPF 零文件检查
- **禁止**跨 Story 并行学习实施

---

## Parallel Example: User Story 1

```bash
# After T018–T022 (crate + learner files):
# failing tests in parallel
Task: tests/c01_cli.rs
Task: tests/c02_diag.rs
Task: tests/c03_strace.rs
Task: tests/c04_perf.rs
Task: tests/c05_gdb.rs
Task: tests/c06_bpftrace.rs

# then examples in parallel
Task: examples/c01_cli.rs
Task: examples/c02_diag.rs
Task: examples/c03_strace.rs
Task: examples/c04_perf.rs
Task: examples/c05_gdb.rs
Task: examples/c06_bpftrace.rs
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 Setup
2. Phase 2 Foundational
3. Phase 3 US1（m1-cli-observe）
4. **STOP**：`./tools/lf-verify-module.sh m1-cli-observe`
5. 再进入 US2

### Incremental Delivery

每完成一个模块就跑该模块的 `lf-verify-module.sh`。不要等 US7 才第一次验证 m1。

### Dual-Track implement rule

`/speckit-implement` 每个学习 Task 必须同时落地 Learner 文件与对应 Answer/实验，且 **Learner 不出现答案**。

---

## Notes

- [P] = 不同文件且无未完成依赖
- 每个实验：一个观察通道、自带 cleanup、example ≤ 80 行
- C-04 / C-06 / C-26：`# PRIV: root`，Ubuntu VM 上不得 skip-as-pass
- C-18 禁止 io_uring；C-19 禁止调度器公式；C-24 禁止拥塞控制；C-25 禁止物理卸载
- C-26 必须真实创建并删除两个 netns；C-22…C-25 只用回环
- 不要在 `learner/` 写内核行号或断言期望值
