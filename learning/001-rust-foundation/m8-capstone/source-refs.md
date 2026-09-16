# Module 8 源码引用

**Story**: US8 | **Capabilities**: C-01…C-24（综合对照） | **依据**: FR-005 / SC-003 /
[learning-artifact-contract §B](../../../contracts/001-rust-foundation/learning-artifact-contract.md)

路径根：`$(rustc --print sysroot)/lib/rustlib/src/rust/library/`
本机为 `/root/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/`

行号在 pinned 工具链 **1.98.0**（`88d9e12ae 2026-08-18`）下固定。
本表对照综合实验**实际用到**的符号；各能力的首次精读仍以 m1–m7 的 `source-refs.md` 为准。

---

## 引用表

| C-ID | 路径（相对 `library/`） | 符号 | 行 | kind | 这段源码回答了什么 |
|------|----------------------|------|----|------|------------------|
| C-01 | `core/src/ops/drop.rs` | `pub const trait Drop` | 209 | library | `PacketBuf` 为什么不需要自己的 `Drop`：它不拥有内存 |
| C-02 | `core/src/mem/mod.rs` | `pub const fn replace<T>` | 953 | library | 移动拥有者时如何不留下无效状态；本实验用 `into_vec` 一次移走 |
| C-03 | `core/src/cell.rs` | `pub struct UnsafeCell` | 2323 | library | 内部可变性的底层载体；本解析器的视图路径**不用**它（只读共享） |
| C-03 | — Rust Reference § Borrow checker | — | — | reference-fallback | 编译期借用检查无库代码；`as_buf` 的寿命由 rustc 钉住 |
| C-04 | `core/src/marker.rs` | `pub struct PhantomData` | 811 | library | 切片迭代器用来钉寿命；`PacketBuf<'a>` 把寿命写在字段 `&'a [u8]` 上，不必 PhantomData |
| C-04 | — Rust Reference § Lifetime elision | — | — | reference-fallback | 方法寿命省略；类型上的 `'a` 不能省略 |
| C-05 | `core/src/mem/mod.rs` | `size_of` / `align_of` / `offset_of` | 373 / 540 / 1617 | library | `repr(C)` 头的确定性布局量 |
| C-06 | `core/src/ops/deref.rs` | `pub const trait Deref` | 139 | library | 智能指针解引用；`Box<dyn ParseHeader>` 经 Deref 调方法 |
| C-07 | `core/src/iter/traits/iterator.rs` | `pub const trait Iterator` | 42 | library | 泛型以 trait bound 为契约；`parse_then` 的 `P: ParseHeader` 同类 |
| C-08 | `core/src/result.rs` | `FromResidual` / `Err(From::from(e))` | 2185 / 2192 | library | `?` 在 `parse_header` 里传播 `Truncated` |
| C-09 | `core/src/slice/iter.rs` | `pub struct Iter<'a, T>` | 67 | library | 标准切片迭代器也是"指针 + 寿命"；`FrameIter` 同一形状 |
| C-09 | `core/src/iter/traits/iterator.rs` | `fn next` | 78 | library | 一次推进返回 `Option<Item>`；空则 `None` |
| C-10 | `core/src/ops/function.rs` | `FnOnce` / `FnMut` | 242 / 163 | library | `parse_then` 吃 `FnOnce`；`map_ok_payloads` 吃 `FnMut` |
| C-11 | `alloc/src/boxed.rs` | `pub struct Box<T, A>` | 234 | library | 独占堆内存 |
| C-11 | `alloc/src/boxed.rs` | `pub fn new` | 284 | library | ZST 与非 ZST 的分配差在 `LAYOUT`；本实验因此装箱 `MagicParser` |
| C-11 | `alloc/src/vec/mod.rs` | `pub struct Vec<T, A>` | 436 | library | `OwnedBytes` / `encode_frame` 的堆缓冲 |
| C-12 | `core/src/marker.rs` | `unsafe auto trait Send` | 92 | library | 自动 trait：字段全 `Send` 则结构体 `Send`，禁止手写 impl |
| C-12 | `core/src/marker.rs` | `unsafe auto trait Sync` | 657 | library | `AtomicUsize: Sync` ⇒ `ParseStats: Sync` |
| C-13 | `core/src/sync/atomic.rs` | 模块文档：原子用于无锁共享 | 8 / 148 | library | 库内并发换实现：原子而不是 `std::thread` |
| C-14 | `core/src/sync/atomic.rs` | `usize AtomicUsize` 宏展开 | 3825 | library | `fetch_add` / `load` 的承载类型 |
| C-15 | `core/src/ptr/mod.rs` | `pub const unsafe fn read` | 1692 | library | 裸读；调用方承担有效性 |
| C-16 | `core/src/ptr/mod.rs` | `read` / `addr_of` 族 | 1692 | library | `read_u8` 走 `ptr::read` |
| C-17 | `core/src/ptr/const_ptr.rs` | `add` / `offset` | 838 / 354 | library | 分配内偏移；越界是 UB，所以检查在块外 |
| C-18 | `core/src/ptr/mod.rs` | `pub const unsafe fn read_unaligned` | 1810 | library | 卸掉对齐义务，有效性还在 |
| C-18 | `core/src/mem/mod.rs` | `align_of` | 540 | library | `try_as_u16_slice` 用它拒绝未对齐起点 |
| C-19 | `core/src/cell.rs` | `UnsafeCell` / `!Sync` | 2323 / 2328 | library | 内部可变才打破共享；本层只读，不引入 `UnsafeCell` |
| C-20 | `core/src/slice/raw.rs` | `pub const unsafe fn from_raw_parts` | 124 | library | `payload_view` 拼子切片；长度必须先合法 |
| C-21 | `core/src/mem/mod.rs` | `size_of` / `offset_of` | 373 / 1617 | library | 布局断言的量具；不是 C 调用 |
| C-21 | `core/src/ffi/mod.rs` | `c_int` / `c_char` | 36–38 | library | 真 FFI 类型别名在 m6；本模块只对照"布局是 FFI 的前置" |
| C-22 | `core/src/lib.rs` | `#![no_core]` | 64 | library | 库的 `#![no_std]` 仍依赖这一层 |
| C-23 | `alloc/src/lib.rs` | `#![needs_allocator]` | 74–75 | library | `extern crate alloc` 请来集合，堆仍要最终二进制提供 |
| C-24 | `alloc/src/alloc.rs` | `__rust_alloc` 由全局分配器生成 | 12–22 | library | 测试里 `CountingAllocator` 接在这条线上；库本身不注册 |

