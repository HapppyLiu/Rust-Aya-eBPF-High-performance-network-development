# Feature Specification: Linux Foundation

**Feature Branch**: `main`（本项目未为该 Feature 单独开分支）

**Created**: 2026-09-16

**Status**: Draft

**Input**: User description: "设计本学习工程的第二个 feature：002-linux-foundation。目标是为《Rust + Aya/eBPF 高性能网络开发学习计划》建立 Linux Foundation。学习者完成本阶段后必须能够熟练使用 Linux CLI 与诊断工具；理解用户态/内核态、进程线程与 task_struct、系统调用路径、虚拟内存、VFS、I/O 模型、scheduler、IPC 与同步、网络栈、socket/TCP/IP/NIC/netns；能阅读相关内核源码；能用系统调用跟踪、采样分析、调试器与 tracing 观察系统行为；能用实验与系统级程序验证关键机制。本阶段不深入 eBPF 编程和 Aya，但每个 Linux 知识点都必须标注其未来与 eBPF/Aya/高性能网络的关系。采用双轨学习模型（Knowledge Track + Experiment Track）。每个 Module 必须同时生成 Learning Framework、Teaching Material、Feynman Questions、Experiment Tasks、Quiz、Answer Key、Acceptance Criteria。学习框架和答案必须分离。目录结构参考第一个 feature，结果放在 002-linux-foundation 下。"

## Why

后续 Feature（Linux Networking 深化、eBPF、Aya、XDP、TC、AF_XDP、zero-copy）全部建立在
**能读 Linux 执行路径**之上。若跳过本 Feature，学习者在看到 XDP 时会把数据包描述对象当成黑盒，
在看到“从用户内存读数据”的内核辅助时无法说出它在保护哪一层地址空间，在看到端口复用或
零拷贝套接字时无法把现象连回套接字、网卡与调度器。Constitution XI 的路线是
Rust → **Linux** → Networking → eBPF；001 完成了 Rust 侧地基，本 Feature 承担 Linux 侧地基。

001 规格曾把“Feature 002”表述为 eBPF 入口。本 Feature **显式纠正该路线**：002 是
Linux Foundation，eBPF/Aya 编程属于 003+。这不是范围膨胀，而是把 Constitution VIII
（Linux-Kernel-Awareness）从 001 的 PARTIAL 覆盖补成可验收的完整地基。

本 Feature 的价值不在于“学过 Linux 命令”，而在于把 15 项阶段目标所覆盖的 27 项系统级
Linux 能力转化为可验证的、可被后续 Feature 依赖的既有能力。

## Scope

**In scope**：支撑后续 eBPF / Aya / 高性能网络学习的 Linux 子集。覆盖范围以
Learner Outcomes（15 项阶段目标）为对外承诺，以 Capability Coverage（27 项能力）为
可追踪验收单位。每个知识点 MUST 标注其与未来 eBPF / Aya / 高性能网络的关系。

**Out of scope**（明确排除）：

- 编写 eBPF 程序、Aya 程序、XDP/TC/AF_XDP 程序、BPF map 用户态管理、verifier 对抗；
- 成为 Linux kernel developer（不要求提交补丁、不要求掌握全部子系统）；
- io_uring、完整调度器数学推导（含 EEVDF/CFS 公式）、完整 TCP 拥塞控制状态机、
  物理网卡驱动开发、内核模块编写、物理网卡卸载；
- 发行版管理、服务编排、容器编排平台本身。

**允许但不作为课程主体**：把内核事件 tracing 等工具当作**观察手段**使用
（与 001 把未定义行为检测工具当判定工具同理）。使用观察工具 MUST NOT 被表述为
“已经在学 eBPF 编程”。

**本规格不做的决定**：不指定内核源码树获取脚本、依赖库、发行版软件包名，或实验代码的
目录实现细节。此类选择由 `/speckit-plan` 阶段决定。

**基准实验环境**（规格已锁定，见 Clarifications Session 2026-09-16）：带完整 Linux 6.x
内核的 Ubuntu **虚拟机**（x86_64）；内核系列以该虚拟机 `uname -r` 的主.次版本为准；
Rust stable；gcc/clang；gdb；strace；perf；bpftrace；iproute2；tcpdump。
安装步骤仍属 Plan。WSL2 与 Docker 不是验收宿主。

**产物归属**：本 Feature 的全部学习产物 MUST 归属于 `002-linux-foundation`。
目录结构 MUST 参考 Feature 001 已验证的学习者视图 / 答案视图分离模式，而不是另起一套组织方式。

## Learner Outcomes

学习者完成本阶段后 MUST 具备下列 15 项能力。它们是对外承诺；内部拆分为 Capability
Coverage 中的 27 项，以便逐项验收。

| Outcome | 阶段目标 | Capabilities |
|---------|----------|--------------|
| O-01 | 熟练使用 Linux 命令行和常见诊断工具 | C-01, C-02 |
| O-02 | 理解 Linux 用户态与内核态 | C-07 |
| O-03 | 理解进程、线程、内核任务对象、上下文切换 | C-08, C-09 |
| O-04 | 理解 Linux 系统调用的基本执行路径 | C-10 |
| O-05 | 理解虚拟内存、页表、物理页、地址翻译缓存、缺页、内存映射 | C-11, C-12, C-13, C-14 |
| O-06 | 理解文件描述符、虚拟文件系统、索引节点、文件对象、打开/读/写 | C-15, C-16, C-17 |
| O-07 | 理解 Linux I/O 模型 | C-18 |
| O-08 | 理解调度器的基本机制 | C-19 |
| O-09 | 理解进程间通信和基础同步机制 | C-20, C-21 |
| O-10 | 理解 Linux 网络栈基本结构 | C-22 |
| O-11 | 理解套接字、TCP/IP、网卡、网络命名空间 | C-23, C-24, C-25, C-26 |
| O-12 | 能够阅读相关 Linux 内核源码 | 跨切：每项能力的源码定位 |
| O-13 | 能够使用系统调用跟踪、采样分析、调试器、内核事件 tracing 观察系统行为 | C-03, C-04, C-05, C-06 |
| O-14 | 能够通过实验验证 Linux 的关键机制 | 跨切：每项能力至少一个实验 |
| O-15 | 能够使用系统级程序（含 Rust）编写部分 Linux 实验 | C-27；US1–US6 允许命令行与程序并存 |

