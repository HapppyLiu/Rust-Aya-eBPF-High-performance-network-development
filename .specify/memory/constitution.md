<!--
Sync Impact Report
- Version change: (unversioned template scaffold) → 1.0.0
- Bump rationale: Initial ratification. The previous file contained only unfilled
  placeholder tokens, so this is the first concrete governance definition (MAJOR baseline).
- Modified principles:
  - [PRINCIPLE_1_NAME] → I. First-Principles Learning
  - [PRINCIPLE_2_NAME] → II. Source-Code-First
  - [PRINCIPLE_3_NAME] → III. Experiment-Driven
  - [PRINCIPLE_4_NAME] → IV. Feynman Explanation (NON-NEGOTIABLE)
  - [PRINCIPLE_5_NAME] → V. Acceptance-Criteria-Driven
- Added sections:
  - Core Principles VI–XIV (Unsafe-Rust-Safety, no_std-Awareness, Linux-Kernel-Awareness,
    Performance-Is-Measured, Reproducibility, Incremental Complexity, Learn → Explain → Build,
    Knowledge Must Be Traceable, Final Capability)
  - Environment & Reproducibility Constraints
  - Learning Workflow & Quality Gates
  - Governance (filled)
- Removed sections: none
- Deferred TODOs: none
-->

# Rust + Aya/eBPF 高性能网络开发学习项目 Constitution

## Core Principles

### I. First-Principles Learning

所有核心技术知识 MUST 从底层机制开始解释，不得停留在 API、框架或概念描述层面。每个知识点
MUST 在下列链条上建立可陈述的因果关系：Rust → Compiler / ABI → Linux → Kernel → eBPF →
Aya → Network Stack → NIC。当某一环无法解释时，该知识点视为未完成，MUST 先补齐缺失环节
而不是继续向上层推进。

Rationale: 高性能网络问题的根因几乎总是出现在抽象层之下；只记住接口无法支撑排障与优化。

### II. Source-Code-First

所有核心技术 MUST 结合源码学习。对每个重要机制，学习者 MUST 能定位到实际源码中的关键结构体、
函数或调用路径，并在学习材料中记录文件路径与符号名。源码范围包括但不限于：Rust core / std、
Rust compiler / MIR（必要时）、Linux kernel、Linux networking subsystem、Linux BPF
subsystem、Aya、以及相关用户态工具。二手资料 MAY 作为入口，但 MUST NOT 作为唯一依据。

Rationale: 文档会过时且省略边界条件，源码是唯一权威且可验证的事实来源。

### III. Experiment-Driven

每个核心知识点 MUST 对应至少一个可运行实验。实验 SHOULD 优先采用 Rust example、Linux
command、eBPF program、Aya program、packet capture、tracing 或 benchmark 形式。实验
MUST 可被重复执行：命令、输入与预期输出 MUST 被记录，使他人（或未来的自己）能复现同样结果。

Rationale: 只有可执行的实验才能把"我以为的机制"与"实际发生的机制"区分开。

### IV. Feynman Explanation (NON-NEGOTIABLE)

每个重要知识模块 MUST 通过 Feynman Method 验收。学习者 MUST 能够：(1) 用自己的语言解释概念；
(2) 给出最小示例；(3) 解释底层机制；(4) 解释常见误区；(5) 回答验证性问题。任何一项无法清晰
完成，该知识点 MUST 标记为未完成，且 MUST NOT 计入阶段进度。

Rationale: 无法讲清楚等同于没有掌握；这是唯一能暴露自我欺骗式学习的检验方式。

### V. Acceptance-Criteria-Driven

任何学习目标 MUST 具有明确、可验证的 Acceptance Criteria。"看过"、"了解过"、"做过笔记"
MUST NOT 作为完成标准。合格的完成标准形如：能解释、能画图、能写代码、能运行实验、能分析输出、
能定位源码、能解释性能差异、能完成测试。缺少 Acceptance Criteria 的学习目标 MUST 在进入
Tasks 阶段前补全。

Rationale: 没有验收标准的学习目标无法判定完成，也无法在后续阶段被审查。

### VI. Unsafe-Rust-Safety

所有 unsafe Rust 代码 MUST 显式说明其 Safety Invariant，说明 MUST 覆盖相关的 raw
pointer、pointer arithmetic、alignment、aliasing、provenance、lifetime、memory
safety、FFI 与 undefined behavior 风险。"Rust 要求 unsafe" MUST NOT 作为解释。无法陈述
不变量的 unsafe 代码 MUST 被重写或删除。

Rationale: unsafe 块把编译器的证明义务转移给作者；未写下的不变量等于不存在的不变量。

### VII. no_std-Awareness

涉及 eBPF、内核或受限执行环境的 Rust 代码时，MUST 明确区分 core、alloc、std、no_std、
allocator、panic、runtime 与 OS services 各自的边界与可用性。MUST NOT 将 `#![no_std]`
简化为"不能使用标准库"，而 MUST 说明缺失的是哪一类运行时能力以及为何缺失。

Rationale: eBPF 与内核环境的约束来自运行时与分配器的缺席，理解边界才能预测编译与验证器失败。

### VIII. Linux-Kernel-Awareness

学习目标 MUST 最终连接到 Linux kernel 的实际执行路径。重点覆盖 syscall、scheduler、
memory、networking、socket、skb、NAPI、driver、XDP、TC 与 BPF subsystem。本项目不要求
成为 Linux kernel developer，但学习者 MUST 能够阅读并解释关键执行路径。

Rationale: 用户态观测到的现象由内核路径决定；不能读路径就只能猜测行为。

### IX. Performance-Is-Measured

