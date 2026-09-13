# Module 8: 综合实验（字节缓冲区报文解析器）

**Story**: US8（P3） | **Capabilities**: C-01…C-24（全部） | **Prerequisite**: m1–m7 全部 accepted

> 本文件属于 **Answer Track**。对应的 Learner Track 是
> [`learner/m8-capstone/guide.md`](../../learner/m8-capstone/guide.md)。
> 如果你还没做过那边的预测表，先去做 —— 这份文件读过之后就不能再自测了。

## 这个模块回答什么问题

1. 单项能力的 AC 都过了，为什么还不能说"会用系统级 Rust"？
2. 一个面向字节缓冲区的最小解析器，怎样同时压上所有权、trait、错误、
   unsafe 封装和 `no_std`，而不是再做 24 个互不相干的函数？
3. 组合时最容易在哪几条缝上裂开：视图活过拥有者、错误类型对不上、
   检查被另一层绕开、库里没有操作系统却去起线程？
4. 24 项能力怎样逐项定位到**文件 + 函数**，漏一项为什么等于综合实验没完成？

---

## 综合场景设计

输入是一段字节，线上格式固定为小端：

```text
magic:u16 | version:u8 | kind:u8 | payload_len:u16 | payload[payload_len]
```

魔数 `0xA5A5`，版本 `1`，`kind` 只有 Data / Control。多帧首尾相接。

输出是 [`Frame`](../../experiments/m8-capstone/src/parse.rs)：逻辑头 + 指向原缓冲的载荷视图。
失败是 [`ParseError`](../../experiments/m8-capstone/src/error.rs) 的四个变体，不是"出错了"。

这个场景不是玩具协议课。它被选中，是因为**同一段字节**会同时碰到七类问题：

| 分层 | 文件 | 为什么这个场景逼出这些能力 |
|------|------|--------------------------|
| 谁拥有这段字节 | `buffer.rs` | 捕获卡、环形缓冲、skb 线性化之前，都是"看着别人的内存" |
| 种类与分发 | `parse.rs` | 报文种类是 enum；解析步骤要能泛型组合，也要能 `dyn` 一次 |
| 失败与多帧 | `error.rs` / `iter.rs` | 截断、坏魔数、坏种类必须分开；多帧是迭代器，不是手写下标 |
| 计数 | `stats.rs` | 数据面统计不能在库里 `thread::spawn`（没有 OS） |
| 读未对齐字节 | `raw.rs` | 网卡 DMA 缓冲不保证按 `u16` 对齐 |
| 与 C 头对话 | `wire.rs` | 以后要和内核结构体比布局；这里只先把 `repr(C)` 量对 |
| 没有 std | crate root | eBPF / 内核模块都没有文件和线程；本库先在 host 上把约束钉死 |

七个模块的能力不是"再各写一个 example"，而是**同一条解析路径上的不同义务**。
拿掉任何一层，要么编译不过，要么 Miri 不再是 clean，要么 24 行定位表出现空行。

---

## 概念

### C-01 Ownership / C-02 Move semantics / C-03 Borrowing / C-04 Lifetime

- **一句话定义**：[`OwnedBytes`](../../experiments/m8-capstone/src/buffer.rs) 独占 `Vec<u8>`；
  [`PacketBuf<'a>`](../../experiments/m8-capstone/src/buffer.rs) 只保存 `&'a [u8]`，因此是 `Copy`。

- **底层机制**：`PacketBuf` 没有 `Drop` glue —— 它不拥有内存。`OwnedBytes::into_vec`
  把 `Vec` 移走，原绑定失效（C-02）。`as_buf` 的返回寿命钉在 `&self` 上（C-04）。
  解析后的 `frame.payload.as_bytes().as_ptr()` 与原缓冲 `HEADER_SIZE` 处相同
  （`parses_data_frame_zero_copy`）：这是指针**关系**，不是地址数值。

- **常见误解**："`Copy` 的视图复制了载荷" → 复制的是指针和长度。
  → **证据**：零拷贝断言；`parse_borrowed_view_allocates_zero` 的 `allocs == 0`。

- **对应实验**：`tests/capstone.rs` `parses_data_frame_zero_copy` / `owned_bytes_borrow_matches_parse`

### C-05 Struct / Enum

- **一句话定义**：线上 `kind` 是 `u8`；逻辑上是 [`FrameKind`](../../experiments/m8-capstone/src/wire.rs) 枚举。

- **底层机制**：`from_wire` 只接受 0/1，其余 → `BadKind`。`WireHeader` 是 `repr(C)` 结构体，
  字段顺序就是 C 会看到的顺序，但字段值是逻辑端序。

- **常见误解**："用 `u8` 就够了，enum 是语法糖" → 未知种类在类型里被挡住，不会 silently 当成 Data。
  → **证据**：`unknown_kind_is_bad_kind`