O-12 与 O-14 不是独立 C-ID：它们是**每项能力的强制属性**（见 FR-003、FR-005）。

## Capability Coverage

27 项核心能力与学习旅程（User Story）的映射，用于 Constitution XIII 要求的可追踪性：

| ID | Capability | Story | Outcome |
|----|-----------|-------|---------|
| C-01 | Linux CLI fluency | US1 | O-01 |
| C-02 | Common diagnostic tools | US1 | O-01 |
| C-03 | Trace system calls from user programs | US1 | O-13 |
| C-04 | Profile execution with a sampling profiler | US1 | O-13 |
| C-05 | Inspect a live or core process with a debugger | US1 | O-13 |
| C-06 | Observe kernel events with a tracing one-liner | US1 | O-13 |
| C-07 | User space vs kernel space | US2 | O-02 |
| C-08 | Process, thread, and task_struct | US2 | O-03 |
| C-09 | Context switch | US2 | O-03 |
| C-10 | System call execution path | US2 | O-04 |
| C-11 | Virtual memory | US3 | O-05 |
| C-12 | Page tables, Page, TLB | US3 | O-05 |
| C-13 | Page Fault | US3 | O-05 |
| C-14 | mmap | US3 | O-05 |
| C-15 | File descriptors | US4 | O-06 |
| C-16 | VFS, inode, and file | US4 | O-06 |
| C-17 | open / read / write path | US4 | O-06 |
| C-18 | I/O models | US4 | O-07 |
| C-19 | Scheduler | US5 | O-08 |
| C-20 | IPC | US5 | O-09 |
| C-21 | Synchronization primitives | US5 | O-09 |
| C-22 | Network stack structure | US6 | O-10 |
| C-23 | Socket | US6 | O-11 |
| C-24 | TCP/IP | US6 | O-11 |
| C-25 | NIC | US6 | O-11 |
| C-26 | Network namespace | US6 | O-11 |
| C-27 | Rust Linux system experiments (capstone) | US7 | O-15 |

**跨切能力（不是独立 C-ID，而是每项能力的强制属性）**：

- **Kernel source reading（O-12）**：每项能力 MUST 定位至少一处 Linux 内核结构体、函数或调用路径
  （FR-005）。
- **eBPF/Aya/高性能网络关联**：每项能力 MUST 标注下游关联（FR-014）。
- **Experiment verification（O-14）**：每项能力 MUST 至少对应一个可重复实验（FR-003）。

## Dual-Track Learning Model

本 Feature 采用双轨学习模型。每个学习模块 MUST **同时**具备两条轨道，缺一不可。
轨道描述的是学习者完成模块时必须经历的活动，不是实现阶段的目录树。

### Track A — Knowledge Track

学习者必须能够：

- 陈述概念（是什么、不是什么）
- 画出或口述架构（对象如何协作、边界在哪里）
- 定位内核源码中的关键结构或路径
- 独立回答 Feynman 问题（不含答案）
- 独立完成 Quiz（不含答案）

### Track B — Experiment Track

学习者必须能够：

- 使用 Linux 命令完成诊断与观察
- 运行最小 C 或系统级程序（含 Rust）触发目标机制
- 用系统调用跟踪、采样分析、调试器、内核事件 tracing、网络诊断工具中与该能力相关的子集
  核对自己的预测
- 把稳定观察与非断言输出分开记录

### Per-Module Artifact Set

每个模块 MUST 同时生成以下全部七类产物。学习框架与答案 MUST 物理分离：学习者 MUST
能够只打开 Learning Framework、Feynman Questions、Quiz 与 Experiment Tasks 完成自测，
然后再打开 Answer Key 纠错。

| # | 产物 | 轨道 | 是否含答案 | 学习者打开时机 |
|---|------|------|-----------|----------------|
| 1 | Learning Framework | A + B 的导航 | 否 | 进入模块时 |
| 2 | Teaching Material | A（概念、架构、源码定位） | 是（机制解释） | 自测之后，或提示用尽之后 |
| 3 | Feynman Questions | A（提问版） | 否 | 学完概念、做实验前后 |
| 4 | Experiment Tasks | B（先预测后验证） | 否（任务本身） | 与 Framework 同时 |
| 5 | Quiz | A | 否 | 模块自测 |
| 6 | Answer Key | A + B 的参考回答 | 是 | 自测提交之后 |
| 7 | Acceptance Criteria | 验收 | 判定标准，不是 Quiz 答案 | 模块结束时对照 |

Answer Key MUST 至少覆盖：Quiz 参考回答、Feynman 五项检验的参考回答、实验稳定断言的解读。
Teaching Material MUST 覆盖概念、架构与内核源码定位，MUST NOT 与 Learning Framework 放在
同一学习者自测视图中。

## Clarifications

本 Feature 在规格阶段对可合理默认的决策直接裁定；本轮 `/speckit-clarify` 把原先推迟的
环境决策写入规格。不保留未决的 `[NEEDS CLARIFICATION]` 标记。

