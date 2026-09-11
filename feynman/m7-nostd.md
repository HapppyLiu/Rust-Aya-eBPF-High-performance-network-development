# Feynman: Module 7 — no_std、运行时边界与分配器

**Capabilities covered**: C-22, C-23, C-24

---

## 1. 用自己的话解释

面向"没学过 Rust 但懂 C"的同事。

C 里你写 `gcc -nostdlib -ffreestanding`，不是"C 语言被砍掉一半"，而是
**别再自动链 libc、别再假定有 `printf` 和 `malloc`**。语言还在：整数、指针、结构体。
Rust 的 `#![no_std]` 是同一类开关。关掉的是默认那个叫 `std` 的 crate，以及它背后的
操作系统服务：文件、线程、环境变量、libc 启动。

`Option`、切片、for 循环用的迭代器还在，因为它们住在更底下的 **core**：
不谈堆，不谈内核，只谈语言自己的积木。core 的源码甚至写的是 `#![no_core]` ——
它不能再依赖自己。

堆上的 `Vec` 不在 core。它住在 **alloc**：一套会**打电话给分配器**的算法。
alloc 不自带一块堆。电话打给谁？一个实现了 `GlobalAlloc` 的静态变量。
普通 Linux 程序里，std 把这个电话接到 libc 的 `malloc`。裸机没有 libc，
你就得自己接 —— 我们接在一块静态数组的 bump 上。所以：
`extern crate alloc` 只是请来了会打电话的人；**真正给内存的是你注册的分配器**。

`std` 自己的源码第一页就写着 `#![no_std]`，旁边一行注释：别再链一份 std，
我们自己就是 std。它把 core 和 alloc 再导出去，再补上文件和线程。
所以 `std::vec::Vec` 这个名字是路牌，不是施工队。施工队在 alloc。

panic 在有操作系统时可以打印、展开栈、abort。没有操作系统时，编译器仍要求
有一个**永不返回**的处理函数。拿掉它，报的不是"缺库"，是缺语言项。
你若强行要求展开（unwind），这个裸机 target 会直接说：没有 std 就不支持展开。
那是运行时，不是 core 少写了一个函数。

这个模块的产物不能在本机跑。没有 `_start` 之外的世界可返回。验收看的是
**编得过、符号对、错误能归属到那七层之一**。把 `no_std` 说成"不能用标准库"，
本模块判不及格 —— 连 std 自己都写着 `no_std`。

---

## 2. 最小示例

### C-22 no_std

```rust
#![no_std]
#![no_main]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop { core::hint::spin_loop() } }

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! { loop { core::hint::spin_loop() } }
```

完整观察：[`src/main.rs`](../experiments/m7-nostd/src/main.rs)、
[`src/c22_nostd.rs`](../experiments/m7-nostd/src/c22_nostd.rs)

### C-23 core / alloc / std

```rust
fn sum_core(xs: &[i32]) -> i32 { /* 只扫切片 */ 0 }
fn sum_alloc(xs: &[i32]) -> i32 {
    let mut v = alloc::vec::Vec::new();
    for &x in xs { v.push(x); }
    sum_core(&v)
}
// File 不在 core：let _f: core::fs::File;  // E0433
```

完整观察：[`src/c23_core_alloc_std.rs`](../experiments/m7-nostd/src/c23_core_alloc_std.rs)

### C-24 Panic & allocator

```rust
#[global_allocator]
static ALLOC: BumpAlloc = BumpAlloc::new(); // 静态数组 bump，见 src/bump.rs

fn demo() { let mut v = alloc::vec::Vec::new(); v.push(1u8); }
```

完整观察：[`src/c24_panic_alloc.rs`](../experiments/m7-nostd/src/c24_panic_alloc.rs)、
[`src/bump.rs`](../experiments/m7-nostd/src/bump.rs)

---

## 3. 底层机制

每条论断带本模块的断言名、脚本退出码或源码符号。

1. **`no_std` 关掉默认 crate，不是关掉语言。**
   依据：`c22_nostd.rs` 的 `core_option_is_available`（`const` 断言 `Option` 可用）；
   `std/src/lib.rs:237` `#![no_std]`；OBSERVATIONS「C-22 / 默认构建」。

2. **core 比 `no_std` 更底。**
   依据：`core/src/lib.rs:64` `#![no_core]`；文档 `:12-14` 声明不知道堆。

3. **Vec 实现在 alloc，std 再导出。**
   依据：`alloc/src/lib.rs:74-75` `needs_allocator`；`std/src/lib.rs:464,607`；
   默认构建中 `sum_alloc` 成功且未链接 std。

4. **`File` 是 OS services，不是 core 漏写的类型。**
   依据：`tools/m7-probe-errors.sh 2a` 退出码 0（探测按设计失败并命中 E0433
   `cannot find fs in core`）；OBSERVATIONS「C-23 / 探测 2a」。

5. **裸机 target 没有 std crate。**
   依据：`tools/m7-probe-errors.sh 2b` 命中 E0463；OBSERVATIONS「C-23 / 探测 2b」。

