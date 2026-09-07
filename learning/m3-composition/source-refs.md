# Module 3 源码引用

**Story**: US3 | **Capabilities**: C-08…C-11 | **依据**: FR-005 / SC-003 /
[learning-artifact-contract §B](../../specs/001-rust-foundation/contracts/learning-artifact-contract.md)

路径根：`$(rustc --print sysroot)/lib/rustlib/src/rust/library/`
本机为 `/root/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/`

行号在 pinned 工具链 **1.98.0**（`88d9e12ae 2026-08-18`）下固定，因此可被记录并复核（规则 B1）。

---

## 引用表

| C-ID | 路径（相对 `library/`） | 符号 | 行 | kind | 这段源码回答了什么 |
|------|----------------------|------|----|------|------------------|
| C-08 | `core/src/result.rs` | `pub enum Result<T, E>` | 557 | library | 成功 / 失败是两个变体，错误值有自己的类型参数 |
| C-08 | `core/src/result.rs` | `FromResidual` for `Result` | 2185 | library | `?` 失败路径：把内层错误包进外层 `Err` |
| C-08 | `core/src/result.rs` | `Err(From::from(e))` | 2192 | library | 转换调用的就是 `From::from` |
| C-08 | `core/src/convert/mod.rs` | `pub const trait From<T>` | 589 | library | `?` 要求外层错误：`From<内层错误>` |
| C-08 | `core/src/convert/mod.rs` | `fn from` | 594 | library | 转换方法本身 |
| C-08 | `std/src/error.rs` | `pub use core::error::Error` | 4 | library | `std::error::Error` 是再导出，定义不在 `std` |
| C-08 | `core/src/error.rs` | `pub trait Error` | 59 | library | `Debug + Display`，并提供 `source()` |
| C-09 | `core/src/iter/traits/iterator.rs` | `pub const trait Iterator` | 42 | library | 迭代器接口；`Item` 是关联类型 |
| C-09 | `core/src/iter/traits/iterator.rs` | `fn next` | 78 | library | 一次推进返回 `Option<Item>`；`None` 表示结束 |
| C-09 | `core/src/iter/traits/iterator.rs` | `fn map` | 831 | library | `map` 只构造 `Map`，不拉动 `next` |
| C-09 | `core/src/iter/adapters/map.rs` | `pub struct Map<I, F>` | 61 | library | 适配器存上游迭代器 + 闭包；文档写明惰性 |
| C-09 | `core/src/iter/adapters/map.rs` | `fn next` | 106 | library | 一次 `next` = 取一个上游元素 + 立刻调用闭包 |
| C-09 | `core/src/iter/adapters/map.rs` | `fn size_hint` | 111 | library | `Map` 原样转发上游 hint，所以 `collect` 能一次按长度分配 |
| C-10 | `core/src/ops/function.rs` | `pub const trait Fn` | 76 | library | `&self` 调用；超 trait 是 `FnMut` |
| C-10 | `core/src/ops/function.rs` | `pub const trait FnMut` | 163 | library | `&mut self` 调用；超 trait 是 `FnOnce` |
| C-10 | `core/src/ops/function.rs` | `pub const trait FnOnce` | 242 | library | 按值 `self` 调用；关联类型 `Output` |
| C-11 | `alloc/src/boxed.rs` | `pub struct Box<T, A>` | 234 | library | 内部 `Unique<T>`：独占这块堆内存 |
| C-11 | `alloc/src/rc.rs` | `pub struct Rc<T, A>` | 320 | library | 共享指针，指向 `RcInner` |
| C-11 | `alloc/src/rc.rs` | `impl !Send for Rc` | 330 | library | 显式负向 impl：不能进 `thread::spawn` |
| C-11 | `alloc/src/rc.rs` | `impl !Sync for Rc` | 338 | library | 显式负向 impl：不能共享引用到其他线程 |
| C-11 | `alloc/src/rc.rs` | `Clone for Rc` / `inc_strong` | 2495 / 2513 | library | `clone` 加计数，不新分配 |
| C-11 | `alloc/src/rc.rs` | `fn strong_count` | 1814 | library | 可观察的共享边数量 |
| C-11 | `alloc/src/sync.rs` | `pub struct Arc<T, A>` | 269 | library | 跨线程共享指针，指向 `ArcInner` |
| C-11 | `alloc/src/sync.rs` | `unsafe impl Send for Arc` | 279 | library | 条件是 `T: Sync + Send` |
| C-11 | `alloc/src/sync.rs` | `unsafe impl Sync for Arc` | 281 | library | 同上；没有这两条，`Arc` 也不能进线程 |
| C-11 | `alloc/src/sync.rs` | `Clone for Arc` | 2383 / 2399 | library | `clone` 同样只加计数 |

