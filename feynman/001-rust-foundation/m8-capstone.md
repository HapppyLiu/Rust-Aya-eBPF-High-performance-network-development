# Feynman: Module 8 — 综合实验（字节缓冲区报文解析器）

**Capabilities covered**: C-01, C-02, C-03, C-04, C-05, C-06, C-07, C-08, C-09, C-10, C-11, C-12, C-13, C-14, C-15, C-16, C-17, C-18, C-19, C-20, C-21, C-22, C-23, C-24

---

## 1. 用自己的话解释

面向"没学过 Rust 但懂 C"的同事。

前面七个模块像是七盒零件：所有权、类型、错误、线程、裸指针、和 C 说话、没有 libc。
每盒都通过了出厂检查。**本模块存在的理由是：出厂检查通过，不等于这些零件能装进同一台机器。**

C 里你写一个读包函数，常常一张图里同时有：谁 `free`、`const char *` 能活多久、
返回 -1 还是 errno、`*(uint16_t *)p` 会不会未对齐、要不要 `#include` 一堆 POSIX。
Rust 把这些拆开教了七次。装回去的时候，缝会裂：

- 你让视图活得比缓冲区久 —— 编译器直接拒绝，不是运行到一半崩。
- 你在没有操作系统的库里 `pthread_create` —— 那一层根本不存在。
- 你把边界检查写进 `unsafe` 块里再"顺便"读 —— 调用方一旦绕过检查就是未定义行为。
- 你箱子一个空结构体，以为一定发生了堆分配 —— 零大小类型的箱子是空的。

所以综合实验选了一个**面向字节缓冲区的最小解析器**。不是因为协议有趣，
而是同一段字节会同时碰到：谁拥有、怎么分发、失败怎么走、指针怎么挪、
以及库里没有文件和线程。24 项能力必须能指到**一个文件里的一个函数**。
指不到的那一项，不是"隐含用到了"，是没做完。

单项能力通过 ≠ 能组合使用。这不是口号。组合失败的形态和单项失败不一样：
单项是"不会 drop"，组合是"会 drop 的类型被另一个模块的裸指针看穿"。

---

## 2. 最小示例

完整观察：[`examples/capstone.rs`](../../experiments/001-rust-foundation/m8-capstone/examples/capstone.rs)
以及下表链接的 `src/`。每段 ≤15 行。

### C-01 Ownership / C-02 Move / C-03 Borrow / C-04 Lifetime

```rust
struct OwnedBytes { inner: Vec<u8> }          // 独占
fn as_buf(o: &OwnedBytes) -> PacketBuf<'_> { PacketBuf::new(&o.inner) }
fn into_vec(o: OwnedBytes) -> Vec<u8> { o.inner } // move
struct PacketBuf<'a> { bytes: &'a [u8] }      // 不拥有，Copy
```

[`buffer.rs`](../../experiments/001-rust-foundation/m8-capstone/src/buffer.rs)

### C-05 Struct / Enum

```rust
enum FrameKind { Data = 0, Control = 1 }
fn from_wire(v: u8) -> Result<FrameKind, ParseError> { /* 0/1 else BadKind */ }
```

[`wire.rs`](../../experiments/001-rust-foundation/m8-capstone/src/wire.rs)

### C-06 Trait / C-07 Generic

```rust
trait ParseHeader { fn parse_header<'a>(&self, b: PacketBuf<'a>) -> Result<(WireHeader, PacketBuf<'a>), ParseError>; }
fn dyn_p(p: &dyn ParseHeader, b: PacketBuf) { let _ = p.parse_header(b); }
fn parse_then<P: ParseHeader + ?Sized, F>(p: &P, b: PacketBuf, f: F) { /* ... */ }
```

[`parse.rs`](../../experiments/001-rust-foundation/m8-capstone/src/parse.rs)

### C-08 Error handling

```rust
enum ParseError { Truncated, BadMagic, BadVersion, BadKind }
let magic = read_u16_le(buf, 0)?; // Truncated 往外传
```

[`error.rs`](../../experiments/001-rust-foundation/m8-capstone/src/error.rs)

### C-09 Iterator / C-10 Closure / C-11 Smart pointer

