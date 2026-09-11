# Module 7: no_std、运行时边界与分配器

**Story**: US7（P1，Feature 002 硬前置） | **Capabilities**: C-22…C-24 | **Prerequisite**: m6（accepted）

> 本文件属于 **Answer Track**。对应的 Learner Track 是
> [`learner/m7-nostd/guide.md`](../../learner/m7-nostd/guide.md)。
> 如果你还没做过那边的预测表，先去做 —— 这份文件读过之后就不能再自测了。

## 这个模块回答什么问题

1. `#![no_std]` 关掉的是哪一类默认依赖？为什么 `Option`、切片、迭代器往往还在？
2. `std::vec::Vec` 的实现写在哪一层？`String` 和 `File` 是不是同一层的东西？
3. `extern crate alloc` 之后 `Vec` 就能用了吗？分配能力由谁提供？
4. 没有操作系统时，panic 必须由谁接住？强制 unwind 时缺的又是什么？
5. 产物不能在本机执行，Independent Test 还过吗？本模块把"可运行"定义成什么？

---

## 七类边界（FR-009 / CHK039）

判定"缺的是哪一层"时，MUST 使用下列定义。它们互斥且穷举本模块涉及的运行时角色：

| 层 | 一句话 | 有它时你得到什么 | 没有它时典型失败 |
|----|--------|----------------|----------------|
| **core** | 语言与可移植原语。不知道堆，不谈 I/O，不绑某个内核。 | `Option`、切片、整数、`Iterator`、原子、`fmt` 的无分配部分 | 几乎不会在本模块单独缺 —— 裸机 sysroot 带着 `libcore` |
| **alloc** | 会调用堆的集合与智能指针。**不提供**那块堆。 | `Vec`、`Box`、`String`、`Rc`/`Arc` 的类型与算法 | `cannot find module or crate alloc`（未 `extern crate alloc`） |
| **std** | 再导出 core+alloc，并加上需要内核的服务。 | 文件、线程、环境、网络、默认分配器、可打印的 panic | `can't find crate for std`（`x86_64-unknown-none` 无此 crate） |
| **allocator** | 真正拿出内存的 `GlobalAlloc` 实现。 | `__rust_alloc` 有去处；`Vec::push` 能链接 | `no global memory allocator found` |
| **panic** | 发散路径的漏斗：`#[panic_handler]` → `panic_impl`。必须永不返回。 | 语言允许 `panic!` / 越界检查编译下去 | `` `#[panic_handler]` function required, but not found `` |
| **runtime** | 编译器要的语言项与启动约定，不是你 `use` 的日常 API。 | `_start`、abort 策略、展开人格 | 强制 unwind 时：`unwinding panics are not supported without std` |
| **OS services** | 必须有内核（或等价物）才能实现的能力：文件、进程、系统调用、libc 启动。 | `std::fs::File`、`std::env`、`__libc_start_main` | `core` 里没有 `fs`；裸机产物没有 libc 启动例程 |

`#![no_std]` **不是**"不能使用标准库"。它的意思是：不要自动链接默认的 `std` crate 及其 OS 运行时。
你仍然使用 **core**（std 自己也是用 core 做成的）；可以再链 **alloc**；std 源码本身写着 `#![no_std]`，
因为"我们自己就是 std"（见 source-refs）。

---

## 概念

### C-22 no_std

- **一句话定义**：`#![no_std]` 取消默认的 `std` crate 与它所依赖的操作系统运行时；
  剩下的是 core 能提供的语言原语，外加你必须自己接上的语言项。

- **底层机制**：
  普通二进制 crate 在展开时相当于 `extern crate std`。`std` 再导出 core 与 alloc，
  并链接 panic 运行时、默认分配器、以及 libc/`_start` 那一套。
  `#![no_std]` 关掉的是这一整包默认依赖，不是把 `std::` 三个字从语言里删掉。

  `core` 自己用的是 `#![no_core]`（它是更底下的那一层，不能再依赖 core）。
  文档点名消费方要提供：内存例程（通常 `compiler_builtins` 给）、**panic handler**、
  以及 unwind 用的 `eh_personality`（abort 策略下不会走到）。

  `x86_64-unknown-none` 是官方裸机 target：无 OS、无 std、默认 panic=abort。
  产物没有 crt0，入口是我们导出的 `_start`。没有可返回的操作系统，所以本 Story 的
  "可运行"= **构建成功 + 静态检查**，不是本机执行（US7 Independent Test / CHK036）。

  编译期断言：`build.rs` 拒绝非 `x86_64-unknown-none` 的 `TARGET`；
  `c22_nostd.rs` 在 `not(target_os = "none")` 时 `compile_error!`，并用 `const` 断言
  指针宽度为 8、`Option` 在 core 里可用。

- **常见误解**：`no_std` = 不能用标准库
  → **实际**：关掉的是默认 crate 与 OS 运行时；core 仍在，std 自己也是 `#![no_std]`
  → **证据**：`source-refs.md` `std/src/lib.rs:237`；`c22_nostd::core_option_is_available`；
    OBSERVATIONS「C-22 / 默认构建」

- **对应实验**：`experiments/m7-nostd/src/c22_nostd.rs` + `build.rs` +
  `tools/check-nostd-artifact.sh`

### C-23 core / alloc / std

- **一句话定义**：三层 crate 叠在一起：core 是原语，alloc 是堆上的集合（不问堆从哪来），
  std 再导出前两层并补上 OS services。