1. **Feynman 粒度**：与 001 相同的混合模型 —— Feynman 提问版与参考回答按 7 个 Story
   模块产出；27 项能力各自保留独立的 Acceptance Criteria 与实验。提问版与参考回答 MUST
   分开放置。
2. **观测工具（基准 vs 偏离）**：在基准环境中，gdb、strace、perf、bpftrace、iproute2、
   tcpdump MUST 视为可用；C-03…C-06 与网络观察实验 MUST 用这些工具产出判定依据，
   MUST NOT 在基准宿主上走“工具缺失”的等价路径来把能力标为 `accepted`。
   仅当实验跑在**非基准**宿主上时，才允许记录探测失败并以进程信息接口、系统调用跟踪
   或书面内核路径作为 `experiment-passed` 的临时载体；Capability 进入 `accepted` 仍要求
   在基准环境补跑原观察实验。
3. **重跑一致性**：与 001 相同 —— 每个实验显式声明稳定断言；进程号、时间戳、地址、
   调度交错、网卡统计瞬时值标注为非断言输出。
4. **系统级实验的前置**：本 Feature 的可执行实验以 Linux 机制为学习标的，Rust 只是
   其中一种载体。基准环境提供 gcc/clang，因此 US1–US6 **允许** C 程序，但 MUST NOT
   要求每个模块都有一份 C 实验；Rust 足以触发机制时用 Rust。US7 强制用 Rust 综合验证。
   学习者被假定已完成 001，但不把 001 的 unsafe/no_std 作为本 Feature 的硬前置
   （本 Feature 不编写 eBPF）。
5. **不深入 eBPF**：C-06 的 tracing 观察是示波器，不是编程课。允许使用 bpftrace
   一行观察；任何需要编写 BPF 程序、加载 BPF 对象、或使用 Aya 接口的任务都属于
   后续 Feature。
6. **目录结构**：产物组织 MUST 参考 Feature 001 的学习者视图 / 答案视图分离，并全部
   落在本 Feature（`002-linux-foundation`）名下。具体子目录名由 Plan 对照 001 决定，
   本规格只约束分离与归属，不约束文件名。
7. **提权**：实验 MUST 声明是否需要 root 或等价能力。C-04（perf）、C-06（bpftrace）、
   C-26（network namespace）以及任何创建网络命名空间、加载 tracing 探针的实验，在
   Ubuntu 虚拟机上 MUST 以 root 或等价能力跑通才可 `accepted`。普通用户失败 MUST NOT
   作为这些能力的最终通过。strace、普通文件读写、本机回环套接字等不需要提权的实验
   MUST 仍以普通用户执行。
8. **深度上限**：I/O 止于阻塞 / 非阻塞 / epoll；调度止于运行队列、唤醒、时间预算；
   TCP 止于连接、基本状态与数据包对象；网卡止于虚拟机内的 NAPI / virtio 收发包路径。
   io_uring、TCP 拥塞控制、物理网卡驱动与卸载 MUST NOT 进入本 Feature。
9. **network namespace 的使用范围**：C-26 MUST 在 Ubuntu 虚拟机上实际创建并使用至少
   两个网络命名空间。C-22…C-25 的路径实验 MAY 使用本机回环，MUST NOT 把 netns 当作
   那些能力的前置。

### Session 2026-09-16

- Q: 本 Feature 的基准实验环境是什么？ → A: Ubuntu Linux；x86_64；Linux Kernel 6.x；Rust stable；gcc/clang；gdb；strace；perf；bpftrace；iproute2；tcpdump。
- Q: 验收实验必须跑在哪种 Ubuntu 宿主上：物理机本机、完整内核虚拟机、WSL2，还是 Docker 容器？ → A: Ubuntu 虚拟机（完整内核）是验收宿主。
- Q: 内核源码引用和验收实验应该钉在哪个 6.x 上：虚拟机当时运行的主.次版本、固定的 6.6 LTS，还是任意 6.x 都可以混用？ → A: 用虚拟机实际运行的 6.x 主.次版本；源码树必须与之同系列，并记录 `uname -r`。
- Q: 在验收用的 Ubuntu 虚拟机上，需要看内核事件或创建网络命名空间的实验，是必须用管理员权限跑通，还是普通用户跑不通也可以算过关？ → A: 需要提权的实验必须在虚拟机上用 root（或等价能力）跑通才算验收通过；不需要提权的实验仍用普通用户。
- Q: 本阶段对调度、I/O、TCP 和网卡，学到哪一层就应该停，以免超出 Linux Foundation？ → A: 地基深度：epoll、运行队列/唤醒、TCP 连接与数据包对象、虚拟网卡 NAPI 路径；排除 io_uring、拥塞控制、驱动开发。

## User Scenarios & Testing *(mandatory)*

本 Feature 的“用户”是学习者本人。Story 按**学习顺序**编号（Constitution XI 要求递进），
优先级 tier 表示**阻塞性**而非顺序：

- **P1** = 后续 eBPF/Aya/高性能网络的硬前置，缺失将直接阻塞 003+
- **P2** = 阅读内核路径与编写系统程序所需，可在相邻 P1 之后补齐
- **P3** = 整合与终验收

每个 Story 模块在独立测试时 MUST 同时走完 Track A 与 Track B：先用 Learning Framework
自测，再用实验核对预测，最后对照 Answer Key 与 Acceptance Criteria。

### User Story 1 - CLI、诊断与观测工具 (Priority: P1)

