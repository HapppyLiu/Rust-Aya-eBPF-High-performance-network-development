# Phase 0 Research: Linux Foundation

**Feature**: 002-linux-foundation | **Date**: 2026-09-17 | **Spec**: [spec.md](./spec.md)

本文件裁定 Technical Context 中的全部实现选择。规格把安装包名、源码获取脚本、crate
推迟到 Plan；Clarifications Session 2026-09-16 已锁定基准宿主与工具清单。此处给出
唯一选择、理由与被拒绝的替代。

---

## R-01 沿用 001 的 pinned Rust toolchain

**Decision**: Rust **1.98.0 stable**（与仓库 `rust-toolchain.toml` 一致），Edition 2024。
本 Feature MUST NOT 执行 `rustup update`。这满足规格“Rust stable”与 FR-020：验收用
已 pin 的 stable，不是每次最新。

**Rationale**: Constitution X 与 001 验收记录绑定同一工具链。Linux 实验的学习标的是
内核路径，不是编译器版本。

**Alternatives considered**: 升级到更新 stable —— 拒绝，会迫使 001 验收记录重跑。

---

## R-02 系统调用载体：`std` + `libc`，默认不引入 `nix`

**Decision**: US1–US7 的 Rust 实验以 `std::os::unix` / `std::process` / `std::fs` /
`std::net` 为第一选择；仅当 std 无法表达所需 syscall（`mmap` 标志、`mincore`、
`epoll`、原始 `socket`）时使用 `libc`。MUST NOT 引入 `nix`、`tokio`、`socket2`、
`aya`、`libbpf`。

**Rationale**: 001 m6 已用 `libc`。再引入 `nix` 会把“Linux 机制”掩盖成 crate API，
违反 Constitution I。

**Alternatives considered**: 每个模块强制一份 C —— 拒绝，FR-025。全部用 C —— 拒绝，
FR-015 要求 US7 为 Rust。

---

## R-03 内核源码绑定虚拟机运行内核的主.次版本

**Decision**: `KERNEL_SERIES` 取 Ubuntu 虚拟机 `uname -r` 的主.次版本（必须是 6.x），
例如 `6.8.0-xx-generic` → `v6.8`。源码树 MUST 与该主.次同系列。引用格式：

```text
KERNEL_SERIES=v<major>.<minor>    # 来自验收 VM 的 uname -r
path/relative/to/linux.git  SYMBOL
```

获取方式（安装脚本属 Plan，包名如下）：优先本地 `$KERNEL_SRC`；否则
`git clone --depth 1 --branch v<major>.<minor> git://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git`
或 Bootlin / git.kernel.org `tree-url`，`kind` 列标 `tree-url`。
允许同系列内行号漂移；符号找不到视为引用失败。

**Rationale**: 规格 Clarification：不要把 6.1 与 6.8 当成可互换的“都是 6.x”。
验收宿主是 Ubuntu VM，不是当前 WSL2 的 6.6 微软内核。

**Alternatives considered**: 固定 v6.6 —— 拒绝，可能与 Ubuntu 24.04 默认 6.8 打架。
跟踪 Linus master —— 拒绝，行号不稳。

---

## R-04 基准 VM 上观测工具全部为必需；偏离宿主才允许等价观察

**Decision**:

| 工具 | 基准 Ubuntu VM | 偏离宿主（WSL2 / Docker） |
|------|----------------|--------------------------|
| strace, gdb, perf, bpftrace, ip, ss, tcpdump, lsns | MUST 存在；缺失 = 环境未就绪，不得 `accepted` | 可记探测失败；`experiment-passed` 可用 `/proc` 等等价路径；`accepted` 仍要回 VM 补跑 |
| C-04 / C-06 / C-26 | MUST 以 root 或等价能力跑通 | 普通用户失败不得冒充掌握 |

探测脚本：`tools/linux-tool-probe.sh`，输出写入
`acceptance/002-linux-foundation/tool-probe.md`，并记录 `host_kind=ubuntu-vm|wsl2|docker|other`
与 `privilege=root|user`。

**Rationale**: 规格 FR-022 / FR-023 / SC-002。旧稿把 perf/gdb/bpftrace 当成 WSL2 上的
可选项，与澄清后的基准冲突，本决策废止该分层。

**Alternatives considered**: 继续 WSL2 回退 —— 拒绝，澄清已选 Ubuntu VM。
强制在 WSL2 安装 bpftrace —— 拒绝，WSL2 不是验收宿主。