- **底层机制**：
  **同一段求和，三层版本**（`c23_core_alloc_std.rs`）：

  1. `sum_core`：只扫 `&[i32]`。默认裸机构建成功。证明"不用 std 也能算数"。
  2. `sum_alloc`：把输入 `push` 进 `Vec` 再求和。默认构建同样成功 —— 因为本 crate
     `extern crate alloc` **并且** 注册了 bump `#[global_allocator]`。
     `Vec` 的实现在 `alloc`，`std::vec::Vec` 只是再导出（`std/src/lib.rs:607`）。
  3. `sum_std`（探测 2a）：写 `core::fs::File`。`core` 没有 `fs` 模块（E0433）。
     文件系统是 OS services，不是 core 忘了写的 API。

  探测 2b 去掉 `#![no_std]`：rustc 按默认去拉 `std`，裸机 target 报 E0463
  `can't find crate for std`。级联还会再要 panic handler（没有 std 就没有默认处理）。
  计入 SC-006 分母的是 **E0463**。

  `String` 在 alloc（堆上的字节串）；`File` 在 std（打开一个内核对象）。
  两者失败不能写成同一层。

- **常见误解**：`std::vec::Vec` 所以 Vec 实现写在 std 里
  → **实际**：路径是再导出，实现在 alloc
  → **证据**：`source-refs.md` `alloc/src/lib.rs:1-9,74-75` 与 `std/src/lib.rs:464,607`；
    默认构建里 `sum_alloc` 并不链接 std

- **对应实验**：`experiments/m7-nostd/src/c23_core_alloc_std.rs` +
  `tools/m7-probe-errors.sh 2a 2b`

### C-24 Panic and allocator fundamentals

- **一句话定义**：panic 在无 OS 时必须由你提供永不返回的处理函数；堆分配由
  `GlobalAlloc` 实现提供，不是 alloc crate 自带的一块堆。

- **底层机制**：

  **panic**。`core/src/panicking.rs` 写明：core 不能定义 handler，只能调用 `panic_impl`；
  该语言项由 `#[panic_handler]` 展开出来。我们的处理函数是 `loop { spin_loop() }` ——
  没有 stderr 可写，也不能返回。有 OS 时 std 的 handler 可以打印、展开、abort；
  没有 OS 时展开运行时也不存在。

  探测 1a：拿掉 handler → `` `#[panic_handler]` function required, but not found ``。
  归属：panic / runtime（语言项，不是 core 里缺了一段库代码）。

  探测 1b：保留 handler，强制 `-C panic=unwind`。1.98.0 在本 target 上**不会**索要
  `eh_personality`，而是直接拒绝：`unwinding panics are not supported without std`。
  归属：runtime（展开运行时由 std / panic_unwind 供给）。spec SC-006 已按实测修订，分母仍为 6。

  **allocator**。`alloc/src/alloc.rs` 通过 `__rust_alloc` 调用全局分配器：有
  `#[global_allocator]` 时 rustc 为你生成跳板；没有时，在 std 里走默认 libc malloc，
  在纯 no_std 里没有默认。探测 3b：`no global memory allocator found but one is required`。
  1.98.0 不再强制 `#[alloc_error_handler]`（OOM 默认 panic）。

  本 crate 用静态 64KiB 数组做 bump：`alloc` 按对齐向上取整，CAS 提交区间，`dealloc` 空操作。
  默认构建里 `Vec::push` 能链接，是因为 **BumpAlloc 被注册为全局分配器**，不是 alloc 自带堆。

  **eBPF 前提（US7 AS2，FR-014，MUST NOT 写 eBPF 程序）**：
  bump 依赖一块可写、可按字节切的 arena。经典 eBPF 没有通用堆，也没有与 `GlobalAlloc`
  对等的内核分配入口；verifier 还禁止任意动态分配。因此"我在裸机 crate 里让 Vec 链上了"
  **不能**推广成"eBPF 里也能 Vec"。用户态 Aya 程序用的是 host 的 std/分配器，那是另一侧。

  **host 侧 Miri（T117）**：裸机产物不能跑 Miri。`harness/tests/c24_bump_alloc.rs` 经
  `#[path]` 引入同一份 `bump.rs`，在 host 上直接调 `alloc`/`dealloc`。
  `cargo +nightly miri test -p rf-harness --test c24_bump_alloc` 三测通过，
  `ub_verdict = clean`（host 侧分配器实现）。这不能证明裸机产物在真实 CPU 上无 UB ——
  范围仅限被测的 `BumpAlloc` 方法。

- **常见误解**：`extern crate alloc` 之后就有堆了
  → **实际**：alloc 只会**调用**分配器；没有 `#[global_allocator]` 就链接失败
  → **证据**：`tools/m7-probe-errors.sh 3b`；`source-refs.md` `alloc/src/alloc.rs:12-22`

- **对应实验**：`experiments/m7-nostd/src/c24_panic_alloc.rs`、`src/bump.rs`、
  `harness/tests/c24_bump_alloc.rs`

---

## 与后续学习的关联            <!-- REQUIRED，FR-014 -->

| C-ID | 与 Linux / eBPF / Aya 的关联 |
|------|------------------------------|
| C-22 | eBPF 程序侧同样没有 std、没有 libc 启动；Aya 的 BPF crate 是 `no_std` 风格。本模块用裸机 target 训练的是"缺的是哪类运行时"，不是编写 eBPF（FR-017）。 |
| C-23 | 经典 eBPF 既没有 alloc 也没有 std；内核 helper 是 OS services 的类比。用户态 Aya 用 std。能说出一次编译失败属于哪一层，才能读懂后续的 crate 特征开关。 |
| C-24 | eBPF 没有通用堆，也没有 panic unwind。本模块的 bump 在那种环境里前提不成立 —— 这不是"再抄一份分配器就行"。用户态仍用 host 分配器。 |

不要把"裸机 Vec 链接成功"读成"内核 eBPF 可以动态分配"。那是本模块最贵的那句边界。