学习者能在一台 Linux 机器上，不依赖图形界面，完成进程、文件、网络与命名空间的日常诊断；
能在运行前**预测**一个最小程序会触发哪些系统调用与哪些可观测事件，再用跟踪、采样、
调试与 tracing 工具核对自己的预测。

**Why this priority**: 后续所有内核路径学习都要用这些工具验证“我以为发生的”与
“实际发生的”是否一致。没有观测能力，Source-Code-First 会退化成抄路径。

**Independent Test**: 给定一个会读写文件并监听本地端口的最小程序，学习者在运行前写下
预期系统调用序列与预期出现的套接字/文件描述符；运行后用诊断与跟踪工具核对，预测与
稳定断言一致即通过。仅打开 Learning Framework 与 Experiment Tasks 即可完成该自测。

**Acceptance Scenarios**:

1. **Given** 一台只有命令行的 Linux 环境，**When** 学习者需要定位某个进程的标识、打开的
   文件、内存映射与网络套接字，**Then** 能仅用命令行与进程信息接口完成，且能解释每条信息
   来自哪一类内核对象。
2. **Given** 一个最小用户态程序，**When** 学习者先写下它将触发的系统调用再跟踪执行，
   **Then** 预测覆盖全部稳定断言中的系统调用名，且学习者能区分“库函数名”与“真正进入内核的调用”。
3. **Given** 采样分析、调试器、内核事件 tracing 三类工具，**When** 环境中某工具不可用，
   **Then** 学习者能说明该工具本应观察哪一层，并改用进程信息接口或系统调用跟踪给出等价证据，
   而不是把“没装工具”写成“已掌握”。

---

### User Story 2 - 用户态/内核态、进程模型与系统调用路径 (Priority: P1)

学习者能够说明用户态与内核态的边界、一次系统调用如何从用户指令跨越到内核入口再返回，
以及 Linux 如何用同一个内核任务对象同时表示进程与线程；能够指出上下文切换保存与恢复的是什么。

**Why this priority**: eBPF 程序运行在内核态，用户态加载器运行在用户态。分不清两边，
就无法解释为什么用户态指针不能直接在内核中解引用、为什么必须通过受控方式读取用户内存。
系统调用路径是 Constitution VIII 的最小完整落点。

**Independent Test**: 给定一段“用户函数 → 库封装 → 陷入内核 → 返回”的最小程序，学习者
独立画出执行路径并标注内核任务对象中与该过程相关的字段；用跟踪工具与进程信息接口验证
进程/线程标识关系与自愿/非自愿上下文切换计数的变化方向。

**Acceptance Scenarios**:

1. **Given** 一个普通函数调用与一次系统调用，**When** 学习者对比二者，**Then** 能说明
   权限、栈、指令指针与可访问地址空间在跨越边界时发生了什么，而不是只说“进了内核”。
2. **Given** 一个多线程程序，**When** 学习者查看线程标识与线程组标识，**Then** 能说明线程在
   Linux 中不是独立于进程的第二套抽象，而是共享地址空间的内核任务对象，并能指向
   内核中该结构的位置。
3. **Given** 一次主动让出或阻塞式读，**When** 学习者观察上下文切换计数，
   **Then** 能解释自愿与非自愿切换的差异，以及切换时必须保存的最小状态。

---

### User Story 3 - 虚拟内存、页表、缺页与内存映射 (Priority: P1)

学习者能够把“进程以为自己有连续地址空间”还原为页表、物理页、地址翻译缓存与缺页处理的协作；
能够解释内存映射如何创建映射、何时真正分配物理页，以及缺页在匿名映射与文件映射上的差异。

**Why this priority**: 零拷贝、用户态数据包内存池、eBPF 对用户内存的访问、以及
“为什么最早路径的数据包处理不能随便碰用户指针”，全部建立在这套模型上。

**Independent Test**: 学习者独立完成一次匿名内存映射：映射后、触碰前、触碰后分别记录
映射存在性与缺页计数；预测与实测在稳定断言上一致，并能指出页表/缺页处理在内核中的入口。

**Acceptance Scenarios**:

1. **Given** 进程映射表中的一段映射，**When** 学习者解释它，**Then** 能区分
   虚拟地址、权限、文件后端与物理页是否已驻留，且不以“占用了这么多内存”混淆虚拟大小与驻留大小。
2. **Given** 一次匿名映射后尚未触碰的区域，**When** 学习者读取缺页计数再写入该区域，
   **Then** 能解释为什么映射存在不等于物理页存在，并指出轻微缺页与严重缺页的差别。
3. **Given** 地址翻译缓存与页表两层查找结构，**When** 学习者说明一次用户态访存，
   **Then** 能说出命中翻译缓存、走页表、陷入缺页三种路径各自在什么条件下发生。

---

### User Story 4 - 文件描述符、虚拟文件系统与 I/O 模型 (Priority: P2)

学习者能够把“打开一个文件”还原为描述符表、文件对象、目录项、索引节点与具体文件系统的协作；
能够跟踪打开/读/写的内核路径；能够对比阻塞、非阻塞与 epoll 多路复用各自把等待放在哪一层。
本阶段 MUST NOT 把 io_uring 或通用异步 I/O 框架当作验收内容。

**Why this priority**: 套接字也是文件描述符；多路复用是后续网络程序的日常路径；后续
BPF 对象与映射表的用户态句柄建立在同一套描述符/虚拟文件系统模型上。

**Independent Test**: 学习者用同一段数据分别走阻塞读与非阻塞读（或等价的多路复用），
能说明两种模型在描述符状态、进程可运行性与系统调用返回值上的差异，并用跟踪工具验证。

**Acceptance Scenarios**:

1. **Given** 同一个文件被打开两次，**When** 学习者比较两个描述符，**Then** 能说明描述符号、
   文件对象与索引节点三者谁共享、谁独立，以及关闭其中一个描述符影响什么。
2. **Given** 一次读操作，**When** 学习者跟踪它，**Then** 能从用户态调用点说到虚拟文件系统
   通用层，再说明“真正读数据”发生在哪一类后端（文件、管道、套接字）。
3. **Given** 阻塞 I/O 与非阻塞 I/O 两种写法，**When** 学习者在尚未就绪的描述符上调用读，
   **Then** 能预测谁会睡眠、谁会立即返回，以及多路复用要解决的是哪一个问题。

---

### User Story 5 - 调度、进程间通信与同步 (Priority: P2)

学习者能够说明 Linux 调度器决定“下一个运行谁”的基本机制，能够选择合适的进程间通信
（管道、本地套接字、共享内存等）并解释数据拷贝发生在哪一层，能够区分用户态锁、
用户态/内核态协作等待与内核自旋锁各自保护的对象。

**Why this priority**: 高性能网络的尾延迟经常来自调度与锁。eBPF 程序不能随意睡眠；
用户态无锁结构与内核同步原语的边界必须在进入 Aya 之前建立。

**Independent Test**: 学习者构造一个会阻塞在进程间通信上的最小程序，观察其调度状态从
可运行变为不可运行再恢复；并构造一次用户态锁竞争，指出哪些等待进入了内核、哪些没有。

**Acceptance Scenarios**:

1. **Given** 一个处理器密集线程与一个刚被唤醒的线程，**When** 学习者解释调度器的选择，
   **Then** 能说出至少“运行队列 / 唤醒 / 时间预算”三件事，而不把调度器简化成
   “谁优先级数字小谁跑”，也 MUST NOT 被要求推导 CFS/EEVDF 公式。
2. **Given** 管道与共享内存两种进程间通信，**When** 学习者对比一次传输，**Then** 能指出
   拷贝次数与同步责任的差异。
3. **Given** 一次用户态互斥锁竞争，**When** 学习者跟踪它，**Then** 能说明未竞争路径
   可以不进入内核，竞争路径如何进入内核睡眠，以及这与内核自旋锁的适用场景不同。

---

### User Story 6 - 网络栈、套接字、TCP/IP、网卡与网络命名空间 (Priority: P1)

学习者能够画出从用户态发送到网卡发出、以及从网卡收到到用户态接收的基本路径；
能够说明套接字、TCP/IP 协议层、数据包描述对象、驱动收包机制与网卡的分工；能够解释
网络命名空间隔离的是哪一类对象。

**Why this priority**: 这是后续在网卡收包最早路径挂钩、流量控制挂钩、套接字过滤、
用户态零拷贝套接字的直接前置。缺失本 Story，eBPF 网络程序只能停留在接口。本 Story 是 P1，
即使它在学习顺序上靠后。

**Independent Test**: 学习者在虚拟机回环上完成一次 TCP 或等价流传输，能指出发送与
接收路径上至少五个命名内核对象/函数（含数据包对象与 NAPI/virtio 路径上的一环）；
并另用一对网络命名空间完成 C-26：在一方创建套接字与地址，证明对另一方的隔离范围。

**Acceptance Scenarios**:

1. **Given** 一次本机 TCP 连接，**When** 学习者说明数据路径，**Then** 能区分套接字、
   传输层、网络层、流量控制/队列、驱动与（虚拟）网卡，且不把“套接字就是网卡”当作解释。
   MUST NOT 被要求陈述拥塞控制算法细节。
2. **Given** 内核数据包描述对象，**When** 学习者指出它的职责，
   **Then** 能说明它为何存在、它携带哪些头、以及后续最早路径处理为什么常常想避开它。
3. **Given** 两个网络命名空间，**When** 学习者在其中一方创建套接字与地址，
   **Then** 能解释对另一方可不可见，以及隔离的是网络栈对象而非进程本身。
   该场景 MUST 在 Ubuntu 虚拟机上真实创建 namespace，MUST NOT 仅用回环口头类比。

---

### User Story 7 - 综合实验：用系统级程序验证 Linux 关键机制 (Priority: P3)

学习者完成一个整合前六个 Story 的综合实验：用 Rust 编写系统级程序，覆盖进程/线程、
系统调用、内存映射、文件描述符、I/O 等待与本地网络传输中的若干关键机制，并用
观测工具验证自己的预测。

**Why this priority**: 单项能力通过不等于能组合使用。本 Story 是 Constitution XII
闭环中的 Build 环节，也是进入 Feature 003（eBPF/Aya）的准入检查。

**Independent Test**: 学习者独立完成该综合实验，产物可重复构建与运行，并能对照
27 项能力清单逐项指出其在实验中的体现位置（允许“本综合实验不直接体现、已在模块实验体现”
的显式标注，但不得遗漏未标注项）。

**Acceptance Scenarios**:

1. **Given** 综合实验完成，**When** 对照 27 项能力清单逐项检查，**Then** 每一项都能在
   实验产物、模块实验或配套说明中定位到具体体现，无遗漏项。
2. **Given** 实验中的系统级程序，**When** 审查其 Linux 交互，**Then** 学习者能说明
   每一处系统调用或映射的预期内核路径，而非只证明“程序能跑”。
3. **Given** 完整环境记录，**When** 在记录的环境中重新执行构建与运行命令，
   **Then** 全部稳定断言复现，非断言输出的差异不影响判定。

### Edge Cases

