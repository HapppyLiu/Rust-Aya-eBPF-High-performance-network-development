# Instructions

使用命令/speckit-constitution 和 以下内容初始化 constitution
/speckit-constitution 
# Rust + Aya/eBPF 高性能网络开发学习项目 Constitution

## 1. First-Principles Learning

所有核心技术知识必须从底层机制开始解释。

学习不能停留在 API、框架或概念描述层面，必须尽可能建立：

Rust
→ Compiler / ABI
→ Linux
→ Kernel
→ eBPF
→ Aya
→ Network Stack
→ NIC

之间的因果关系。

---

## 2. Source-Code-First

所有核心技术必须结合源码学习。

源码范围包括但不限于：

* Rust core / std
* Rust compiler / MIR（必要时）
* Linux kernel
* Linux networking subsystem
* Linux BPF subsystem
* Aya
* 相关用户态工具

对于重要机制，必须能够定位到实际源码中的关键结构、函数或调用路径。

---

## 3. Experiment-Driven

每一个核心知识点必须尽可能对应一个可运行实验。

实验应优先采用：

* Rust example
* Linux command
* eBPF program
* Aya program
* packet capture
* tracing
* benchmark

实验必须能够被重复执行。

---

## 4. Feynman Learning

每个重要知识模块必须能够通过 Feynman Method 进行解释。

学习者必须能够：

1. 用自己的语言解释概念；
2. 给出最小示例；
3. 解释底层机制；
4. 解释常见误区；
5. 回答验证性问题。

如果无法清晰解释，则视为该知识点尚未完成。

---

## 5. Acceptance-Criteria-Driven

任何学习目标必须具有明确的 Acceptance Criteria。

“看过”、“了解过”、“做过笔记”不能作为完成标准。

完成标准应该是可验证的，例如：

* 能解释；
* 能画图；
* 能写代码；
* 能运行实验；
* 能分析输出；
* 能定位源码；
* 能解释性能差异；
* 能完成测试。

---

## 6. Unsafe-Rust-Safety

所有 unsafe Rust 代码必须说明其 Safety Invariant。

必须关注：

* raw pointer
* pointer arithmetic
* alignment
* aliasing
* provenance
* lifetime
* memory safety
* FFI
* undefined behavior

不得仅以“Rust 要求 unsafe”作为解释。

---

## 7. no_std-Awareness

涉及 eBPF、内核或受限执行环境的 Rust 代码时，必须明确区分：

* core
* alloc
* std
* no_std
* allocator
* panic
* runtime
* OS services

不能将 `#![no_std]` 简化为“不能使用标准库”。

---

## 8. Linux-Kernel-Awareness

学习目标最终必须能够连接到 Linux kernel 的实际执行路径。

重点理解：

* syscall
* scheduler
* memory
* networking
* socket
* skb
* NAPI
* driver
* XDP
* TC
* BPF subsystem

不要求成为 Linux kernel developer，但必须能够阅读和解释关键执行路径。

---

## 9. Performance-Is-Measured

所有“高性能”结论必须尽可能通过实际测量验证。

关注：

* throughput
* packets per second
* latency
* p50 / p95 / p99
* CPU utilization
* cycles per packet
* memory allocation
* lock contention
* packet drops

不能仅依据文档或主观判断得出性能结论。

---

## 10. Reproducibility

实验必须尽可能做到可重复。

项目应该记录：

* Rust version
* kernel version
* architecture
* dependencies
* commands
* configuration
* benchmark environment

任何重要实验都应该能够重新运行。

---

## 11. Incremental Complexity

学习必须从简单机制逐步进入复杂系统。

总体路线：

Rust
→ Linux
→ Networking
→ Unsafe Rust
→ no_std
→ eBPF
→ BPF internals
→ Aya
→ XDP
→ TC
→ Socket
→ AF_XDP
→ Zero Copy
→ Lock-Free
→ High Performance Networking

不得在缺少基础知识的情况下直接跳到复杂框架。

---

## 12. Learn → Explain → Build

每个阶段必须形成以下闭环：

Learn
→ Read Source
→ Experiment
→ Explain
→ Feynman Tutorial
→ Acceptance Test
→ Build
→ Benchmark
→ Review

学习计划不是课程目录，而是一个可以验证能力成长的工程。

---

## 13. Knowledge Must Be Traceable

每个重要学习目标必须能够追踪到：

Spec
→ Plan
→ Task
→ Learning Material
→ Experiment
→ Source Code
→ Acceptance Criteria

避免产生无法验证来源和目标的孤立学习笔记。

---

## 14. Final Capability

项目最终目标不是“学会 Aya API”。

学习者最终必须能够独立设计、实现、调试和优化基于 Rust + Aya/eBPF 的高性能 Linux 网络程序。

最终能力至少包括：

* Rust systems programming
* Unsafe Rust
* Linux networking
* eBPF
* Aya
* XDP
* TC
* BPF maps
* BPF verifier
* packet parsing
* zero-copy
* AF_XDP
* lock-free programming
* kernel source reading
* network performance analysis
* benchmarking

---

## Governance

以上原则是本项目后续 Spec、Plan、Tasks、Implementation 和 Convergence 的约束。

任何违反 MUST 原则的设计都必须重新讨论，而不是通过实现阶段绕过。

学习范围可以演进，但核心学习目标、验收标准和实验可重复性必须保持可追踪。