- **对应实验**：`tests/capstone.rs` `unknown_kind_is_bad_kind`；`tests/wire.rs`

### C-06 Trait / C-07 Generic

- **一句话定义**：[`ParseHeader`](../../experiments/m8-capstone/src/parse.rs) 对象安全，可 `&dyn ParseHeader`；
  [`parse_then`](../../experiments/m8-capstone/src/parse.rs) 对 `P: ParseHeader + ?Sized` 单态化。

- **底层机制**：`Frame<'a>` 带着输入切片的寿命，不能直接当 `dyn` 的关联类型。
  把头解析拆成不带 GAT 的 trait，才能走 vtable（`parse_header_dyn`）。
  完整帧走泛型闭包，不硬塞进同一个 `dyn`。

- **常见误解**："有了 trait 就能 `dyn`" → 关联类型一旦提到 `'a`，对象安全就没了。
  → **证据**：代码里 `dyn` 只出现在 `ParseHeader`，不出现在 `Frame`。

- **对应实验**：`dyn_header_parser_matches_static`；`parse_frame` 内部调用 `parse_then`

### C-08 Error handling

- **一句话定义**：失败是 [`ParseError`](../../experiments/m8-capstone/src/error.rs) 四变体；
  `?` 把 `read_u16_le` 的 `Truncated` 传出 `parse_header`。

- **底层机制**：截断、坏魔数、坏版本、坏种类是不同变体。`Display` + `core::error::Error`
  在 `no_std` 下仍然可用（core 的 `Error`，不是 std 再导出）。

- **常见误解**："解析失败就是 false" → 四条路径的测试分别断言变体。
  → **证据**：`truncated_header_and_payload` / `bad_magic_and_version_are_distinct`

- **对应实验**：`tests/capstone.rs` 错误路径组

### C-09 Iterator / C-10 Closure / C-11 Smart pointer

- **一句话定义**：[`FrameIter`](../../experiments/m8-capstone/src/iter.rs) 实现 `Iterator`；
  [`map_ok_payloads`](../../experiments/m8-capstone/src/iter.rs) 吃 `FnMut`；
  [`boxed_header_parser`](../../experiments/m8-capstone/src/iter.rs) 是 `Box<dyn ParseHeader>`。

- **底层机制**：空缓冲 `next` 返回 `None`，不先产一个 `Err`。失败后把 rest 置空，停止。
  `Box::new(HeaderParser)` **不会**分配：`HeaderParser` 是 ZST。所以装箱的是带 `u16`
  字段的 [`MagicParser`](../../experiments/m8-capstone/src/parse.rs)（`boxing_parser_allocates_once`）。

- **常见误解**："`Box::new` 一定堆分配" → ZST 的 Box 是一个对齐后的空壳。
  → **证据**：把 `HeaderParser` 换成 `MagicParser` 前后的分配次数对照（实现注释 + 断言 `allocs == 1`）。

- **对应实验**：`empty_iter_yields_nothing` / `iterates_two_frames_and_maps_payloads` / `boxing_parser_allocates_once`

### C-12 Send / Sync / C-13 Concurrency / C-14 Atomic

- **一句话定义**：[`ParseStats`](../../experiments/m8-capstone/src/stats.rs) 只含 `AtomicUsize`，
  因此自动 `Send + Sync`；库里**没有** `std::thread`。

- **底层机制**：缺的是 OS services（C-22/C-13 交界），不是原子 API。
  `record_ok` 用 `Ordering::Relaxed` 做计数 —— 本场景只断言终值，不断言交错。
  测试 crate 有 std，用 `thread::scope` 起两个线程各加一次，终值 2
  （`concurrent_stats_end_at_two`）。禁止手写 `unsafe impl Send/Sync`。

- **常见误解**："综合实验在库里起线程才算做了并发" → 库的约束就是不能起。
  → **证据**：`src/stats.rs` 无 `std::`；`views_and_stats_are_send_sync` 的静态断言。

- **对应实验**：`views_and_stats_are_send_sync` / `concurrent_stats_end_at_two` / `iter_updates_stats`

### C-15 Unsafe Rust / C-16 Raw pointer / C-17 Pointer arithmetic / C-18 Alignment / C-19 Aliasing / C-20 Memory safety

- **一句话定义**：[`raw`](../../experiments/m8-capstone/src/raw.rs) 对外全是安全函数；
  检查在 `unsafe` 之外；块内一次一个 unsafe 操作。

- **底层机制**：
  - C-16：`ptr::read` 读 `u8`。
  - C-17：`p.add(offset)`，前置条件 `offset` 已证明在分配内。
  - C-18：报文不保证 `u16` 对齐，走 `read_unaligned` + `from_le_bytes`；
    `try_as_u16_slice` 对未对齐起点返回 `None`。
  - C-19：只产生共享切片，不产生 `&mut`，与输入 `&[u8]` 兼容。
  - C-20：`payload_view` 用 `from_raw_parts` 拼子切片，长度先检查。

