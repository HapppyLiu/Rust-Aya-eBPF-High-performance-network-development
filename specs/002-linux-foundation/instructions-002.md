# Instructions
#创建第二个 Feature
/speckit-specify 
设计本学习工程的第二个feature: 002-linux-foundation。

目标是为《Rust + Aya/eBPF 高性能网络开发学习计划》建立 Linux Foundation。

学习者完成本阶段后必须能够：

1. 熟练使用 Linux CLI 和常见诊断工具。
2. 理解 Linux 用户态与内核态。
3. 理解进程、线程、task_struct、上下文切换。
4. 理解 Linux system call 的基本执行路径。
5. 理解虚拟内存、页表、Page、TLB、Page Fault、mmap。
6. 理解 Linux 文件描述符、VFS、inode、file、open/read/write。
7. 理解 Linux I/O 模型。
8. 理解 scheduler 的基本机制。
9. 理解 Linux IPC 和基础同步机制。
10. 理解 Linux 网络栈基本结构。
11. 理解 socket、TCP/IP、NIC、network namespace。
12. 能够阅读相关 Linux kernel 源码。
13. 能够使用 strace、perf、gdb、bpftrace 等工具观察系统行为。
14. 能够通过实验验证 Linux 的关键机制。
15. 能够使用 Rust 编写部分 Linux 系统级实验程序。

本阶段不深入 eBPF 编程和 Aya。
但每个 Linux 知识点都必须标注其未来与 eBPF/Aya/高性能网络的关系。

采用双轨学习模型：

Track A：
Knowledge Track
- Concepts
- Architecture
- Kernel source
- Feynman questions
- Quiz

Track B：
Experiment Track
- Linux commands
- C/Rust programs
- strace
- perf
- gdb
- bpftrace
- network tools

每个 Module 必须同时生成：

1. Learning Framework
2. Teaching Material
3. Feynman Questions
4. Experiment Tasks
5. Quiz
6. Answer Key
7. Acceptance Criteria

学习框架和答案必须分离。
学习者应该能够只打开 Learning Framework 完成自测，
然后再打开 Answer Key 进行纠错。
目录结构可以参考第一个feature，并将对应的结果放在第二个feature对应的目录 002-linux-foundation下。


----------------------------------------
执行 /speckit-clarify 
/speckit-clarify 
请找出：
哪里定义不清楚？
哪里可能产生歧义？
哪些知识深度不明确？
哪些实验无法验证？
哪些内容超出 Linux Foundation？

尤其要检查：
Linux Kernel version
实验环境
Rust version
是否需要 C
是否使用虚拟机
是否允许 Docker
是否需要 root
perf 是否可用
bpftrace 是否可用
网络实验是否需要 network namespace

实验环境明确为：

Ubuntu Linux
x86_64
Linux Kernel 6.x
Rust stable
gcc/clang
gdb
strace
perf
bpftrace
iproute2
tcpdump


----------------------------------------------------------------------
执行 /speckit-plan 
按照双轨学习模型生成

每一个核心知识模块都应该能够被独立验证。

实验应该尽可能小、独立、可运行、可观察。

---------------------------------------------------------------------------------
/speckit-checklist 

每执行这一步

----------------------------------------------------------------------------------
/speckit-tasks 


----------------------------------------------------------------------------------------------------------------

/speckit-implement 


---------------------------------------------------------------------------

/speckit-implement 
开始实现 US2（m2-types，C-05…C-07）


----------------------------------------------------------------------------------
/speckit-implement 
US3（m3-composition，C-08…C-11），同时在learning 中生成Answer Track对应的答案。

由 Grok4.6 生成。

----------------------------------------------------------------------------------
/speckit-implement 
实现 US4（m4-concurrency）

-------------------------------------------------------------------------------------------
/speckit-implement 
实现US5（m5-unsafe）

-------------------------------------------------------------------------------------------
/speckit-implement 
US6（m6-ffi）

----------------------------------------------------------------------------------------------
/speckit-implement 
实现 US7（m7-nostd）
----------------------------------------------------------------------------------------------
/speckit-implement 
实现 US8（m8-capstone）
----------------------------------------------------------------------------------------------