6. **panic 处理是语言项。**
   依据：`core/src/panicking.rs:10-22,67-69`；`tools/m7-probe-errors.sh 1a`。

7. **unwind 是 runtime，1.98.0 上不表现为单独缺 eh_personality。**
   依据：`tools/m7-probe-errors.sh 1b` 命中 `unwinding panics are not supported without std`；
   spec SC-006 修订说明。

8. **堆由 GlobalAlloc 提供。**
   依据：`alloc/src/alloc.rs:12-22`；`tools/m7-probe-errors.sh 3a`（缺 crate）与
   `3b`（缺分配器）两条必须分开；`vec_push_demo` 在默认构建中链接成功。

9. **产物不是 Linux 用户态程序。**
   依据：`tools/check-nostd-artifact.sh` 退出码 0：有 `_start`、无 `__libc_start_main`、
   无 `eh_personality`。

10. **host Miri clean 只覆盖 bump 方法。**
    依据：`bump_alloc_aligned_roundtrip` 等三条；OBSERVATIONS「C-24 / host 侧 Miri」。

---

## 4. 常见误区

- **误解**：`no_std` = 不能用标准库
  → **实际**：关掉默认 `std` crate 与 OS 运行时；core 仍在；std 自己也写着 `#![no_std]`
  → **证据**：`std/src/lib.rs:237`；`core_option_is_available`；本节强制收录此条（T120）

- **误解**：`std::vec::Vec` 所以实现在 std
  → **实际**：再导出；实现在 alloc
  → **证据**：`std/src/lib.rs:607`；默认构建的 `sum_alloc`

- **误解**：`extern crate alloc` 之后就有堆
  → **实际**：还缺 `#[global_allocator]`
  → **证据**：`tools/m7-probe-errors.sh 3b`

- **误解**：产物不能执行 = 实验失败
  → **实际**：本 Story 的"可运行"= 构建成功 + 静态检查
  → **证据**：spec US7 Independent Test；`tools/check-nostd-artifact.sh`

- **误解**：裸机 bump 能搬进 eBPF 当通用堆
  → **实际**：eBPF 没有这块可写 arena，也没有对等的分配入口
  → **证据**：`learning/m7-nostd/concept.md`「eBPF 前提」；OBSERVATIONS「C-24 / Vec 链接成功」的"不能证明"

- **误解**：强制 unwind 会看到 `eh_personality` 缺失
  → **实际**：1.98.0 在本 target 上直接拒绝 unwind without std
  → **证据**：探测 1b；spec SC-006 修订

---

## 5. 验证性问题

每题的回答指向一条断言、一处源码或一个观测块。

1. 产物不能在本机执行。Independent Test 还过吗？本模块把"可运行"定义成什么？
   → spec US7 Independent Test；`cd experiments/m7-nostd && cargo build` 退出码 0；
     `tools/check-nostd-artifact.sh` 退出码 0。

2. 一次构建里同时拿掉 panic 处理、引入 File、再写 Vec。分母能不能用"本次 stderr 有几条"？
   → 不能。SC-006 分母固定为 6，每步单独构建（`tools/m7-probe-errors.sh`）。
     rustc 会提前中止或追加级联（见探测 2b 的四条错误只计 E0463）。

3. `String` 失败和 `File` 失败能写成同一层吗？
   → 不能。`String` 在 alloc；`File` 是 OS services。依据：探测 2a（`core::fs` E0433）
     与 3a（crate `alloc` E0433）不是同一层；`source-refs` alloc 文档 vs core 无 fs。

4. 已经 `extern crate alloc` 仍然链接失败。下一步找谁？
   → 找有没有 `#[global_allocator]`。依据：探测 3b；`alloc/src/alloc.rs:12-22`。

5. 把 bump 搬进没有通用堆的受限环境，缺的是哪类前提？
   → 缺一块可写、可按字节切的 arena / 对等分配入口。依据：concept.md「eBPF 前提」；
     OBSERVATIONS「C-24 / Vec 链接成功」的证据边界。

6. `nm` 里没有 `__libc_start_main`，能不能推出"所以这是 Linux 用户态程序"？
   → 不能。脚本断言的是**没有** libc 启动例程。依据：`tools/check-nostd-artifact.sh`；
     OBSERVATIONS「C-22 / 产物静态检查」。

---

## 检验结果

| # | 小节 | 结果 | 依据 |
|---|------|------|------|
| 1 | 用自己的话解释 | pass | 面向懂 C 的同事；`no_std`/`Vec`/`GlobalAlloc` 均用 C 世界类比当场解释 |
| 2 | 最小示例 | pass | C-22/C-23/C-24 各 ≤15 行，链到 m7 源码 |
| 3 | 底层机制 | pass | 10 条均指向脚本退出码、`const` 断言或 source-refs 符号 |
| 4 | 常见误区 | pass | 含强制条「no_std = 不能用标准库」；每条有证据 |
| 5 | 验证性问题 | pass | 6 题，每题指向断言 / 源码 / 观测块 |

**模块 Feynman 五项：合取通过。**