- 某项观测工具在**非基准**宿主上不可用时，如何判定对应能力？
  规格要求：探测失败 MUST 被记录；必须走 Clarifications 第 2 条的等价观察路径；
  不得把“命令不存在”写成掌握。基准 Ubuntu 宿主上这些工具缺失视为环境未就绪，
  MUST NOT 把该次运行计为 `accepted`。
- 实验结果在不同处理器架构、或不同内核主.次版本上不同时，如何判定？规格要求：验收只以
  Ubuntu 虚拟机当时运行的 6.x 主.次版本为准。能解释其他版本差异来源者可作为对照笔记，
  仅记录现象、无法对应本机系列者视为该条源码引用未完成。
- 内核源码树与运行中的内核版本不完全一致时，源码引用是否有效？规格要求：源码树 MUST
  与虚拟机 `uname -r` 的主.次版本同系列。允许同系列内行号漂移，但符号与文件路径 MUST
  仍能定位；主.次版本不一致或找不到符号视为引用失败。
- 需要提升权限或创建网络命名空间的实验在无权限环境失败时如何判定？
  规格要求：实验 MUST 声明所需权限。在 Ubuntu 虚拟机上，声明需要提权的实验
  （至少包括 C-04、C-06、C-26）MUST 用 root 或等价能力跑通；普通用户失败 MUST NOT
  计为 `accepted`。在 WSL2 / Docker 等偏离宿主上无权限时，可记录失败并改用回环完成
  路径讲解，该能力可达到 `experiment-passed`，但 MUST 在 Ubuntu 虚拟机上以提权方式
  补跑后才能 `accepted`。
- 在 WSL2 或 Docker 中跑通的实验是否算本 Feature 完成？规格要求：不算最终验收。
  这两类宿主是偏离环境；C-04 / C-06 / C-26 以及需要完整内核观测的条目 MUST 在
  Ubuntu 虚拟机上复现。
- Feynman 五项检验仅一项无法完成？规格要求：模块判定为未完成，生成补齐任务；五项不接受
  部分通过（与 001 / Constitution IV 同一口径）。
- 学习者只打开 Answer Key 而未先完成 Learning Framework 自测？规格要求：不阻止打开，
  但该模块 MUST NOT 被标记为完成，直到存在一次“先自测、后对照”的记录。
- 学习者想在本 Feature 编写最早路径数据包程序或 Aya 程序？规格要求：拒绝。记为范围外，
  引导至 003+。
- 学习者要求覆盖 io_uring、TCP 拥塞控制、物理网卡驱动或卸载？规格要求：拒绝。
  记为超出 Linux Foundation，留给后续 Networking / 高性能网络 Feature。
- 某模块七类产物缺一类？规格要求：该模块 MUST NOT 标记完成（FR-002）。

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: 本 Feature MUST 覆盖 Capability Coverage 表中全部 27 项能力（C-01 至 C-27），
  每项 MUST 归属到恰好一个 Story，且 MUST NOT 存在无归属的能力项。15 项 Learner Outcomes
  MUST 全部被这些能力或跨切属性覆盖，MUST NOT 出现未映射的阶段目标。
- **FR-002**: 每个学习模块 MUST 同时生成以下全部产物，缺一 MUST NOT 标记为完成：
  1. Learning Framework（学习者视图，不含答案）
  2. Teaching Material（概念 + 架构 + 内核源码定位）
  3. Feynman Questions（提问版，不含答案）
  4. Experiment Tasks（先预测后验证）
  5. Quiz（学习者可独立作答）
  6. Answer Key（与框架/测验物理分离）
  7. Acceptance Criteria
- **FR-003**: 每项能力 MUST 至少对应一个可重复执行的最小实验。实验 MUST 记录执行命令、
  输入与实际输出，并 MUST 显式声明稳定断言。进程号、地址、时间戳、调度交错、计数器瞬时值
  MUST 标注为非断言输出。
- **FR-004**: Experiment Track MUST 能使用 Linux 命令、C 或系统级程序、系统调用跟踪、采样分析、
  调试器、内核事件 tracing 与网络诊断工具中与该能力相关的子集。Knowledge Track MUST
  覆盖 Concepts、Architecture、Kernel source、Feynman questions、Quiz。
- **FR-005**: 每项能力 MUST 至少定位一处 Linux 内核源码中的实际结构体、函数或调用路径，
  并记录文件路径与符号名。引用 MUST 绑定 Ubuntu 虚拟机运行内核的主.次版本系列
  （取自 `uname -r`，且 MUST 为 6.x）。源码树 MUST 与该主.次版本同系列，MUST NOT
  用另一个 6.x 小版本冒充。若某机制在架构相关目录，MUST 记录 x86_64 下的路径。
- **FR-006**: Feynman 五项检验 MUST 以学习模块为单位执行，7 个 Story 各产出一份覆盖其
  全部所属能力的 Feynman Questions 与对应 Answer Key 参考回答；任一项未通过 MUST 记为未完成
  并生成补齐任务。
- **FR-007**: 每项能力 MUST 具备可验证的 Acceptance Criteria；“看过”“了解过”“做过笔记”
  MUST NOT 被接受为完成标准。
- **FR-008**: 学习框架与答案 MUST 物理分离。学习者 MUST 能够只打开 Learning Framework、
  Quiz、Feynman Questions 与 Experiment Tasks 完成自测，然后再打开 Answer Key。
  学习者自测视图中 MUST NOT 出现稳定断言的预期值、内核行号、机制性结论或 Answer Key 原文。
- **FR-009**: 本 Feature MUST NOT 包含实际 eBPF 程序、Aya 程序、XDP/TC/AF_XDP 程序的编写
  或加载。C-06 仅允许把 tracing 工具当作观察手段。
