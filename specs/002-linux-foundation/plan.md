# Implementation Plan: Linux Foundation

**Branch**: `002-linux-foundation`（规格目录仍为 `specs/002-linux-foundation/`；本仓库未强制
工作区只在该分支上） | **Date**: 2026-09-17 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-linux-foundation/spec.md`

## Summary

把 27 项系统级 Linux 能力（C-01…C-27）转化为**双轨、可独立验证**的产物：7 个学习模块；
每项能力一个**小而独立**的可观察实验 + 一组稳定断言 + 一条内核源码引用 + 一条
Acceptance Criterion + 一条 eBPF/Aya/高性能网络关联；每模块同时具备 Learner Track
（Framework / Feynman Questions / Experiment Tasks / Quiz）与 Answer Track
（Teaching Material / Answer Key），物理分离。

技术路径的四个支点：
1. **验收宿主是 Ubuntu 虚拟机**（R-11），不是 WSL2 / Docker；
2. **内核源码绑定该 VM `uname -r` 的主.次版本**（R-03）；
3. **基准 VM 上 gdb/strace/perf/bpftrace/iproute2/tcpdump 全部必需**，C-04/C-06/C-26 必须 root（R-04 / R-12）；
4. **模块与实验独立可验证**（R-15）：一次只触发一个机制，失败可归因到单一 C-ID。

本 Feature **MUST NOT** 编写 eBPF/Aya/XDP 程序。`bpftrace` 仅作示波器。实验尽量小、
独立、可运行、可观察。

全部技术决策见 [research.md](./research.md)。

## Technical Context

**Language/Version**: Rust **1.98.0 stable**（`rust-toolchain.toml`），**Edition 2024**。
实验载体为 Rust + Linux 命令；C 允许但不强制每模块一份（R-13）。gcc 或 clang 在验收 VM 上可用。

**Primary Dependencies**: 默认仅 `libc`（与 001 m6 同策略）。workspace 内 `lf-harness`
（`linux-harness/`）零外部依赖。MUST NOT 引入 `nix` / `tokio` / `aya` / `libbpf`。

**Storage**: N/A —— 学习产物为仓库内 Markdown 与源码。

**Testing**: 模块级独立验证 `./tools/lf-verify-module.sh mN-<name>`（R-15）。
Rust 稳定断言：`cargo test -p mN-... -- --test-threads=1`。Shell 级断言由
`tools/lf-run.sh` 封装，提权实验经 `sudo -n` 或显式 root。基准 VM 上工具缺失 = 失败，
不是 skip-pass。偏离宿主（`LF_HOST_KIND=wsl2|docker`）才允许等价观察。

**Target Platform**: **Ubuntu x86_64 虚拟机**，Linux Kernel 6.x，系列 = 该 VM `uname -r`
的主.次版本。WSL2 与 Docker 不是验收宿主。虚拟 NIC（virtio/NAPI）用于 C-25，不要求物理网卡。

**Project Type**: Learning project + executable experiments。产物按**类型目录 × Feature
子目录**分放（与 001 相同）。双轨隔离：Learner Track 无答案。

**Performance Goals**: **不设**网络或调度性能目标（R-07 / FR-017）。

**Constraints**:
- MUST NOT 编写 eBPF / Aya / XDP / TC / AF_XDP 程序（FR-009）。
- MUST NOT 升级 pinned Rust toolchain（FR-020）。
- Learner Track MUST NOT 泄漏答案（FR-008）。
- 学习顺序 US1→US7（FR-011）；**验证**按模块独立（R-15）。
- 稳定断言中 MUST NOT 出现 PID/地址/时间戳/瞬时计数绝对值（R-05）。
- 深度上限：epoll / 运行队列·唤醒 / TCP 连接与数据包对象 / virtio-NAPI（FR-024）。
- C-26 必须真实创建两个 netns；C-22…C-25 用回环，不依赖 netns（FR-025）。

**Scale/Scope**: 7 个学习模块 / 27 项能力 / ≥27 个独立实验（example≤80 行）/
7 份 Feynman / 7 份 Quiz / 27 条 AC / 1 个 Rust 综合实验。

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.0.0。**Plan gate** 要求每个目标有源码定位范围（II）与至少一个可运行实验（III）。

| Principle | 状态 | 本 Plan 中的落点 |
|-----------|------|----------------|
| I. First-Principles Learning | PASS | 每项能力下沉到内核路径或可观测 `/proc`/syscall/工具输出，禁止停在 man page |
| II. Source-Code-First | PASS | Capability Gate Matrix 为 27 项指定内核文件 + 符号；系列绑定 VM `uname -r` |
| III. Experiment-Driven | PASS | 每项 1 个独立小实验 + ≥1 条稳定断言 + 命名观察通道（R-15） |
| IV. Feynman Explanation | PASS | 7 份提问版 + 7 份参考回答，五项合取 |
| V. Acceptance-Criteria-Driven | PASS | 27 条 `criteria/cNN.md`；模块可用 `lf-verify-module.sh` 单独验收 |
| VI. Unsafe-Rust-Safety | PASS（沿用） | 不以 unsafe 为标的；若 `mmap` 切片等出现 `unsafe`，MUST 写 SAFETY 并覆盖 001 五要素 |
| VII. no_std-Awareness | N/A（已 justify） | 不编写 eBPF；用户/内核边界由 C-07 承担。见 Complexity Tracking |
| VIII. Linux-Kernel-Awareness | PASS | syscall / mm / vfs / sched / net / skb / NAPI / netns 均覆盖。XDP/TC/BPF **只作下游关联** |
| IX. Performance-Is-Measured | PASS | 不产生需 benchmark 的主张（R-07） |
| X. Reproducibility | PASS | toolchain pin + env-record + VM 工具清单 + KERNEL_SERIES |
| XI. Incremental Complexity | PASS | 学习 US1→US7；002 纠正“跳过 Linux 直接 eBPF” |
| XII. Learn → Explain → Build | PASS | 双轨闭环 + US7 Build |
| XIII. Knowledge Must Be Traceable | PASS | `capability-matrix.md` 为单一事实源 |
| XIV. Final Capability | PASS | 承担 Linux networking 地基与 kernel source reading，不降级为“学会一些命令” |

**Gate 判定**：通过。偏离项见 Complexity Tracking。

### Dual-Track per Module

每个模块 **同时** 走完 Track A 与 Track B 才可 `complete`。模块验证命令只触及该模块文件。

```text
                    Spec (C-IDs in this module)
                              │
                              ▼
                           Module
                              │
                    ┌─────────┴─────────┐
                    ▼                   ▼
              Track A                 Track B
              Knowledge               Experiment
                    │                   │
         concepts / arch /         one small experiment
         kernel source /           per capability:
         Feynman Q / Quiz          setup → run → observe → cleanup
                    │                   │
                    └─────────┬─────────┘
                              ▼
                    Learner views (no answers)
                    then Answer Key
                              ▼
                    lf-verify-module.sh  →  AC