---

## R-05 稳定断言与非断言输出物理隔离

**Decision**: 沿用 001 experiment-contract：稳定断言只出现在 `tests/` 或以退出码判定的
`scripts/`。Linux 特有**禁止断言**：PID/TID 具体值、时间戳、绝对地址、perf event count
绝对值、ss Recv-Q、网卡 packet 计数绝对值、strace 全文逐字节比对。

**允许的关系性质**：缺页计数在触碰后**增加**、fd 关闭后路径**不存在**、strace **出现**
名为 `write` 的调用、loopback **UP**、`ip netns list` **含有**本实验创建的名字（实验结束删除）。

**Rationale**: FR-003。

---

## R-06 `linux-harness`：工具探测、strace 解析、环境记录

**Decision**: crate `linux-harness`（package `lf-harness`），零外部依赖。可依赖
workspace 内 `rf-harness` 的 env 格式。职责见
[harness-api.md](../../contracts/002-linux-foundation/harness-api.md)。

MUST NOT 把学习目标代码放进 harness。`tool_probe` 在基准 VM 上发现缺失时 MUST 让测试
**失败**（环境未就绪），MUST NOT 返回 skip-as-pass。仅当 `LF_HOST_KIND` 显式为
`wsl2`/`docker` 时才允许 skip + 等价观察。

---

## R-07 不产生性能主张

**Decision**: 不设 throughput / pps / latency 目标。调度与网络讨论停留在机制。
任何“这样更快”MUST 标为假设。

---

## R-08 双轨产物：七件套物理分离

**Decision**: 001 的 Learner/Answer 目录隔离全部保留。每模块七类产物映射：

| FR-002 产物 | Track | 路径 |
|-------------|-------|------|
| 1 Learning Framework | Learner | `learner/002-linux-foundation/mN-*/guide.md` |
| 4 Experiment Tasks | Learner | `learner/.../predictions.md` |
| 3 Feynman Questions | Learner | `learner/.../selfcheck.md` |
| 5 Quiz | Learner | `learner/.../quiz.md` |
| 2 Teaching Material | Answer | `learning/002-linux-foundation/mN-*/concept.md` + `source-refs.md` |
| 6 Answer Key | Answer | `feynman/002-linux-foundation/mN-*.md` + `learning/.../quiz-answers.md` |
| 7 Acceptance Criteria | 汇合 | `acceptance/002-linux-foundation/criteria/cNN.md` |

Learner Track 禁止：稳定断言预期值、内核行号、机制性结论、Quiz/Feynman 答案、eBPF 程序写法。

**Rationale**: FR-002 / FR-008。

---

## R-09 目录结构：类型 × Feature（与 001 相同）

**Decision**: `learner/` `learning/` `feynman/` `experiments/` `acceptance/` `contracts/`
下各建 `002-linux-foundation/`。`specs/002-linux-foundation/` 只放规格。
根 workspace 增加 `linux-harness` 与 `experiments/002-linux-foundation/m*`。

---

## R-10 bpftrace 只允许观察型 one-liner

**Decision**: C-06 最大程序是单行 `bpftrace -e '...'`（root）。MUST NOT 编写 `.bpf.c`、
MUST NOT 使用 Aya、MUST NOT 加载自定义 BPF 对象。

---

## R-11 验收宿主是 Ubuntu 虚拟机；WSL2/Docker 是偏离

**Decision**: 最终 `accepted` 只在带完整 6.x 内核的 Ubuntu x86_64 **虚拟机**上判定。
当前开发机若为 WSL2，只可用于起草 Markdown 与编译检查；C-04、C-06、C-26 以及任何
需要完整内核观测的条目 MUST 在 VM 上复现。Docker MUST NOT 作为验收宿主。

C-25 使用虚拟机里的 virtio（或等价虚拟 NIC）+ NAPI 源码，MUST NOT 要求物理网卡卸载。
C-22…C-24 使用回环即可。

**Rationale**: 规格宿主选择 B；深度上限 FR-024。

**Alternatives considered**: 把当前 WSL2 当验收环境 —— 拒绝。等物理机 —— 拒绝，会阻塞学习。

---

## R-12 提权与 network namespace

**Decision**:

- C-04 `perf`、C-06 `bpftrace`、C-26 `ip netns`：Ubuntu VM 上 MUST 以 root 或
  `CAP_PERFMON`/`CAP_SYS_ADMIN`/`CAP_NET_ADMIN` 跑通。