```rust
impl Iterator for FrameIter<'_> { type Item = Result<Frame<'_>, ParseError>; fn next(&mut self) -> Option<Self::Item> { /* ... */ } }
fn map_ok_payloads<F, R>(it: FrameIter, mut f: F) -> Vec<R> where F: FnMut(PacketBuf) -> R { /* ... */ }
fn boxed_header_parser() -> Box<dyn ParseHeader> { Box::new(MagicParser { expected_magic: MAGIC }) }
```

[`iter.rs`](../../experiments/001-rust-foundation/m8-capstone/src/iter.rs)

### C-12 Send/Sync / C-13 Concurrency / C-14 Atomic

```rust
struct ParseStats { frames: AtomicUsize, errors: AtomicUsize } // 自动 Send+Sync
fn record_ok(&self) { self.frames.fetch_add(1, Ordering::Relaxed); }
// 库里没有 thread::spawn
```

[`stats.rs`](../../experiments/001-rust-foundation/m8-capstone/src/stats.rs)

### C-15…C-20 unsafe 封装

```rust
fn read_u16_le(buf: PacketBuf, off: usize) -> Result<u16, ParseError> {
    if off.checked_add(2).map(|e| e > buf.len()).unwrap_or(true) { return Err(ParseError::Truncated); }
    let q = unsafe { buf.as_bytes().as_ptr().add(off) };
    let a = unsafe { ptr::read_unaligned(q.cast::<[u8; 2]>()) };
    Ok(u16::from_le_bytes(a))
}
```

[`raw.rs`](../../experiments/001-rust-foundation/m8-capstone/src/raw.rs)

### C-21 FFI（布局，不是 C 调用）

```rust
#[repr(C)] struct WireHeader { magic: u16, version: u8, kind: u8, payload_len: u16 }
const _: () = { assert!(size_of::<WireHeader>() == 6); };
```

[`wire.rs`](../../experiments/001-rust-foundation/m8-capstone/src/wire.rs) · 说明：[`capability-location-map.md`](../../acceptance/001-rust-foundation/capability-location-map.md)

### C-22 / C-23 / C-24

```rust
#![no_std]
extern crate alloc;
// 库不注册 panic_handler / global_allocator
```

[`lib.rs`](../../experiments/001-rust-foundation/m8-capstone/src/lib.rs) · 分配断言：[`tests/capstone.rs`](../../experiments/001-rust-foundation/m8-capstone/tests/capstone.rs)

---

## 3. 底层机制

每条论断带本模块的断言名或源码符号。

1. **载荷是原缓冲的子区间，不是拷贝。**
   依据：`parses_data_frame_zero_copy`（指针关系相等）；`parse_borrowed_view_allocates_zero`（`allocs == 0`）。

2. **`PacketBuf` 是 `Copy` 因为不拥有。**
   依据：`buffer.rs` 的 `#[derive(Copy)]`；对照 `OwnedBytes` 无 `Copy`；`owned_bytes_borrow_matches_parse`。

3. **未知 kind 是枚举拒绝，不是默当 Data。**
   依据：`unknown_kind_is_bad_kind`；`FrameKind::from_wire`。

4. **`dyn` 只接对象安全的 `ParseHeader`，不接带寿命的 `Frame`。**
   依据：`parse_header_dyn`；`dyn_header_parser_matches_static`。

5. **`?` 把截断从 `read_u16_le` 传到 `parse_frame`。**
   依据：`truncated_header_and_payload`；`result.rs:2192` `FromResidual`。

6. **空迭代立即结束；失败再停止。**
   依据：`empty_iter_yields_nothing`；`iter_updates_stats` 的 `errors == 1`。

7. **装箱 ZST 不分配；装箱 `MagicParser` 分配一次。**
   依据：`boxing_parser_allocates_once`；`alloc/src/boxed.rs:284` `Box::new`。

8. **`ParseStats` 自动 `Send + Sync`，没有 `unsafe impl`。**
   依据：`views_and_stats_are_send_sync`；`marker.rs:92` `Send`、`:657` `Sync`。

9. **库内并发是原子，不是线程。**
   依据：`stats.rs` 无 `std::`；`concurrent_stats_end_at_two` 在**测试 crate**。

