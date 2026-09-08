# Module 5 —— OBSERVATIONS

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
| `command` | （按各记录块内的命令为准） |

> 基线：[../../acceptance/environment-baseline.md](../../acceptance/environment-baseline.md)
> 本块字段与基线一致，无差异。

---

## 记录块

### C-15 / c15_unsafe  [NON-ASSERTION]

命令：`cargo run -p m5-unsafe --example c15_unsafe`

输出：

```text
=== 边界内 get_unchecked ===
  idx 2 = Some(30)
  idx 9 (checked miss) = None
```

解释：
  为什么会这样：`get_in_bounds` 先走安全 `get`。下标 2 合法，才进入 `get_unchecked`，
  读到 30。下标 9 在 `get` 处变成 `None`，根本不进 `unsafe`。
  这不能证明什么：打印 `Some(30)` 不能证明"所有 `get_unchecked` 都安全"，
  也不能证明没跑 Miri 时没有 UB。安全侧的 `clean` 见 UB 判定表。

架构相关性：可跨架构推广。边界检查与 `get_unchecked` 的义务是语言规则，与 endian / 字长无关。

---

### C-15 / c15_unsafe_ub  [NON-ASSERTION]

命令：`cargo run -p m5-unsafe --example c15_unsafe_ub`

输出（debug 构建，本机）：

```text
thread 'main' panicked at examples/c15_unsafe_ub.rs:16:27:
unsafe precondition(s) violated: slice::get_unchecked requires that the index is within the slice
...
Aborted (core dumped)
```

解释：
  为什么会这样：1.98.0 的 `ub_checks` 在 debug 构建里把"越界 get_unchecked"变成一次
  可选的运行期断言。这是实现细节，文档写明它**不能**被当成安全保证。
  这不能证明什么：abort **不能**证明"UB 就是崩溃"。同一源码交给 Miri，报的是
  `` `assume` called with `false` ``（W2 别名），那才是本实验的判定。
  release / 关掉检查之后，本机也可能不崩。

架构相关性：可跨架构推广。越界 `get_unchecked` 在语言里一律是 UB；
debug abort 是否出现取决于 `ub_checks`，不是架构。

---

### C-16 / c16_raw_ptr  [NON-ASSERTION]

命令：`cargo run -p m5-unsafe --example c16_raw_ptr`

输出：

```text
=== addr_of / read / write ===
  write 9 then read = 9
  two addr_of same place = true
```

解释：
  为什么会这样：`addr_of_mut!` 取址不创建引用，`write` 把 9 写入仍然活着的 `i32`，
  `read` 读回同一位置。两次 `addr_of!` 指向同一局部变量。
  这不能证明什么：打印 9 不能证明"所有裸指针读写都安全"。悬垂指针那一侧在对照实验。

架构相关性：可跨架构推广。值圆回与"同一位置"是语言规则。

---

### C-16 / c16_raw_ptr_ub  [NON-ASSERTION]

命令：`cargo run -p m5-unsafe --example c16_raw_ptr_ub`

输出：

```text
use-after-free read (NON-ASSERTION) = -1911247082
```

解释：
  为什么会这样：`Box` 被拼回去并 drop 之后，堆槽被分配器收回。本机这一次读到了
  一个看起来像随机的 `i32`。指针值还在，对象没了。
  这不能证明什么：这个具体数字不能断言，也不能证明"还能读所以没问题"。
  Miri 报 W6 `has been freed`。换一次运行、换一个分配器，打印会变。

架构相关性：仅适用于记录本次 x86_64 glibc 分配器的表象。
UB 本身可跨架构推广；打印出的垃圾值不可推广。

---

### C-17 / c17_ptr_arith  [NON-ASSERTION]

命令：`cargo run -p m5-unsafe --example c17_ptr_arith`

输出：

```text
=== 分配内 add / offset / wrapping_add ===
  read_at(1) = Some(20)
  add == wrapping_add (one step) = true
  offset == add = true
  one-step bytes = 4
  wrapping_add(100) produced a pointer (do not deref): 0x7ffd...
```

解释：
  为什么会这样：三个 `u32` 里走一步读到 20。分配内 `add` / `offset` / `wrapping_add`
  得到同一指针。`wrapping_add(100)` 按文档本身安全，所以能打印出一个地址。
  这不能证明什么：打印那个外指针不能证明可以解引用它。具体地址是 NON-ASSERTION。

架构相关性：可跨架构推广。分配内偏移与 `wrapping_add` 的分工是语言规则。
打印出的绝对地址不可推广。

---

### C-17 / c17_ptr_arith_ub  [NON-ASSERTION]

命令：`cargo run -p m5-unsafe --example c17_ptr_arith_ub`

输出：

```text
oob add pointer (NON-ASSERTION) = 0x7ffe...
```

解释：
  为什么会这样：本机 `add(8)` 只是算出一个地址并打印，debug 没有像 `get_unchecked`
  那样插入前置检查。语言上这一步已经是 UB。
  这不能证明什么：打印出地址不能证明 `add` 越界合法。Miri 报 W7
  `in-bounds pointer arithmetic failed`，不必解引用。