所有"高性能"结论 MUST 通过实际测量验证。测量 MUST 覆盖与结论相关的指标，可包括 throughput、
packets per second、latency（p50 / p95 / p99）、CPU utilization、cycles per packet、
memory allocation、lock contention 与 packet drops。MUST NOT 仅依据文档、直觉或主观判断
得出性能结论；未测量的性能主张 MUST 标注为假设。

Rationale: 性能是环境相关的经验事实，只有测量能把优化与迷信区分开。

### X. Reproducibility

实验 MUST 可重复。项目 MUST 记录 Rust version、kernel version、architecture、
dependencies、commands、configuration 与 benchmark environment。任何重要实验 MUST 能够
按记录重新运行；无法重新运行的实验结果 MUST NOT 被引用为结论依据。

Rationale: 不可复现的结果无法被审查，也无法在环境变化后继续成立。

### XI. Incremental Complexity

学习 MUST 从简单机制逐步进入复杂系统，遵循总体路线：Rust → Linux → Networking → Unsafe
Rust → no_std → eBPF → BPF internals → Aya → XDP → TC → Socket → AF_XDP → Zero Copy →
Lock-Free → High Performance Networking。MUST NOT 在前置知识未通过验收的情况下直接跳到
复杂框架。路线顺序 MAY 调整，但跳过 MUST 在 Plan 中显式记录理由与补齐计划。

Rationale: 跳级学习会把缺失的基础转化为长期存在、难以定位的理解债务。

### XII. Learn → Explain → Build

每个阶段 MUST 形成完整闭环：Learn → Read Source → Experiment → Explain → Feynman
Tutorial → Acceptance Test → Build → Benchmark → Review。阶段 MUST NOT 在闭环缺环的
情况下被标记为完成。学习计划 MUST 被视为可验证能力成长的工程，而非课程目录。

Rationale: 闭环把输入型学习转换为可检验的产出，使进度具备真实含义。

### XIII. Knowledge Must Be Traceable

每个重要学习目标 MUST 可追踪到完整链条：Spec → Plan → Task → Learning Material →
Experiment → Source Code → Acceptance Criteria。MUST NOT 产生无法追溯来源与目标的孤立
学习笔记；孤立笔记 MUST 在下一次 Review 中并入链条或删除。

Rationale: 可追踪性使知识可审查、可更新，并防止学习材料随时间腐化为无主碎片。

### XIV. Final Capability

项目最终目标 MUST NOT 被降级为"学会 Aya API"。学习者最终 MUST 能够独立设计、实现、调试并
优化基于 Rust + Aya/eBPF 的高性能 Linux 网络程序。最终能力 MUST 至少覆盖：Rust systems
programming、Unsafe Rust、Linux networking、eBPF、Aya、XDP、TC、BPF maps、BPF
verifier、packet parsing、zero-copy、AF_XDP、lock-free programming、kernel source
reading、network performance analysis 与 benchmarking。

Rationale: 明确的终态能力清单让每个阶段的取舍都有可对照的判据。

## Environment & Reproducibility Constraints

技术栈固定为 Rust（含 no_std 子集）+ Aya + Linux eBPF；实验 MUST 在 Linux 环境执行。

每个包含实验或 benchmark 的产出 MUST 附带环境记录，至少包括：Rust toolchain 版本、
kernel 版本（`uname -r`）、CPU 架构、相关 crate 版本与执行命令。

Benchmark MUST 记录测量方法与运行次数；单次运行结果 MUST NOT 作为性能结论。

涉及内核加载、网卡配置或流量注入的实验 MUST 记录所需权限与清理步骤，使环境可恢复到实验前状态。

## Learning Workflow & Quality Gates

工作流阶段顺序为 Spec → Plan → Tasks → Implement → Converge，各阶段 MUST 遵守以下门禁：

- **Spec gate**: 每个学习目标 MUST 带有 Acceptance Criteria（Principle V），并声明其在
  Incremental Complexity 路线上的位置（Principle XI）。
- **Plan gate**: Plan MUST 为每个目标指明源码定位范围（Principle II）与至少一个可运行实验
  （Principle III）；涉及 unsafe 或 no_std 的目标 MUST 标注 Principle VI / VII 适用。
- **Tasks gate**: 任务 MUST 可追踪到 Spec 条目并落在 Principle XIII 的链条上。
- **Implement gate**: 产出 MUST 包含可复现命令与实际输出；性能类任务 MUST 附测量数据
  （Principle IX、X）。
- **Review / Converge gate**: 知识模块 MUST 通过 Feynman 五项检验（Principle IV）方可
  标记完成；未通过项 MUST 作为新任务回写 tasks.md。

## Governance

本 Constitution 约束本项目后续所有 Spec、Plan、Tasks、Implementation 与 Convergence
阶段，并优先于临时约定。

任何违反 MUST 原则的设计 MUST 重新讨论并修订 Constitution 或修改设计，MUST NOT 通过实现
阶段静默绕过。已知的原则偏离 MUST 在对应 Plan 中显式记录理由与补救计划。

学习范围 MAY 随理解深入而演进，但核心学习目标、Acceptance Criteria 与实验可重复性 MUST
保持可追踪。

修订流程：修订 MUST 通过 `/speckit-constitution` 执行，MUST 附带 Sync Impact Report，并
按语义化版本递增——MAJOR 用于原则移除或不兼容重定义，MINOR 用于新增原则或实质性扩充，PATCH
用于澄清与措辞修正。

合规审查：每个阶段完成时 MUST 对照 Learning Workflow & Quality Gates 逐条核对；未通过的
门禁 MUST 阻塞该阶段完成，而不是记为待办后放行。

**Version**: 1.0.0 | **Ratified**: 2026-09-03 | **Last Amended**: 2026-09-03