10. **未对齐走 `read_unaligned`；要 `&[u16]` 则先查对齐。**
    依据：`le_u16_from_unaligned_bytes`；`u16_slice_requires_alignment`；`ptr/mod.rs:1810`。

11. **公开 API 越界返回错误，Miri 对安全路径 clean。**
    依据：`short_slice_is_truncated_not_ub`；OBSERVATIONS「UB 判定记录」。

12. **`repr(C)` 布局是 FFI 前置，不是 FFI 本身。**
    依据：`wire_header_layout`；location-map C-21 说明。

13. **`--no-default-features` 成功 ≠ 裸机可执行。**
    依据：OBSERVATIONS「C-22 / no_std 库构建」的"不能证明"；对照 m7 `_start`。

---

## 4. 常见误区

每条：**误解 → 实际 → 证据**。覆盖全部 24 项（按层合并书写，C-ID 标在误解前）。

- **C-01/C-02** 误解：视图 `Copy` 复制了包
  → **实际**：复制指针和长度
  → **证据**：`parses_data_frame_zero_copy`

- **C-03/C-04** 误解：`as_buf` 返回的视图可以比 `OwnedBytes` 活得长
  → **实际**：寿命钉在 `&self` 上，编译期拒绝
  → **证据**：`PacketBuf<'a>` 字段；对照 m1 悬空引用样本（本模块用类型系统，不另做 compile_fail）

- **C-05** 误解：`kind: u8` 就够了
  → **实际**：未知值必须是 `BadKind`
  → **证据**：`unknown_kind_is_bad_kind`

- **C-06/C-07** 误解：有 trait 就能 `dyn`
  → **实际**：带输入寿命的关联类型会破坏对象安全
  → **证据**：`dyn` 只出现在 `ParseHeader`

- **C-08** 误解：解析失败就是 false
  → **实际**：四变体分开断言
  → **证据**：`bad_magic_and_version_are_distinct` / `truncated_header_and_payload`

- **C-09** 误解：空缓冲上的迭代器先产出一个错误
  → **实际**：`None`，结束
  → **证据**：`empty_iter_yields_nothing`

- **C-10** 误解：闭包在 `map_ok_payloads` 被写出的那一行就开始跑
  → **实际**：迭代器拉动时才调用（与 m3 惰性同一机制）
  → **证据**：`iterates_two_frames_and_maps_payloads`；`iterator.rs:78` `next`

- **C-11** 误解：`Box::new` 一定 heap allocate
  → **实际**：ZST 不分配，所以装箱带 `u16` 的 `MagicParser`
  → **证据**：`boxing_parser_allocates_once`；实现注释

- **C-12** 误解：要跨线程必须手写 `unsafe impl Send`
  → **实际**：字段全是原子则自动成立；手写会跳过检查
  → **证据**：`views_and_stats_are_send_sync`；`stats.rs` 模块文档

- **C-13** 误解：综合实验必须在库里 `spawn`
  → **实际**：库没有 OS services；原子是换实现，线程只在测试 crate
  → **证据**：location-map C-13 行；`concurrent_stats_end_at_two`

- **C-14** 误解：Relaxed 不能用于计数
  → **实际**：本场景只断言终值、不断言交错，Relaxed 足够
  → **证据**：`iter_updates_stats`；R-05 禁止断言线程交错

- **C-15** 误解：有 `unsafe` 块就够了，注释可写"Rust 要求"
  → **实际**：五要素逐项写；空洞注释被契约拒绝
  → **证据**：`raw.rs` 每块 `SAFETY`；clippy `undocumented_unsafe_blocks`

- **C-16/C-17** 误解：越界 `add` 只要别解引用就行
  → **实际**：`add` 本身要求在分配内
  → **证据**：检查在 `add` 之前；`short_slice_is_truncated_not_ub`

- **C-18** 误解：x86_64 容忍未对齐所以可以 `*(p as *const u16)`
  → **实际**：语言要对齐或 `read_unaligned`；Miri 按语言判
  → **证据**：`le_u16_from_unaligned_bytes`；Miri clean

