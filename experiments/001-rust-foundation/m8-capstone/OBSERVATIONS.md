# Module 8 —— OBSERVATIONS

本文件里的一切都是 **NON-ASSERTION**：它的差异不参与一致性判定（§C8.1）。
稳定断言在 `tests/`，两者物理隔离（R-05）。

## 环境记录

| 字段 | 值 |
|------|-----|
| `rustc_stable` | 1.98.0 (88d9e12ae 2026-08-18) |
| `rustc_nightly` | 1.100.0-nightly (17fd5b8a3 2026-08-28) |
| `edition` | 2024 |
| `kernel` | 6.6.114.1-microsoft-standard-WSL2 |
| `arch` | x86_64 |
| `target` | x86_64-unknown-linux-gnu |
| `command` | cargo test -p m8-capstone |

> 基线：[../../../acceptance/001-rust-foundation/environment-baseline.md](../../../acceptance/001-rust-foundation/environment-baseline.md)
> 本块字段与基线一致。

---

## 记录块

### C-22 / no_std 库构建  [NON-ASSERTION]

命令：`cargo build -p m8-capstone --no-default-features`

输出：

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s)
BUILD_OK
```

解释：
  为什么会这样：库 crate 写了 `#![no_std]` 和 `extern crate alloc`，没有引用
  `std::thread` / `std::fs`。`--no-default-features` 关掉空的 `std` feature，
  构建的仍是 host target 上的 rlib，不是 `x86_64-unknown-none` 可执行文件。
  这不能证明什么：不能证明裸机可执行、不能证明有 heap、不能证明有 panic 处理。
  那三件是最终二进制的义务（对照 m7）。也不能证明"任何 no_std 环境都能跑这个解析器"——
  eBPF 连 alloc 都没有。

架构相关性：可跨架构推广。`no_std` 库在 host target 上产出 rlib 是工具链行为，
不是 x86_64 特有；换架构同样不能把"编过"读成"裸机能跑"。

---

### US8 / example capstone  [NON-ASSERTION]

命令：`cargo run -p m8-capstone --example capstone`

输出：

```text
buffer_len=19
first kind=Data payload_len=4 rest_len=9
frames_ok=2 stats_frames=2
truncated=Truncated
```

解释：
  为什么会这样：两帧分别是 6+4 与 6+3 字节，总和 19。第一次 `parse_frame` 只吃第一帧，
  剩余 9 字节是第二帧。迭代器走两帧，`ParseStats` 用 Relaxed 原子加到 2。
  把头截到 3 字节触发 `Truncated`，因为 3 < `HEADER_SIZE`。
  这不能证明什么：打印的 `kind=Data` 不是稳定断言（稳定断言在 `tests/` 比枚举）。
  也不能由"程序打印了 Truncated"推出"过短切片没有 UB"——那要看 Miri 记录块。

架构相关性：可跨架构推广。帧长度由编码函数显式小端写出，不依赖本机端序；
`kind` 是逻辑枚举。换到 big-endian 主机，example 仍应打印同一组逻辑结果
（若不同，是编解码 bug，不是架构特性）。

---

### US8 / cargo test  [NON-ASSERTION]

命令：`cargo test -p m8-capstone`

输出（摘要）：

```text
test result: ok. 14 passed; 0 failed;  ...  (tests/capstone.rs)
test result: ok. 4 passed; 0 failed;   ...  (tests/raw.rs)
test result: ok. 1 passed; 0 failed;   ...  (tests/wire.rs)
```

共 19 个 integration test。分配次数、布局数字、错误变体在测试源码里断言，
本块只记录"全绿"这一现象。

解释：
  为什么会这样：每条 `CLAIM` 绑定一个确定性量（变体、长度、`allocs`、`size_of`）。
  测试 crate 默认有 std，所以能注册 `CountingAllocator`、能 `thread::scope`。
  这不能证明什么：全绿不等于 Miri clean（那是下一块）。也不等于 24 项定位无遗漏
  （那是 `capability-location-map.md`）。`cargo test --workspace` 会跑到本包，
  但 m7 仍不在 workspace 内。

架构相关性：可跨架构推广。断言对象是枚举变体、相对指针相等、布局关系、分配次数；
未断言绝对地址或耗时。`WireHeader` 的 6/2/4 是这组 `repr(C)` 字段在常见 ABI 上的数字，
换 ILP32 可能变 —— 稳定断言同时比 `size_of` 与 `HEADER_SIZE` 的相等关系，
因此若某架构插入填充，const 断言会在编译期拦住，而不是静默印错数。

---

## UB 判定记录

事前预测：综合实验只含**安全路径**，预期 `clean`。未运行时只能记 `n/a`（FR-019）。

| 实验 | 事前预测 | 工具与命令 | 实际类别 | `ub_verdict` | 命中? |
|------|---------|-----------|---------|-------------|-------|
| `cargo test -p m8-capstone` 全部 | clean | `cargo +nightly miri test -p m8-capstone` | 无报告（19 integration + lib/doc 空集） | **clean** | 是 |

命令输出摘要：capstone 14 ok / raw 4 ok / wire 1 ok；Miri 未打印 `Undefined Behavior`。

解释：
  为什么会这样：`raw.rs` 的偏移在块外证明 `offset + size <= len`，`u16` 走
  `read_unaligned`，`from_raw_parts` 的长度等于已检查的子区间。别名只有共享只读。
  测试里的 `thread::scope` 只碰 `AtomicUsize`。
  这不能证明什么：Miri clean 覆盖的是**这次跑到的路径**（Stacked Borrows 默认模型），
  不是"任意调用方永远无 UB"。公开 API 若被改成跳过检查，预测必须重写，且按
  experiment-contract §C5.1a 先记 PREDICTION-MISS，不得就地改预期。
  也没有跑 Tree Borrows 对照 —— 那是 C-19 在 m5 的义务，本综合实验的安全路径
  不含 m5 那种保留借用冲突。

架构相关性：可跨架构推广。Miri 解释的是 Rust 别名与有效性规则，不是 x86_64
硬件是否容忍未对齐访问。换架构不得把"本机没崩"当成 clean。

---

## IR 观察

本模块无单独的 MIR/LLVM 义务。分发结构（`dyn ParseHeader` vs 泛型 `parse_then`）
的代价用测试里"两种入口解析出同一个头"表达，不拿 IR 文本当断言。
