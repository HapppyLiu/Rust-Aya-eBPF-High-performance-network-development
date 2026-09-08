# Module 4 源码引用

**Story**: US4 | **Capabilities**: C-12…C-14 | **依据**: FR-005 / SC-003 /
[learning-artifact-contract §B](../../specs/001-rust-foundation/contracts/learning-artifact-contract.md)

路径根：`$(rustc --print sysroot)/lib/rustlib/src/rust/library/`
本机为 `/root/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/`

行号在 pinned 工具链 **1.98.0**（`88d9e12ae 2026-08-18`）下固定，因此可被记录并复核（规则 B1）。

> 计划里的 `std/src/sync/mutex.rs` 在 1.98.0 已拆到 `std/src/sync/poison/mutex.rs`
> （`std/src/sync/mod.rs:235` 再导出历史默认的带 poison 版本）。本表记录**实际路径**。

---

## 引用表

| C-ID | 路径（相对 `library/`） | 符号 | 行 | kind | 这段源码回答了什么 |
|------|----------------------|------|----|------|------------------|
| C-12 | `core/src/marker.rs` | `unsafe auto trait Send` | 92 | library | `Send` 方法体为空；约束在推导与 impl |
| C-12 | `core/src/marker.rs` | `impl !Send for *const T` / `*mut T` | 97 / 99 | library | 裸指针默认拒绝跨线程移动 |
| C-12 | `core/src/marker.rs` | `unsafe impl Send for &T` | 105 | library | `&T: Send` 要求 `T: Sync`（`T: Sync ⟺ &T: Send`） |
| C-12 | `core/src/marker.rs` | `unsafe auto trait Sync` | 657 | library | `Sync` 同样是空方法体的 auto trait |
| C-12 | `core/src/marker.rs` | `impl !Sync for *const T` / `*mut T` | 672 / 674 | library | 裸指针默认拒绝跨线程共享 |
| C-12 | `core/src/cell.rs` | `unsafe impl Send for Cell` | 317 | library | `T: Send` 则 `Cell` 可移动 |
| C-12 | `core/src/cell.rs` | `impl !Sync for Cell` | 325 | library | `Cell` 显式禁止共享；注释承认来自 `UnsafeCell` |
| C-12 | `core/src/cell.rs` | `impl !Sync for UnsafeCell` | 2328 | library | 内部可变性的根：默认 `!Sync` |
| C-13 | `std/src/thread/mod.rs` | `pub use functions::spawn` / `scoped::scope` | 196 / 205 | library | 两个创建入口从这里再导出 |
| C-13 | `std/src/thread/functions.rs` | `pub fn spawn` | 125 / 128 | library | `F: Send + 'static`：标记 + 寿命两问 |
| C-13 | `std/src/thread/scoped.rs` | `pub fn scope` | 141 | library | 寿命收到 scope 结束 |
| C-13 | `std/src/thread/scoped.rs` | `Scope::spawn` | 201 / 203 | library | `F: Send + 'scope`：标记还在 |
| C-13 | `std/src/sync/mod.rs` | `pub use poison::{Mutex, MutexGuard}` | 235 | library | 历史默认的 `std::sync::Mutex` 来自 poison |
| C-13 | `std/src/sync/poison/mutex.rs` | `pub struct Mutex` | 227 | library | 系统锁 + poison 旗标 + `UnsafeCell<T>` |
| C-13 | `std/src/sync/poison/mutex.rs` | `unsafe impl Send for Mutex` | 238 | library | `T: Send` |
| C-13 | `std/src/sync/poison/mutex.rs` | `unsafe impl Sync for Mutex` | 257 | library | **`T: Send`**，不是 `T: Sync` —— 提升发生在这里 |
| C-13 | `std/src/sync/poison/mutex.rs` | `impl !Send for MutexGuard` | 289 | library | POSIX：解锁必须在加锁的同一线程 |
| C-13 | `std/src/sync/poison/mutex.rs` | `unsafe impl Sync for MutexGuard` | 294 | library | `&Guard` 只能读 `T`，不能 `Drop` |
| C-14 | `core/src/sync/atomic.rs` | `pub struct Atomic<T>` | 361 | library | 内部 `UnsafeCell`；原子类型的包装 |
| C-14 | `core/src/sync/atomic.rs` | `unsafe impl Send/Sync for Atomic` | 366 / 368 | library | 显式加回共享能力，因为操作是原子的 |
| C-14 | `core/src/sync/atomic.rs` | `pub enum Ordering` | 442 | library | 内存序变体；`Relaxed` 只保证原子性 |
| C-14 | `core/src/sync/atomic.rs` | `usize AtomicUsize`（宏展开点） | 3825 | library | `AtomicUsize` 从这里生成 |

### 复核命令

```bash
SRC="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
sed -n '92,105p;657,674p' "$SRC/core/src/marker.rs"
sed -n '317,325p;2328p' "$SRC/core/src/cell.rs"
sed -n '196,205p' "$SRC/std/src/thread/mod.rs"
sed -n '125,129p' "$SRC/std/src/thread/functions.rs"
sed -n '141,144p;201,205p' "$SRC/std/src/thread/scoped.rs"
sed -n '235,238p' "$SRC/std/src/sync/mod.rs"
sed -n '227,257p;277,294p' "$SRC/std/src/sync/poison/mutex.rs"
sed -n '361,368p;442,449p;3820,3826p' "$SRC/core/src/sync/atomic.rs"
```

---

## 读这些源码时最值得注意的三件事

### 1. `Send` / `Sync` 的方法体是空的

`marker.rs:92` 和 `:657` 没有方法。真正的判断在字段推导和显式 impl。
所以每一道题集题都要问"哪个字段最弱"，而不是"这个 trait 要求实现什么方法"。

### 2. `Mutex` 的 `Sync` 条件是 `T: Send`

`poison/mutex.rs:257` 写得明白：锁保证同一时刻最多一个线程碰到 `T`，
于是不要求 `T: Sync`。这就是第 5 题 `Mutex<Cell<u32>>` 两者皆是的原因。
`Arc` 没有这层互斥，所以两边都要 `T: Send + Sync`。

### 3. 原子类型的 `Sync` 是加回去的，不是天生的

`atomic.rs:361` 里仍是 `UnsafeCell`。`:366-368` 才显式 `Send` / `Sync`。
和 `Cell` 的差别是修改走原子指令，不是"没有内部可变性"。

---

## reference-fallback 理由说明

本模块三项能力都在 `library/` 下有对应实现，**无 fallback 项**。
`spawn` / `scope` 的调度属于 OS，但本模块要钉的是 bound 与锁的 auto trait 条件，
这两件事都在库代码里。