### 复核命令

```bash
SRC="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
sed -n '67,78p'   "$SRC/core/src/slice/iter.rs"          # C-09 Iter
sed -n '436,450p' "$SRC/alloc/src/vec/mod.rs"            # C-11 Vec
sed -n '234,239p;284,291p' "$SRC/alloc/src/boxed.rs"     # C-11 Box / new
sed -n '92,94p;657,659p' "$SRC/core/src/marker.rs"      # C-12 Send / Sync
sed -n '3820,3826p' "$SRC/core/src/sync/atomic.rs"       # C-14 AtomicUsize
sed -n '1810,1815p' "$SRC/core/src/ptr/mod.rs"           # C-18 read_unaligned
sed -n '124,140p' "$SRC/core/src/slice/raw.rs"           # C-20 from_raw_parts
sed -n '74,75p'   "$SRC/alloc/src/lib.rs"                # C-23 needs_allocator
```

---

## 读这些源码时最值得注意的三件事

### 1. 切片迭代器和 `PacketBuf` 是同一类东西

`slice::Iter` 存的是指针 + 寿命标记（`:67`），不是元素的拷贝。
`PacketBuf` 更薄：直接存 `&'a [u8]`。零拷贝不是优化口号，是这个数据结构。

### 2. `Box::new` 的分配由 `LAYOUT` 决定

`:284` 的 `new` 按 `T` 的 layout 分配。`HeaderParser` 零大小时没有"一块堆"可独占，
所以综合实验装箱 `MagicParser { expected_magic }`。这是 C-11 在组合场景里才会撞上的细节。

### 3. `read_unaligned` 卸的是对齐，不是有效性

`:1810` 的文档与实现仍然要求指针指向足够的已初始化内存。
网卡缓冲未对齐是常态；越界仍然是 UB。检查留在 `unsafe` 外面。

---

## reference-fallback 理由说明

仅 C-03（borrowck）与 C-04（elision）使用 fallback，与 m1 同一范围（规则 B2 已知三项中的两项）。
其余能力均为 `kind = library`。C-21 引用 `core/src/ffi` 是为了标明"真 FFI 在 m6"，
不是把别名当成本 crate 做过 C 调用。