架构相关性：可跨架构推广（UB 规则）。本机没 abort 是实现细节，不可推广成"算出来就没事"。

---

### C-18 / c18_alignment  [NON-ASSERTION]

命令：`cargo run -p m5-unsafe --example c18_alignment`

输出：

```text
=== align_of / aligned read / read_unaligned ===
  align_of::<u64>() = 8
  aligned addr % align = 0
  read_aligned(7) = 7
  read_unaligned u16 at +1 = 0x1234
```

解释：
  为什么会这样：本机 `u64` 对齐要求是 8。独立局部变量满足取模 0。普通 `read` 读回 7。
  偏移 1 处的两字节按本机小端解释为 `0x1234`（`from_ne_bytes([0x34, 0x12])`）。
  这不能证明什么：`align_of::<u64>() == 8` 不能推广到所有架构（参考文档自己的警告）。
  `0x1234` 依赖小端，断言里用的是 `from_ne_bytes`，不是写死的跨端数值。

架构相关性：仅适用于 x86_64。原因：本机 SysV ABI 上 `align_of::<u64>() == 8`；
`read_unaligned` 的合法性可跨架构推广，但打印出的 `0x1234` 随 endian 变。

---

### C-18 / c18_alignment_ub  [NON-ASSERTION]  （核心教学对照）

命令：`cargo run -p m5-unsafe --example c18_alignment_ub`

输出：

```text
2
```

解释：
  为什么会这样：8 字节缓冲从偏移 1 起写 `u64` 值 2。x86_64 容忍未对齐访问，
  且这一次写碰巧没立刻把相邻栈槽用一种可见的方式毁掉，所以打印 2。
  这不能证明什么：**绝对不能**证明该写合法，也不能证明在 aarch64 上不会 SIGBUS。
  同一源码 Miri 报 W2 `memory access failed`（只剩 7 字节却写 8 字节）。
  未对齐是同一操作的另一面；Miri 先报越界长度。这正是 FR-019 存在的理由。

架构相关性：仅适用于 x86_64。原因：本机 TSO + 容忍未对齐，把 UB 藏成"打印 2"。
aarch64 上同类构造可能 SIGBUS，不能用这一次打印做跨架构结论。

---

### C-19 / c19_aliasing  [NON-ASSERTION]

命令：`cargo run -p m5-unsafe --example c19_aliasing`

输出：

```text
=== UnsafeCell sequential writes ===
  write_twice(0, 1, 2) = 2
```

解释：
  为什么会这样：两次写通过同一裸指针顺序发生，没有两份同时活着的 `&mut`。
  `UnsafeCell` 允许这条通路。读回最后一次写入 2。
  这不能证明什么：打印 2 不能证明"任何通过 `get()` 的写都合法"。叠两份 `&mut` 在对照侧。

架构相关性：可跨架构推广。内部可变性规则是语言规则。

---

### C-19 / c19_aliasing_ub  [NON-ASSERTION]

命令：`cargo run -p m5-unsafe --example c19_aliasing_ub`

输出：

```text
overlapping mut writes (NON-ASSERTION) = 2
```

解释：
  为什么会这样：本机没有别名模型检查，两份 `&mut` 只是两份地址，最后内存里是 2。
  这不能证明什么：打印 2 不能证明叠 `&mut` 合法。Miri SB 报 W8，TB 报 W9，
  两套都判 UB（见 UB 判定表）。一次硬件写入顺序不能当模型结论。

架构相关性：可跨架构推广。别名规则是语言（实验性模型）规则，不是微架构。
本机打印 2 是硬件表象，不可推广。

---

### C-20 / c20_mem_safety  [NON-ASSERTION]

命令：`cargo run -p m5-unsafe --example c20_mem_safety`

输出：

```text
=== safe parse / from_raw_parts wrapper ===
  parse off 0 = Some(4660)
  parse off 3 (miss) = None
  as_u16_slice len = Some(2)
  odd len rejected = None
```

解释：
  为什么会这样：`0x1234 = 4660`。越界 `get` 返回 `None`。对齐后的 4 字节看成两个 `u16`。
  奇数长度在进入 `from_raw_parts` 之前被拒绝。
  这不能证明什么：打印 `Some(2)` 不能证明任意字节切片都能当 `u16` 切片。
  未对齐起始会被 `as_u16_slice` 拒绝，example 用 `repr(align(2))` 钉住了演示缓冲。

架构相关性：可跨架构推广。按字节拼大端与拒绝奇数长度是语言 / 我们自己的约定。
`u16` 对齐在几乎所有目标上都是 2；演示缓冲用 `repr(align(2))` 钉住，避免把栈对齐运气写进结论。

---

### C-20 / c20_mem_safety_ub  [NON-ASSERTION]

命令：`cargo run -p m5-unsafe --example c20_mem_safety_ub`

输出：

```text
uninit via lying Vec (NON-ASSERTION) = 0
```