- C-26 MUST 创建至少两个 namespace（例如 `lf-a` / `lf-b`），在一方配置地址，证明
  另一方不可见，然后 **删除** namespace。不得把“`lsns` 看到了 init netns”当作 C-26 通过。
- C-22…C-25 MUST NOT 依赖这两个 namespace。
- 不需要提权的实验（strace、文件 I/O、回环 TCP）MUST 以普通用户执行。

**Rationale**: FR-023 / FR-025。旧稿 R-12 的 lsns 回退仅适用于偏离宿主的
`experiment-passed`，不能把 C-26 标为 `accepted`。

---

## R-13 实验语言：Rust 默认，C 允许但不强制每模块一份

**Decision**: 每个能力至少一个 Rust example/test **或**一个可断言的 shell 检查
（纯 CLI 能力）。C 程序 MAY 用于 libc 符号与内核入口对照（建议放在 m2 `c10` 旁，
不是硬性模块门禁）。US7 必须是 Rust。gcc 或 clang 在 VM 上 MUST 可用。

---

## R-14 基准架构与系统调用入口

**Decision**: 基准架构 **x86_64**。C-07/C-10 以 `arch/x86/entry/common.c` 的
`do_syscall_64` 为第一引用。VDSO 只作为“有些调用可能不陷入”的对照。

---

## R-15 模块独立可验证；实验小、独立、可运行、可观察

**Decision**: 落实 instructions-002.md「每一个核心知识模块都应该能够被独立验证。
实验应该尽可能小、独立、可运行、可观察。」

**模块独立验证**（学习顺序仍是 US1→US7，验证不共享运行时状态）：

```bash
./tools/lf-verify-module.sh mN-<name>
```

该脚本只构建 `lf-harness` + 该模块 crate，运行该模块 `cargo test` 与该模块
`scripts/cNN_*.sh`，并检查该模块七类产物文件存在。MUST NOT 要求其他模块的
namespace、mmap 区域或后台进程仍在。m7 capstone 可以**引用**前六模块的产物路径，
但 MUST 自己 setup/teardown。

**实验独立**：

- 一项能力 = 一个主实验 slug `cNN_<slug>`（example + test，CLI 可加 script）。
- 单次调用完成 setup → 触发机制 → 观察 → 断言 → cleanup。
- example 目标 ≤ 80 行；每个 test 文件一条主 `CLAIM`。
- 必须有且仅聚焦**一个**主观察通道（`/proc` 字段、strace 集合、perf 退出码、
  gdb batch 符号、bpftrace 一行输出、ip/ss/tcpdump 存在性）。
- MUST NOT 让 C-14 依赖 C-26，MUST NOT 让 C-18 依赖真实网卡。

**可观察**：没有命名观察通道的实验视为未完成（禁止“我看见了”）。

**Rationale**: 用户对本 Plan 的显式约束。大而全的“综合脚本”会让失败无法归因到某一能力。

**Alternatives considered**: 每 Story 一个巨型 example —— 拒绝，失败时无法独立复现。
模块验证必须先跑完 m1–m6 —— 拒绝，违反独立验证。

---

## R-16 Ubuntu VM 软件包（安装脚本，规格未锁定包名）

**Decision**: 验收 VM 使用 Ubuntu，安装：

```text
build-essential clang gdb strace linux-tools-$(uname -r) bpftrace
iproute2 tcpdump linux-headers-$(uname -r)
```

`perf` 来自 `linux-tools-*`。若 `linux-tools-$(uname -r)` 不可用，改用
`linux-tools-generic` 并在环境记录注明。内核源码按 R-03 获取，不把
`apt-get source` 的发行版补丁树当作唯一权威（符号以对应 upstream 系列为准）。

**Alternatives considered**: 在 Plan 里不写包名 —— 会使 VM 准备不可复现。已拒绝。

---

## R-17 I/O 实验用 epoll，不用 io_uring

**Decision**: C-18 的多路复用实现为 `epoll_wait`（`libc` 或等价），源码定位
`fs/eventpoll.c`。MUST NOT 把 `io_uring` 写入 example/test/AC。

---

## 未决项

无。规格 Clarifications 九条与 Session 五问已覆盖宿主、内核系列、root、深度、
工具清单。C 不强制每模块、netns 仅 C-26 强制，由 FR-025 与 R-13/R-12 落实。