```

| Module | Story | Caps | Track A 焦点 | Track B 主观察通道（每能力一个，互不共享运行时） |
|--------|-------|------|--------------|--------------------------------------------------|
| m1-cli-observe | US1 P1 | C-01…C-06 | CLI 对象从哪类内核结构来 | `/proc`；`ip`/`ss`；strace 名集合；perf 退出码；gdb batch；bpftrace 一行 |
| m2-process-syscall | US2 P1 | C-07…C-10 | 用户/内核边界与 `task_struct` | 函数 vs syscall 对照；Tgid/Pid；自愿切换计数方向；strace `write` |
| m3-memory | US3 P1 | C-11…C-14 | 虚存 ≠ 物理页 | maps；mincore/pagemap 驻留；minflt 增加；mmap 条目出现/消失 |
| m4-vfs-io | US4 P2 | C-15…C-18 | fd / file / inode | `/proc/self/fd`；两 fd 同 inode；write/read 往返；阻塞 vs 非阻塞 vs **epoll** |
| m5-sched-ipc | US5 P2 | C-19…C-21 | 运行队列 / 唤醒 / 预算 | `/proc/self/sched`；pipe 传输；竞争路径出现 `futex` |
| m6-net | US6 P1 | C-22…C-26 | socket→协议→NAPI/virtio | lo 发送；socket/bind；本机 TCP 状态；虚拟 NIC sysfs；**两个 netns**（仅 C-26） |
| m7-capstone | US7 P3 | C-27 | 组合前六模块机制 | 单一 Rust 程序：映射+fd+本地 TCP；自己 setup/cleanup |

### Capability Gate Matrix

Constitution Plan gate 的逐项落实。**每行一个独立实验**；**Priv** 为基准 VM 上的权限；
**Obs** 为命名观察通道。源码系列 = VM `uname -r` 主.次，表中路径相对 linux.git。

| C-ID | Exp | 源码定位范围（II） | Priv | Obs | Downstream（FR-014） |
|------|-----|-------------------|------|-----|---------------------|
| C-01 | `c01_cli` | `fs/proc/base.c` | user | `/proc/self` | 排障 eBPF 加载失败时看进程/权限 |
| C-02 | `c02_diag` | `net/ipv4/inet_diag.c` | user | `ss`/`ip` | socket 诊断；后续 SO_REUSEPORT / map fd |
| C-03 | `c03_strace` | `kernel/ptrace.c` | user | strace 名集合 | 分清 libc 与真正 syscall；对照 bpftrace |
| C-04 | `c04_perf` | `kernel/events/core.c` | **root** | `perf stat` 退出码 0 且有 cycles 行 | 后续 cycles/packet 测量入口 |
| C-05 | `c05_gdb` | `kernel/ptrace.c` | user | `gdb -batch` 能断在用户符号 | 用户态加载器崩溃 vs 内核路径 |
| C-06 | `c06_bpftrace` | `kernel/trace/bpf_trace.c`（只读） | **root** | 一行 `BEGIN{...exit()}` 打印 ok | **观察不是编程**；真正 BPF 在 003+ |
| C-07 | `c07_user_kernel` | `arch/x86/entry/common.c` `do_syscall_64` | user | 普通调用 vs syscall 对照 | 为何 helper 必须 probe_read |
| C-08 | `c08_task` | `include/linux/sched.h` `task_struct` | user | Tgid/Pid | 线程与 BPF 程序上下文不是一回事 |
| C-09 | `c09_ctxsw` | `kernel/sched/core.c` `context_switch` | user | 自愿切换计数增加 | 尾延迟与调度；eBPF 不能随意睡眠 |
| C-10 | `c10_syscall` | `do_syscall_64` + `kernel/sys.c` | user | strace 见 `write` | 后续 `bpf(2)` 也是这条进入路径 |
| C-11 | `c11_vmem` | `include/linux/mm_types.h` `mm_struct` | user | maps 存在 | 用户指针不能在内核直接解引用 |
| C-12 | `c12_page` | `struct page`；`arch/x86/include/asm/pgtable.h` | user | mincore 驻留位 | TLB/缺页成本；零拷贝页钉住 |
| C-13 | `c13_fault` | `mm/memory.c` `handle_mm_fault` | user | minflt 触碰后增加 | umem/mmap 后尚未触碰无物理页 |
| C-14 | `c14_mmap` | `mm/mmap.c` `do_mmap` | user | maps 出现再 munmap 消失 | AF_XDP umem / 共享包缓冲 |
| C-15 | `c15_fd` | `include/linux/fdtable.h` | user | `/proc/self/fd` | bpf map / prog 也是 fd |
| C-16 | `c16_vfs` | `include/linux/fs.h` `file`/`inode` | user | 两 fd 同 inode | 套接字与文件走同一套对象 |
| C-17 | `c17_rw` | `fs/read_write.c` `vfs_read` | user | 写后读回 | 后续 sendmsg 前的 VFS 层 |
| C-18 | `c18_io` | `fs/eventpoll.c`（**epoll**，禁止 io_uring） | user | 未就绪非阻塞立即返回；epoll 等到可读 | 网络程序日常等待模型 |
| C-19 | `c19_sched` | `kernel/sched/core.c` `__schedule` | user | 运行队列/唤醒/预算可陈述 + sched 文件字段存在 | 禁止 EEVDF 公式；尾延迟来源 |
| C-20 | `c20_ipc` | `fs/pipe.c` | user | pipe 跨线程收到字节 | 拷贝次数 vs 共享内存 |
| C-21 | `c21_sync` | `kernel/futex/core.c` | user | 竞争路径 strace 见 `futex` | 用户态锁 vs 内核自旋锁；eBPF 侧限制 |
| C-22 | `c22_netstack` | `include/linux/skbuff.h` `sk_buff`；`net/core/dev.c` | user | lo 发送 + 路径五个命名对象 | XDP 为何想避开 skb |
| C-23 | `c23_socket` | `net/socket.c` `__sys_socket` | user | bind 127.0.0.1 后 ss 可见 | socket filter / 后续 sockmap |
| C-24 | `c24_tcp` | `net/ipv4/tcp.c` `tcp_sendmsg` | user | 本机 connect 进入 ESTABLISHED | **禁止拥塞控制状态机** |
| C-25 | `c25_nic` | `include/linux/netdevice.h`；NAPI in `net/core/dev.c` | user | 虚拟 NIC sysfs + 驱动名（virtio 等） | **禁止物理卸载**；XDP 挂钩点预习 |
| C-26 | `c26_netns` | `include/net/net_namespace.h` `struct net` | **root** | 创建 lf-a/lf-b，地址隔离，然后删除 | 容器网络；tc/xdp 按 ns 挂载 |
| C-27 | `c27_capstone` | 综合引用 m2–m6 已定位符号 | user（子步骤若调 perf/ns 则 root） | 单一 Rust 程序自包含 | 003 准入 |

### Post-Design Re-check（Phase 1 完成后）

Phase 1 产出 data-model.md、contracts/×3、quickstart.md 后重新核对，**结论：仍然通过**。
独立模块验证、Learner 不泄漏、eBPF 零源码、VM+root 门禁、深度上限，均写入契约。
未引入新的原则偏离。

## Project Structure

```text
specs/002-linux-foundation/          # spec / plan / research / data-model / quickstart / tasks
specs/002-linux-foundation/contracts/  # Phase 1 副本（与根目录 contracts 同步）
contracts/002-linux-foundation/      # 权威契约：learning-artifact / experiment / harness-api
learner/002-linux-foundation/        # Track A+B 学习者视图（无答案）
learning/002-linux-foundation/       # Track A 答案：concept / source-refs / quiz-answers
feynman/002-linux-foundation/        # Track A 答案：Feynman 五项
experiments/002-linux-foundation/    # Track B：一模块一 crate，一能力一实验
acceptance/002-linux-foundation/     # matrix + criteria + tool-probe
linux-harness/                       # lf-harness
tools/linux-tool-probe.sh
tools/lf-run.sh
tools/lf-verify-module.sh            # 模块独立验证
```

### Feature artifacts

```text
experiments/002-linux-foundation/
├── m1-cli-observe/          # C-01..C-06  可单独 lf-verify-module.sh
├── m2-process-syscall/      # C-07..C-10
├── m3-memory/               # C-11..C-14
├── m4-vfs-io/               # C-15..C-18
├── m5-sched-ipc/            # C-19..C-21
├── m6-net/                  # C-22..C-26  （C-26 自建自毁 netns）
└── m7-capstone/             # C-27        （自包含，不依赖其它 crate 的运行时状态）