- **常见误解**："x86_64 容忍未对齐，所以可以直接 `*(p as *const u16)`" →
  语言规则仍要求对齐或走 unaligned；Miri 按语言规则判。
  → **证据**：`le_u16_from_unaligned_bytes`；`u16_slice_requires_alignment`；Miri clean。

- **对应实验**：`tests/raw.rs` 全部；`cargo +nightly miri test -p m8-capstone`

### C-21 FFI

- **一句话定义**：本 crate **不做**真实 C 调用。C-21 落在 [`wire.rs`](../../experiments/m8-capstone/src/wire.rs)
  的 `repr(C)` 布局断言，以及本段说明。

- **底层机制**：`layout()` 给出 size=6 / align=2 / `payload_len` 偏移 4。
  线上编码是显式小端，和本机端序分开。真实 syscall、双向调用、谁分配谁释放，
  已经在 m6 验收。这里再链一遍 C 会把 `no_std` 库拖进 `cc`/`libc`，并假装
  "综合实验做过 FFI"。SC-009 允许产物**或配套说明**。

- **常见误解**："写了 `repr(C)` 就是 FFI 安全" → 还缺调用约定、端序、所有权。
  → **证据**：`wire_header_layout`；本小节；location-map C-21 行。

- **对应实验**：`tests/wire.rs`；说明见 `acceptance/capability-location-map.md`

### C-22 no_std / C-23 core / alloc / std / C-24 Panic and allocator

- **一句话定义**：库是 `#![no_std]` + `extern crate alloc`；`std` 只出现在测试 crate
  与 `#[cfg(test)] extern crate std`。

- **底层机制**：库不是裸机二进制，不提供 `#[panic_handler]` / `#[global_allocator]`。
  panic 与系统分配器由 host 测试运行器（std）提供。这与 m7 不同：m7 必须自己接两头。
  `parse_borrowed_view_allocates_zero` 证明零拷贝路径不向堆要内存；
  `OwnedBytes::from_slice` 与 `boxed_header_parser` 各一次分配。
  `cargo build -p m8-capstone --no-default-features` 成功，证明库不依赖 std 的 OS services。

- **常见误解**：
  - "构建成功 = 裸机可执行" → 这是库，没有 `_start`。
  - "`extern crate alloc` 所以库自己有堆" → 堆仍由最终二进制的全局分配器提供。
  → **证据**：`--no-default-features` 退出码 0；分配次数两条断言；对照 m7 的 bump。

- **对应实验**：crate 构建；`parse_borrowed_view_allocates_zero` / `boxing_parser_allocates_once`

---

## 与后续学习的关联

| C-ID | 关联 |
|------|------|
| C-01 | skb / xdp 数据是别人拥有的页，解析器通常只借 |
| C-02 | 把 skb 所有权交给协议栈是 move，不是 memcpy |
| C-03 | verifier 看到的是借用式访问，不是别名随便写 |
| C-04 | 包缓冲的寿命必须覆盖整个解析；悬空视图是漏洞 |
| C-05 | 协议头是 C 结构体 + 判别字段；Rust enum 是逻辑层 |
| C-06 | Aya 用 trait 抽象 map / 上下文；对象安全决定能不能 `dyn` |
| C-07 | 单态化 vs vtable 是 eBPF 指令预算和用户态热路径的分叉 |
| C-08 | 内核返回码 / XDP action 都是带标签的结果，不是 bool |
| C-09 | NAPI 轮询是"拉下一个"，和迭代器同一形状 |
| C-10 | 回调式 filter / map 的捕获决定能不能进 per-cpu 上下文 |
| C-11 | 用户态 AF_XDP umem 是独占登记的内存，不是随便 `Rc` |
| C-12 | per-cpu 指针能不能 `Send` 进工作线程，编译器说了算 |
| C-13 | 数据面尽量无锁；起线程是控制面 / 用户态的事 |
| C-14 | 统计、ring 的 producer/consumer 用原子，不是 Mutex |
| C-15 | eBPF 里几乎没有 `unsafe` 逃生口；用户态驱动里有，必须写不变量 |
| C-16 | DMA 地址、umem 描述符都是裸指针 |
| C-17 | 按字节偏移读头，等价于指针加法，必须先证明在分配内 |
| C-18 | 网卡缓冲对齐弱于 `u64`；未对齐读是常态 |
| C-19 | 同一块 umem 上的别名规则，错了就是数据竞争 |
| C-20 | 安全封装的目标：调用方拼不出越界切片 |
| C-21 | 内核 ABI、`struct xdp_md`、syscall —— m6 已练，本模块只定位布局 |
| C-22 | eBPF 程序是比 `no_std` 更窄的环境 |
| C-23 | 内核模块有 core 式原语，没有 `std::fs` |
| C-24 | eBPF 没有通用堆；本模块的 bump/Vec 前提在那里不成立（见 m7） |
