# Capability Location Map —— m8-capstone（SC-009）

**Feature**: 001-rust-foundation | **Story**: US8 | **粒度**: **文件 + 函数**
（tasks.md T134 / CHK010：不以"模块名"或行号充数；行号会随编辑漂移）

对照 24 项能力，每一项必须能在综合实验产物**或配套说明**中定位。
空行 = 本 Story 未完成。函数名必须能在所列文件中搜到。

稳定断言：`cargo test -p m8-capstone`（19 项）。
UB：`cargo +nightly miri test -p m8-capstone` → `ub_verdict = clean`（已运行，非 n/a）。

---

| C-ID | Capability | 文件 | 函数 | 在综合实验里体现什么 |
|------|-----------|------|------|---------------------|
| C-01 | Ownership | `experiments/m8-capstone/src/buffer.rs` | `OwnedBytes::from_slice` | `Vec<u8>` 的唯一主人；视图不拥有 |
| C-02 | Move semantics | `experiments/m8-capstone/src/buffer.rs` | `OwnedBytes::into_vec` | 所有权移走，原绑定失效 |
| C-03 | Borrowing | `experiments/m8-capstone/src/buffer.rs` | `OwnedBytes::as_buf` | 从拥有者借出 `PacketBuf` |
| C-04 | Lifetime | `experiments/m8-capstone/src/buffer.rs` | `PacketBuf::new` | `'a` 钉在输入 `&'a [u8]` 上 |
| C-05 | Struct / Enum | `experiments/m8-capstone/src/wire.rs` | `FrameKind::from_wire` | 种类是枚举；未知值失败 |
| C-06 | Trait | `experiments/m8-capstone/src/parse.rs` | `parse_header_dyn` | `&dyn ParseHeader` 走 vtable |
| C-07 | Generic | `experiments/m8-capstone/src/parse.rs` | `parse_then` | `P: ParseHeader + ?Sized` 组合器 |
| C-08 | Error handling | `experiments/m8-capstone/src/error.rs` | `ParseError`（四变体）+ `HeaderParser::parse_header` 中的 `?` | 失败分路径传播 |
| C-09 | Iterator | `experiments/m8-capstone/src/iter.rs` | `FrameIter::next` | 空 → `None`；失败后停止 |
| C-10 | Closure | `experiments/m8-capstone/src/iter.rs` | `map_ok_payloads` | `FnMut` 作用在载荷视图上 |
| C-11 | Smart pointer | `experiments/m8-capstone/src/iter.rs` | `boxed_header_parser` | `Box<dyn ParseHeader>` 独占带状态解析器 |
| C-12 | Send / Sync | `experiments/m8-capstone/src/stats.rs` | `ParseStats`（类型注释）+ `tests/capstone.rs` `views_and_stats_are_send_sync` | 自动 trait；禁止 `unsafe impl` |
| C-13 | Concurrency | `experiments/m8-capstone/src/stats.rs` | `ParseStats::record_ok` | **库内换实现**：原子计数，无 `std::thread`。跨线程演示在 `tests/capstone.rs` `concurrent_stats_end_at_two`（测试 crate 有 OS services） |
| C-14 | Atomic | `experiments/m8-capstone/src/stats.rs` | `ParseStats::record_ok` / `frames` | `AtomicUsize::fetch_add` / `load` |
| C-15 | Unsafe Rust | `experiments/m8-capstone/src/raw.rs` | `read_u8`（及其 `// SAFETY:`） | 检查在块外；块内一次一个 unsafe 操作 |
| C-16 | Raw pointer | `experiments/m8-capstone/src/raw.rs` | `read_u8` | `ptr::read` 自 `*const u8` |
| C-17 | Pointer arithmetic | `experiments/m8-capstone/src/raw.rs` | `read_u16_le` | 边界检查后的 `p.add(offset)` |
| C-18 | Alignment | `experiments/m8-capstone/src/raw.rs` | `read_u16_le` / `try_as_u16_slice` | unaligned 读；未对齐 `u16` 切片被拒 |
| C-19 | Aliasing | `experiments/m8-capstone/src/raw.rs` | `payload_view` | 只构造共享切片，不产生 `&mut` |
| C-20 | Memory safety | `experiments/m8-capstone/src/raw.rs` | `payload_view` | 检查后 `from_raw_parts`；对外安全 |
| C-21 | FFI | `experiments/m8-capstone/src/wire.rs` | `layout` | 见下方**配套说明** |
| C-22 | no_std | `experiments/m8-capstone/src/lib.rs` | crate 属性 `#![no_std]` | 关掉默认 std；不是裸机二进制 |
| C-23 | core / alloc / std | `experiments/m8-capstone/src/lib.rs` | `extern crate alloc` | 集合来自 alloc；`std` 仅测试 |
| C-24 | Panic & allocator | `experiments/m8-capstone/tests/capstone.rs` | `parse_borrowed_view_allocates_zero` / `boxing_parser_allocates_once` | 零拷贝 0 次分配；`Box` 非 ZST 1 次。库不注册 `#[global_allocator]` / `#[panic_handler]`（最终二进制由 std 测试运行器提供） |

**行数校验**：24 行，无空文件、无空函数（C-21 函数存在，义务见说明）。

---

## C-21 配套说明（SC-009 允许"产物或配套说明"）

本 crate **故意**不调用 C、不链 `cc`/`libc`。

- **产物里有什么**：`#[repr(C)] struct WireHeader` + `layout()` 的 size/align/offset 断言
  （`tests/wire.rs` `wire_header_layout`）。这是跨语言对话的**布局前置**。
- **产物里没有什么**：`extern "C"` 函数、syscall、谁分配谁释放。那些已在
  `experiments/m6-ffi` 按 SC-008 验收。
- **为什么这里不做**：库是 `no_std` + `alloc`。再引入 C 工具链会（1）把 C-22 的约束
  搅浑；（2）让"综合实验做过 FFI"变成重复 m6，而不是定位。
- **因此 C-21 的定位** = `wire.rs::layout` **加上本段说明**，不是"有 repr(C) 就算 FFI 完成"。

---

## 因缺少 OS services 而换掉的实现（T131 / C-13 / C-22）

| 能力 | m4/m7 里的形态 | m8 库里换成 | 测试 crate 里 |
|------|----------------|------------|---------------|
| C-13 起线程 | `std::thread::spawn` | `AtomicUsize` 计数 | `thread::scope` 验证终值 |
| C-24 panic 处理 | m7 自己的 `#[panic_handler]` | 不提供（库） | std 测试运行器 |
| C-24 全局分配器 | m7 bump / 测试 `CountingAllocator` | 不注册 | `#[global_allocator]` 仅测试二进制 |
| C-21 真 C 调用 | m6 `cc` + `extern "C"` | 仅布局 | 无 |

`cargo build -p m8-capstone --no-default-features` 成功证明的是：
**这个库在不启用 std 时能链上**。
它**不能**证明：裸机可执行、有 heap、有 panic 处理。那三件事属于最终二进制，见 m7。