### 复核命令

```bash
SRC="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
sed -n '557,575p' "$SRC/core/src/result.rs"                 # C-08 Result
sed -n '2185,2194p' "$SRC/core/src/result.rs"               # C-08 FromResidual + From::from
sed -n '589,594p' "$SRC/core/src/convert/mod.rs"            # C-08 From
sed -n '4,5p'     "$SRC/std/src/error.rs"                   # C-08 re-export
sed -n '59p'      "$SRC/core/src/error.rs"                  # C-08 Error
sed -n '42,78p'   "$SRC/core/src/iter/traits/iterator.rs"   # C-09 Iterator / next
sed -n '831,836p' "$SRC/core/src/iter/traits/iterator.rs"   # C-09 map
sed -n '61,65p;106,112p' "$SRC/core/src/iter/adapters/map.rs"  # C-09 Map::next / size_hint
sed -n '76,79p;163,166p;242,250p' "$SRC/core/src/ops/function.rs"  # C-10 Fn*
sed -n '234,239p' "$SRC/alloc/src/boxed.rs"                 # C-11 Box
sed -n '320,338p' "$SRC/alloc/src/rc.rs"                    # C-11 Rc + !Send/!Sync
sed -n '1814p;2495,2515p' "$SRC/alloc/src/rc.rs"            # C-11 strong_count / Clone
sed -n '269,281p;2383,2399p' "$SRC/alloc/src/sync.rs"       # C-11 Arc Send/Sync / Clone
```

---

## 读这些源码时最值得注意的三件事

### 1. `?` 的转换写在 `FromResidual` 里，不写在 `From` 里

`From`（`convert/mod.rs:589`）只定义"一种类型怎么变成另一种"。
`?` 在 `Result` 上的行为是 `result.rs:2192` 那一行 `Err(From::from(e))`。
所以缺 `From` 时诊断会说"the trait `From<Inner>` is not implemented"——
对准的是转换，报出的错误码是 E0277，不是一种新的"`?` 专用码"。

### 2. `Map::next` 是"一个元素走完整条链"的直接证据

`map.rs:106` 一次调用只做 `self.iter.next().map(&mut self.f)`。
没有循环，没有"先耗尽上游"。example 里 `map`/`filter` 交错打印，
就是这三行代码的运行时形状。`size_hint`（`:111`）原样转发，
这才让精确长度的 `collect` 能一次 `alloc`。

### 3. `Rc` 的 `!Send` 是写在结构体旁边的负向 impl，不是推断出来的附带现象

`rc.rs:330` 显式 `impl !Send`。注释说即使不写，内部的 `Cell` 也会让它 `!Sync`，
但负向 impl 是给人和诊断看的。`Arc` 反过来：`sync.rs:279` 在 `T: Send + Sync`
时正向 `Send`。C-10 的 E0373（借用活不过 `'static`）和 C-11 的 E0277（`!Send`）
对准 `spawn` 的两个 bound（`F: Send + 'static`），不是同一问。

---

## reference-fallback 理由说明

本模块四项能力都在 `library/` 下有对应实现，**无 fallback 项**。
闭包的代码生成（捕获环境怎么排进闭包结构体）属于 rustc，但本模块要钉的是
三个 `Fn*` trait 的 `self` 形式与 `Send`/`'static` 边界，这两件事都在库代码里。