- **C-19** 误解：只读解析不可能有别名问题
  → **实际**：再做一个 `&mut` 看同一块就是冲突；本层保证不产生 `&mut`
  → **证据**：`payload_view` SAFETY「别名」；m5 C-19 对照

- **C-20** 误解：程序没崩说明封装安全
  → **实际**：以 Miri 为准
  → **证据**：OBSERVATIONS UB 表；FR-019

- **C-21** 误解：`repr(C)` = 做过 FFI
  → **实际**：只证明布局；调用、端序、所有权在 m6
  → **证据**：location-map C-21 说明；`wire_header_layout`

- **C-22** 误解：`--no-default-features` 成功 = 裸机可跑
  → **实际**：这是 host 上的 rlib
  → **证据**：OBSERVATIONS「C-22 / no_std 库构建」

- **C-23** 误解：`extern crate alloc` 等于有堆
  → **实际**：alloc 会打电话；接电话的是最终二进制的分配器
  → **证据**：`alloc/src/lib.rs:74-75`；测试里才有 `#[global_allocator]`

- **C-24** 误解：解析器永远不分配
  → **实际**：借用视图 0 次；`OwnedBytes` / 非 ZST 的 `Box` 会分配
  → **证据**：`parse_borrowed_view_allocates_zero` vs `boxing_parser_allocates_once`

- **模块级（强制）** 误解：24 项 AC 都过了，综合实验只是走个过场
  → **实际**：组合缝上的失败（ZST 的 Box、库内不能 spawn、未对齐、C-21 只能说明）单项测试看不见
  → **证据**：本节全部；T136 要求解释本条

---

## 5. 验证性问题

每题的回答指向一条断言、一处源码或一个观测块。

1. 24 项定位表允许把哪一项写成"配套说明"？说明里必须出现什么？
   → C-21。必须写清**为什么**不做 C 调用，以及布局断言在哪个函数。
     依据：location-map C-21 段；`wire_header_layout`。

2. 测试 crate 有 `std`。C-13 应写在库的原子上、测试的 `spawn` 上，还是两者都写并标明边界？
   → 两者都写。库：`ParseStats::record_ok`（换实现）。测试：`concurrent_stats_end_at_two`（OS services）。
     依据：location-map C-13 行。

3. 去掉 `raw` 的长度检查、只留 `unsafe` 读，Miri 还会是 clean 吗？
   → 不会。过短切片会变成越界 `add`/`read`。依据：`short_slice_is_truncated_not_ub` 的对立面；
     OBSERVATIONS UB 块"这不能证明什么"。

4. `Box<dyn ParseHeader>` 证明的是智能指针、trait 对象，还是交界？定位表怎么拆？
   → 交界。C-06 → `parse_header_dyn`；C-11 → `boxed_header_parser`。一行充数两项会让 SC-009 漏项。
     依据：location-map C-06 / C-11。

5. 解析借用视图 `allocs == 0`，能否推出解析器永远不分配？
   → 不能。`OwnedBytes::from_slice` 与非 ZST 的 `Box` 会分配。
     依据：`parse_borrowed_view_allocates_zero`；`boxing_parser_allocates_once`。

6. 本模块 Feynman 哪一项专门挡住"积木都会、整机不会"？
   → 第 1 节（自述本模块存在的理由）和第 4 节强制误区。五项合取，缺一则 m8 不得 complete。
     依据：FR-006；T136；本节强制条。

---

## 检验结果

| # | 小节 | 结果 | 依据 |
|---|------|------|------|
| 1 | 用自己的话解释 | pass | 面向懂 C 的同事；Rust 术语当场用 C 类比；写明"单项通过 ≠ 能组合" |
| 2 | 最小示例 | pass | 24 项按层各有 ≤15 行片段，链到 `src/` 与 `examples/capstone.rs` |
| 3 | 底层机制 | pass | 13 条均指向断言名、Miri 记录或 source-refs 符号 |
| 4 | 常见误区 | pass | 每层 C-ID 均有三段式；含强制条"综合实验不是过场" |
| 5 | 验证性问题 | pass | 6 题，每题指向断言 / location-map / 观测块 |

**模块 Feynman 五项：合取通过。**