learner/002-linux-foundation/mN-*/{guide,predictions,selfcheck,quiz}.md
learning/002-linux-foundation/mN-*/{concept,source-refs,quiz-answers}.md
feynman/002-linux-foundation/mN-*.md
acceptance/002-linux-foundation/criteria/c01.md … c27.md
```

**Structure Decision**：沿用 001 的**类型目录 × Feature 子目录 + 单 workspace + 双轨隔离**。
crate 粒度 7（对齐模块独立验证与 Feynman）；实验文件粒度 27（对齐能力级 AC 与 R-15）。

根 `Cargo.toml` members 增加 `"linux-harness"` 与 `"experiments/002-linux-foundation/m*"`。
workspace.dependencies 增加 `lf-harness = { path = "linux-harness" }`。

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| **Constitution VII（no_std）本 Feature 不直接实验** | 禁止 eBPF 编程（FR-009）；no_std 属 001 m7 | 在 002 重做 no_std 会混淆标的。**补救**：C-07 写清用户/内核指针边界，作为 003 接口 |
| **001 规格将 Feature 002 称为 eBPF 入口** | 真实路线是 Rust → Linux → eBPF | 按 001 字面跳进 eBPF 会让 VIII 停在 PARTIAL。**补救**：spec Assumptions 重新对准 |
| **当前开发宿主可能是 WSL2，验收却是 Ubuntu VM** | 澄清选择虚拟机以保证 perf/bpftrace/netns | 把 WSL2 当验收 —— 这些实验经常不可验证。**补救**：R-11；偏离宿主只允许 `experiment-passed` |
| **Constitution XII 的 Benchmark 以机制描述替代** | 本 Feature 无性能目标 | 引入 iperf 会诱导未测量的“快/慢”。**补救**：只断言路径对象与 syscall 集合 |