- **FR-010**: 每个实验 MUST 附带环境记录，至少包含：所用系统级语言工具链版本（若该实验用到）、
  内核版本、处理器架构、相关工具是否可用、执行命令与所需权限。
- **FR-011**: 学习顺序 MUST 遵循 US1 → US2 → US3 → US4 → US5 → US6 → US7；任何跳过
  MUST 在 Plan 中显式记录理由与补齐计划。
- **FR-012**: US1、US2、US3、US6 是进入 Feature 003（eBPF/Aya）的硬前置。US4、US5、US7
  亦 MUST 在启动 003 前完成。硬前置是必要条件，不是充分条件；
  `可启动 003` ⟺ `US1..US7 全部 complete`。
- **FR-013**: 每个学习目标 MUST 可追踪到 Spec → Plan → Task → Learning Material →
  Experiment → Source Code → Acceptance Criteria；MUST NOT 产生无归属的孤立笔记。
- **FR-014**: 每项能力 MUST 说明其与后续 eBPF / Aya / 高性能网络学习的关联点，MUST NOT
  省略。允许写“本能力支撑的是诊断/排障，不直接对应某一 BPF 程序类型”，但不允许空白。
- **FR-015**: US7 综合实验 MUST 使用 Rust 编写部分系统级程序，MUST 整合前六个 Story
  的机制，产出 MUST 可重复构建与运行。
- **FR-016**: 本规格 MUST NOT 指定发行版软件包名、依赖库或内核源码获取脚本；
  此类决策 MUST 推迟至 Plan 阶段。基准 OS、架构、内核主版本系列与工具清单已由
  Clarifications Session 2026-09-16 锁定，MUST NOT 在 Plan 中改口为另一套环境。
- **FR-017**: 涉及性能的陈述 MUST 遵守 Constitution IX：未测量的性能主张 MUST 标注为假设。
  本 Feature 不设网络吞吐目标。
- **FR-018**: 当实验结果依赖内核版本、架构、受限环境或权限时，产出 MUST 记录这些条件，
  并说明是否可跨环境推广。
- **FR-019**: 观测类实验 MUST 以工具输出或进程信息接口的可读字段作为判定依据，MUST NOT 以
  “我看见了”作为通过证据。工具不可用时走 Clarifications 第 2 条。
- **FR-020**: 本 Feature 的 Rust 实验 MUST 沿用仓库已锁定的工具链，MUST NOT
  在本 Feature 期间升级该工具链；Linux 实验以运行中的内核为准并记录版本。
- **FR-021**: 本 Feature 的全部学习产物 MUST 归属于 `002-linux-foundation`，MUST NOT
  写入 Feature 001 的产物位置。学习者视图与答案视图 MUST 采用与 Feature 001 相同的
  物理分离模式。具体子目录命名由 Plan 对照 Feature 001 决定。
- **FR-022**: 基准验收宿主 MUST 为带完整 Linux 6.x 内核的 Ubuntu 虚拟机（x86_64），
  并 MUST 具备 Rust stable、gcc 或 clang、gdb、strace、perf、bpftrace、iproute2、
  tcpdump。内核系列 MUST 取该虚拟机 `uname -r` 的主.次版本，环境记录 MUST 写下该值。
  针对 C-03…C-06 与网络观察的验收 MUST 在该虚拟机上用对应工具完成。
  WSL2 与 Docker MUST NOT 作为 C-04、C-06、C-26 或任何需要完整内核观测之能力的
  最终验收宿主；它们上的结果 MAY 记为偏离观察。
- **FR-023**: 实验 MUST 声明所需权限。C-04、C-06、C-26 以及创建网络命名空间或加载
  内核事件 tracing 的实验，在基准虚拟机上 MUST 以 root 或等价能力执行且稳定断言通过，
  方可进入 `accepted`。不需要提权的实验 MUST 以普通用户执行。普通用户下的失败
  MUST NOT 被当作这些提权能力的通过证据。
- **FR-024**: 本 Feature 的知识深度 MUST 遵守下列上限：C-18 止于阻塞 / 非阻塞 / epoll；
  C-19 止于运行队列、唤醒与时间预算；C-24 止于连接、基本状态与数据包对象；
  C-25 止于虚拟机内 NAPI / virtio 收发包路径。io_uring、调度器公式、TCP 拥塞控制、
  物理网卡驱动与卸载 MUST NOT 作为本 Feature 的验收内容。
- **FR-025**: C-26 MUST 在 Ubuntu 虚拟机上实际创建至少两个网络命名空间并完成隔离观察。
  C-22…C-25 MUST NOT 把 network namespace 当作前置；回环路径足以验收那些能力。
  gcc/clang 可用，C 程序 MAY 用于系统调用对照；MUST NOT 要求每个模块都有 C 实验。

### Key Entities

- **Learner Outcome（阶段目标）**：完成本 Feature 后对外承诺的 15 项能力（O-01 至 O-15）。
  由一到多项 Capability 或跨切属性实现。
- **Capability（能力项）**：27 项系统级 Linux 能力之一，具有唯一 ID（C-01 至 C-27）、
  所属 Story、所属 Outcome、下游 eBPF/Aya 关联、以及完成状态。是本 Feature 的最小可验收单位。
- **Learning Module（学习模块）**：与 Story 一一对应的学习单元，共 7 个。每个模块同时
  具备 Knowledge Track 与 Experiment Track，并同时产出 FR-002 所列七类产物。