解释：
  为什么会这样：`set_len` 只改长度。本机这一次未初始化堆槽碰巧是 0。
  函数没标 `unsafe`，调用方只是读 `v[0]`。
  这不能证明什么：打印 0 不能证明封装安全，也不能证明"未初始化的 `u8` 就是 0"。
  Miri 报 W10。这是"作者撒谎、调用方没做错"的调用序列。

架构相关性：仅适用于记录本次分配器把新槽留成 0 的表象。UB 本身可跨架构推广。

---

### C-20 / c20_bounds_check  [NON-ASSERTION]  （T090 / US5 AS4）

命令：`cargo run -p m5-unsafe --example c20_bounds_check`

输出：

```text
=== bounds-checked parse ===
  off 0 = Some(2048)
  off 3 (would be oob) = None
=== same offset, check then unchecked ===
  checked = Some(17664)
  unchecked after check = Some(17664)
  removing the check and using off=3 is the UB path ...
```

解释：
  为什么会这样：`0x0800 = 2048`，`off=3` 只剩 1 字节，检查返回 `None`。
  `off=2` 检查通过后再走 unchecked，两个字节是 `0x4500 = 17664`，两侧一致。
  这不能证明什么：两侧一致不能证明"可以拿掉检查"。拿掉之后用 `off=3` 就是 UB，
  本机不一定崩（回到 C-15/C-18）。这不能证明 eBPF verifier 的任何具体输出 ——
  那是书面推导，见 `concept.md` C-20 关联小节，本 Feature 不写 eBPF（FR-017）。

架构相关性：可跨架构推广。检查与否的义务是语言规则。verifier 对应关系是书面推导，
不构成对 verifier 行为的断言。

---

## UB 判定记录

未运行工具时只能记 `n/a`，MUST NOT 记 `clean`（FR-019）。
本轮两套 Miri 均已运行。

| 实验 | 事前预测（`PREDICT-UB`） | 工具与命令 | 实际类别 | `ub_verdict` | 命中? |
|------|----------------------|-----------|---------|-------------|-------|
| `c15_unsafe` | clean | `cargo +nightly miri run -p m5-unsafe --example c15_unsafe` | 无 UB | clean | ✓ |
| `c15_unsafe_ub` | W1 + W2 | `cargo +nightly miri run -p m5-unsafe --example c15_unsafe_ub` | W1 + `` `assume` called with `false` ``（W2 别名） | expected-ub | ✓ |
| `c16_raw_ptr` | clean | 同上模式 | 无 UB | clean | ✓ |
| `c16_raw_ptr_ub` | W1 + W6 | 同上 | W1 + `has been freed` | expected-ub | ✓ |
| `c17_ptr_arith` | clean | 同上 | 无 UB | clean | ✓ |
| `c17_ptr_arith_ub` | W1 + W7 | 同上 | W1 + `in-bounds pointer arithmetic failed`（W7 别名） | expected-ub | ✓ |
| `c18_alignment` | clean | 同上 | 无 UB | clean | ✓ |
| `c18_alignment_ub` | W1 + W2 | 同上 | W1 + `memory access failed` | expected-ub | ✓ |
| `c19_aliasing` | clean | SB 默认 + TB | 两轮均无 UB | clean | ✓ |
| `c19_aliasing_ub` | W1 + W8（SB）；W1 + W9（TB） | `cargo +nightly miri run …` 与 `MIRIFLAGS=-Zmiri-tree-borrows` | SB：W8 borrow stack；TB：W9 `foreign write` | expected-ub | ✓ |
| `c20_mem_safety` | clean | 同上 | 无 UB | clean | ✓ |
| `c20_mem_safety_ub` | W1 + W10 | 同上 | W1 + `memory is uninitialized`（W10 别名） | expected-ub | ✓ |
| `c20_bounds_check` | clean | 同上 | 无 UB | clean | ✓ |

全模块扫描：

```text
cargo +nightly miri test -p m5-unsafe                 # Stacked Borrows：全绿
MIRIFLAGS="-Zmiri-tree-borrows" cargo +nightly miri test -p m5-unsafe  # Tree Borrows：全绿
```

两轮均退出码 0。安全侧测试在 Miri 下直接执行；对照侧经 `run_example` 子进程
（`cargo miri test` 内子进程不可用，这些断言跳过，由 `cargo test` 覆盖）。

### C-19 双模型

本实验属于 plan.md 情形 1：两套都报 UB，`ub_verdict = expected-ub`，结论稳健。
**未**进入模型敏感（情形 3/4），故无 `[MODEL-SENSITIVE]` 块。
差异仍有教学价值：SB 在写第一份时爆发（tag 已从 stack 消失），
TB 在写第二份时爆发（tag 已被 foreign write 置 Disabled）。
断言分两条，绑定各自模型，不断言"两轮相同"。

---

## 可推广性汇总

| 记录块 | 判定 |
|--------|------|
| 语言规则类（C-15 义务、C-16 圆回、C-17 分配内、C-19 顺序写、C-20 检查） | 可跨架构推广 |
| C-18 打印 `2`、C-16/C-20 UB 的本机垃圾/零 | 仅适用于 x86_64 本次运行的表象 |
| C-18 `align_of::<u64>() == 8` | 仅适用于当前 ABI |