- **Learner Track Artifact**：不含答案的学习框架、Feynman 提问、预测任务、自检、测验。
- **Answer Track Artifact**：概念与架构材料、源码引用、Feynman 参考回答、测验答案、实验实现。
- **Experiment（实验）**：可重复执行的最小验证单元，载体可以是 Linux 命令、C 程序、
  系统级程序或观测工具调用；包含稳定断言与非断言输出。
- **Source Reference（源码引用）**：指向 Linux 内核源码中具体位置的记录，包含文件路径、
  符号名与内核版本系列。
- **Feynman Material（Feynman 材料）**：提问版与参考回答成对出现；五项检验合取通过。
- **Acceptance Criterion（验收标准）**：针对某能力项的可验证完成条件。
- **Environment Record（环境记录）**：实验的可复现上下文。
- **Tool Probe（工具探测）**：记录某观测工具是否可用；不可用时触发等价观察路径。
- **Downstream Link（下游关联）**：该 Linux 能力与后续 eBPF / Aya / 高性能网络的关系说明。

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 7 个 Story 模块 100% 具备已通过 Feynman 五项检验的材料（提问版 + 参考回答），
  且每份材料覆盖其所含能力项的全部条目（27 项能力在模块材料中的覆盖率为 100%）。
- **SC-002**: 27 项能力中 100% 至少对应一个实验，且这些实验在**基准环境**中重新执行时，
  稳定断言全部复现的比例达到 100%（非断言输出的差异不计入）。仅非基准宿主上的
  工具不可用条目，才允许以探测记录 + 等价观察计入分母；基准宿主上 gdb / strace /
  perf / bpftrace / iproute2 / tcpdump 缺失计为环境未就绪，不计通过。
- **SC-003**: 27 项能力中 100% 至少关联一处带文件路径与符号名的内核源码引用，且引用
  所绑定的主.次版本与验收虚拟机 `uname -r` 同系列。
- **SC-004**: 27 项能力中 100% 具有一条指向后续 eBPF / Aya / 高性能网络的关联说明。
- **SC-005**: 学习者能在不打开 Answer Key 的前提下，仅用 Learning Framework、Feynman
  Questions、Quiz 与 Experiment Tasks 完成一个模块的自测；Answer Key 与 Framework
  位于不同的学习者视图。抽查任意模块，学习者自测视图中稳定断言预期值、内核行号、
  机制性结论的出现次数为 0。
- **SC-006**: 给定一个会进行文件读写与本地网络操作的最小程序，学习者能在运行前写出
  预期系统调用集合，运行后与跟踪结果相比，稳定集合的漏报数为 0。
- **SC-007**: 学习者能独立完成一次匿名内存映射实验：映射存在、触碰前未分配物理页、
  触碰后缺页计数变化，三项判断全部正确。
- **SC-008**: 学习者能说明一次本机数据发送从套接字到网卡方向上至少五个命名对象或函数，
  且能解释网络命名空间隔离的对象类别。
- **SC-009**: US7 综合实验完成，且 27 项能力全部能在综合实验或模块实验中定位到具体体现。
- **SC-010**: 本 Feature 产物中，eBPF/Aya/XDP 程序数量为 0。
- **SC-011**: 所有学习目标均可追溯到完整链条，孤立笔记数量为 0。
- **SC-012**: 7 个模块中 100% 同时具备 FR-002 所列七类产物；任一模块缺一类即该模块未完成。
  15 项 Learner Outcomes 中 100% 能在 Capability Coverage 或跨切属性中定位到对应条目。
- **SC-013**: 抽查本 Feature 产物归属，写入 Feature 001 产物位置的本 Feature 文件数为 0。

## Assumptions

- 学习者已完成 Feature 001（Rust Foundation），能够阅读与编写中等复杂度的安全 Rust，
  并理解基本跨语言调用。本 Feature 不把 001 的 unsafe/no_std 模块当作硬前置，因为本阶段
  不编写 eBPF。
- 基准验收宿主是 Ubuntu 虚拟机（x86_64）。内核系列以该虚拟机 `uname -r` 的主.次版本
  为准（必须是 6.x），源码树必须同系列；工具清单见 FR-022。
  WSL2 与 Docker 是偏离宿主：可用来起草材料，MUST NOT 静默当作最终验收环境。
  Rust 使用仓库已锁定的 stable 工具链（FR-020），与“Rust stable”一致，不是每次最新。
  C-04 / C-06 / C-26 在虚拟机上需要 root 或等价能力（FR-023）。
  深度上限见 FR-024。C 实验不强制每个模块；C-26 必须真实创建 netns，其余网络能力可用回环
  （FR-025；因问题配额用尽而按合理默认写入，若需改口应在 Plan 登记）。
- 每项能力的学习深度以“能支撑后续 eBPF/Aya/高性能网络”为界，不追求内核开发者级别的完备性。
- 本 Feature 无固定日历工期，完成与否由 Acceptance Criteria 判定。
- 本项目为单人学习工程，评审由学习者对照可验证产物自评；产物的可验证性是评审有效性的前提。
- 具体工具安装方式、内核源码树位置、依赖库选择由 Plan 阶段确定；工具**是否在基准中存在**
  已由本规格锁定。
- 产物目录结构参考 Feature 001 的学习者/答案分离，全部放在 `002-linux-foundation` 名下；
  不在本规格中锁定文件名。
- 001 规格中“Feature 002 = eBPF 入口 / 部分 Story 为 002 硬前置”的表述，被本规格
  **重新对准**：002 是 Linux Foundation；001 全部 Story 完成仍是启动本 Feature 的
  推荐条件，但本 Feature 的硬前置改为“具备 001 的安全 Rust 读写能力”。001 文档不在
  本 Feature 中直接改写；偏差在 Plan 的 Complexity Tracking 登记。
