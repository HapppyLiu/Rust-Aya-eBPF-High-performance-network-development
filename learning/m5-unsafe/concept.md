# Module 5: Unsafe Rust 与裸指针内存模型

**Story**: US5（P1，Feature 002 硬前置） | **Capabilities**: C-15…C-20 | **Prerequisite**: m4（accepted）

> 本文件属于 **Answer Track**。对应的 Learner Track 是
> [`learner/m5-unsafe/guide.md`](../../learner/m5-unsafe/guide.md)。
> 如果你还没做过那边的预测表，先去做 —— 这份文件读过之后就不能再自测了。

## 这个模块回答什么问题

1. `unsafe` 打开的是义务还是豁免？写"语言要求这里用 unsafe"算不算 Safety 注释？
2. 普通运行没有崩溃、甚至打印了一个"合理"的数，能不能当成没有未定义行为？
3. 裸指针的读/写、加减、对齐，各自把哪一条规则从编译器手里接过来？
4. 两份可变引用叠在同一块内存上，两套别名模型怎么判？不一致时取哪一侧？
5. 怎样判断一个"对外安全"的封装是真安全，而不是把炸弹藏在接口后面？

---

## 概念

### C-15 Unsafe Rust

- **一句话定义**：`unsafe` 把编译器代劳的若干检查改成调用方必须书面保证的义务。
  它不是"这段代码没有规则"。

- **底层机制**：

  `slice::get_unchecked`（`core/src/slice/mod.rs:640`）是 `unsafe fn`。
  Safety 段写明：下标越界是未定义行为，**即使结果立刻丢掉**（`:614-619`）。
  安全路径 `get` 返回 `Option`；走 `get_unchecked` 之前必须已经证明 `idx < len`。

  `get_in_bounds` 先 `slice.get(idx)?`，再进入 `unsafe`。
  那一次 `get` 就是证明。SAFETY 注释要把有效性/对齐/别名/provenance/生命周期五件事写完，
  不适用的写原因。写"Rust 要求 unsafe"会被契约拒绝（§C6.3）。

  **UB 不等于崩溃。** 语言把一类操作定义为没有意义。崩溃、打印出数、看起来正常，
  都只是这一次运行的表象。未运行 UB 工具时 `ub_verdict` 只能是 `n/a`（FR-019）。
  本工具链上，越界 `get_unchecked` 在 debug 构建里会先撞上 `ub_checks` 的
  `assume(false)`；Miri 报的是 W2 的该别名，不是"程序崩了所以是 UB"。

  UB 定义本身在语言参考里，不在 `core` 库代码中，按 FR-005 记 `reference-fallback`。

- **常见误解**：
  - **误解**："`unsafe` 就是关闭检查，所以更快也更自由"
    → **实际**：检查改由你保证。保证不了就不要写。
    → **证据**：`get_unchecked_in_bounds_matches_index` 对照 `get_unchecked_oob_is_ub_under_miri`。
  - **误解**："没崩溃就是没有 UB"
    → **实际**：判定依据是 Miri 类别，不是退出码 0。
    → **证据**：OBSERVATIONS「C-15 UB：debug 会 abort，那也不是定义」；FR-019。

- **对应实验**：[`c15_unsafe`](../../experiments/m5-unsafe/examples/c15_unsafe.rs)
  / [`c15_unsafe_ub`](../../experiments/m5-unsafe/examples/c15_unsafe_ub.rs)
  / [tests](../../experiments/m5-unsafe/tests/c15_unsafe.rs)

---

### C-16 Raw pointer

- **一句话定义**：裸指针记下地址，不延长借用，也不阻止分配被释放。
  `ptr::read` / `write` 要求这块内存此刻仍然有效、对齐、已初始化。

- **底层机制**：

  `addr_of!`（`core/src/ptr/mod.rs:2729`，等价于 `&raw const`）取址但不创建引用，
  因此不触发借用冲突。随后的 `ptr::write`（`:1916`）和 `ptr::read`（`:1692`）
  是 typed 操作：对齐、有效性、初始化都在调用方头上。

  `write_then_read` 对栈上 `i32` 写再读，断言的是**值圆回来**，以及两次取址是
  **同一位置**（`addrs_of`），不断言具体地址数字（§C2.2）。

  释放后再读：`Box::into_raw` 交出指针，`Box::from_raw` 拼回去并 `drop`，
  然后再 `ptr::read`。指针值还在，分配已经结束。Miri 报 W1 + W6（`has been freed`）。
  本机这一次打印出了一个垃圾数 —— 那是这一次分配器的运气，不是"还能读"。

- **常见误解**：
  - **误解**："指针还在，所以内存还在"
    → **实际**：指针是整数大小的地址加 provenance。分配结束，provenance 指向的对象没了。
    → **证据**：`use_after_free_is_ub_under_miri`；OBSERVATIONS「C-16 UB：本机打印垃圾」。

- **对应实验**：[`c16_raw_ptr`](../../experiments/m5-unsafe/examples/c16_raw_ptr.rs)
  / [`c16_raw_ptr_ub`](../../experiments/m5-unsafe/examples/c16_raw_ptr_ub.rs)

---

### C-17 Pointer arithmetic

- **一句话定义**：`add` / `offset` 要求结果仍在**同一次分配**的可访问范围内
  （含 one-past）；越出本身就是 UB，不必解引用。`wrapping_add` 本身永远安全，
  只是得到的外指针不能解引用。

- **底层机制**：

  `const_ptr.rs:838` 的 `add` 是 `unsafe`。文档与 Miri 都把"算出越界指针"
  当成立即 UB。本工具链 Miri 1.100 的措辞是 `in-bounds pointer arithmetic failed`
  （W7 别名）：它说的是"本应保持 in-bounds 的算术失败了"。

  `wrapping_add`（`:1035`）Safety 第一句：本操作永远安全。它"推迟"了分配内约束，
  直到有人解引用。分配内走一步，`add(1)` 与 `wrapping_add(1)` 得到同一指针
  （`wrapping_add_agrees_inside_allocation`）。走 100 步的 `wrapping_add` 只产生
  一个不能碰的指针，example 打印它但不解引用。

  可断言的是元素值和**字节差等于 `size_of::<u32>()`**，不是具体地址。

- **常见误解**：
  - **误解**："只要不解引用，指针运算怎么挪都没事"
    → **实际**：`add` 越界立刻犯规。`wrapping_add` 才是"先挪着、用的时候再算"。
    → **证据**：`add_past_allocation_is_ub_under_miri` 对照 example 里不解引用的
    `wrapping_add(100)`。

- **对应实验**：[`c17_ptr_arith`](../../experiments/m5-unsafe/examples/c17_ptr_arith.rs)
  / [`c17_ptr_arith_ub`](../../experiments/m5-unsafe/examples/c17_ptr_arith_ub.rs)

---

### C-18 Alignment

- **一句话定义**：每种类型有 ABI 要求的对齐；普通 `read`/`write` 要求指针满足它。
  硬件宽容不等于语言合法。

- **底层机制**：

  `align_of::<T>()`（`mem/mod.rs:540`）返回类型要求，不是某块内存的实测。
  独立 `u64` 局部变量满足 `(p as usize) % align_of::<u64>() == 0`
  （`local_u64_is_aligned`）。未对齐地址用 `read_unaligned`（`ptr/mod.rs:1810`），
  它把对齐义务卸掉，有效性/provenance 还在。

  **核心教学对照**（T084 / research R-02）：8 字节缓冲、从偏移 1 起写 `u64` 2。
  本机 x86_64 正常退出并打印 `2`。同一源码 Miri 报 W1 + W2
  （`memory access failed`：偏移 1 起只剩 7 字节）。未对齐是同一操作的另一面；
  Miri 先报越界长度。打印 `2` **MUST NOT** 当成无 UB。

- **常见误解**：
  - **误解**："x86_64 能跑完的未对齐读写就是合法的"
    → **实际**：硬件容忍是微架构事实。语言规则仍要求对齐或走 `read_unaligned`。
    → **证据**：`misaligned_u64_is_ub_under_miri`；普通运行打印 2。
  - **误解**："ASan 没报告也等于没有 UB"
    → **实际**：那是 US6 的假设（CHK045），强度弱于本模块的 Miri 判定。本能力不用 ASan。

- **对应实验**：[`c18_alignment`](../../experiments/m5-unsafe/examples/c18_alignment.rs)
  / [`c18_alignment_ub`](../../experiments/m5-unsafe/examples/c18_alignment_ub.rs)

---

### C-19 Aliasing

- **一句话定义**：共享引用默认不许改里面。`UnsafeCell` 是内部可变性的根：
  它交出裸指针，并把"不要叠两份冲突引用"交还给调用方。

- **底层机制**：

  `UnsafeCell<T>`（`cell.rs:2323`）`#[repr(transparent)]`，`get`（`:2443`）是**安全**函数，
  返回 `*mut T`。它自己 `!Sync`（`:2328`）。合法用法是同一时刻只有一条通路在写，
  像 `write_twice` 那样顺序写。Miri 两套模型都判 `clean`。

  对照：从同一裸指针造两份 `&mut`，再先后写入。安全 Rust 里这是两次 `&mut`，编不过。
  用裸指针绕过去之后，缺的那道检查改由别名模型来做。

  - **Stacked Borrows**（默认）：第二份 Unique retag 使第一份 tag 从 borrow stack 消失；
    再写第一份 → W8 `does not exist in the borrow stack`。
  - **Tree Borrows**：第一份写入使第二份 tag 转到 `Disabled`；再写第二份 → W9 `foreign write`。

  本实验两套都报 UB（plan.md C-19 情形 1），`ub_verdict = expected-ub`，结论稳健。
  没有进入"模型敏感"。两轮各写一条断言，不断言"两轮必须相同"（规则 3）。

- **常见误解**：
  - **误解**："`UnsafeCell::get` 是 unsafe，所以用它就自动合法"
    → **实际**：`get` 是安全的，它只交出指针。犯规发生在你怎么用这个指针。
    → **证据**：`source-refs.md` `:2443`；`unsafecell_sequential_writes` 对照
    `overlapping_mut_is_ub_under_stacked_borrows`。

- **对应实验**：[`c19_aliasing`](../../experiments/m5-unsafe/examples/c19_aliasing.rs)
  / [`c19_aliasing_ub`](../../experiments/m5-unsafe/examples/c19_aliasing_ub.rs)

---

### C-20 Memory safety

- **一句话定义**：安全抽象的意思是：调用方即使用尽公开 API，也拼不出 UB。
  对内可以有 `unsafe`；对外必须把不变量守住。

- **底层机制**：

  `slice::from_raw_parts`（`slice/raw.rs:124`）把裸指针 + 长度拼成切片。
  长度、对齐、来源、寿命全在调用方头上。`as_u16_slice` 在公开 API 里先拒绝奇数长度
  和未对齐起始，再进入 `unsafe`。调用方给奇数切片只能得到 `None`。

  `parse_u16_be_checked` 用 `get` 做边界检查。`parse_u16_be_unchecked` 是 `unsafe fn`，
  不对外作为"安全入口"。`c20_bounds_check` 展示：先检查，再走裸指针，结果相同。

  **撒谎的封装**（US5 AS3 后半）：`lying_bytes` 不标 `unsafe`，内部 `Vec::set_len`
  （`alloc/src/vec/mod.rs:2224`）把长度改成容量却不初始化。调用方按普通 `Vec<u8>` 读
  `v[0]` —— 调用方没做错。Miri 报 W1 + W10（`memory is uninitialized`）。
  本机这一次打印 `0`，那是未初始化内存碰巧是零，不是封装安全。

- **常见误解**：
  - **误解**："函数没标 `unsafe`，所以怎么调用都安全"
    → **实际**：没标只说明作者**声称**安全。要看是否存在合法调用序列会穿帮。
    → **证据**：`lying_set_len_is_ub_under_miri`。
  - **误解**："去掉边界检查，本机没崩，所以检查是多余的"
    → **实际**：见下方关联小节与 `c20_bounds_check`。

- **对应实验**：[`c20_mem_safety`](../../experiments/m5-unsafe/examples/c20_mem_safety.rs)
  / [`c20_mem_safety_ub`](../../experiments/m5-unsafe/examples/c20_mem_safety_ub.rs)
  / [`c20_bounds_check`](../../experiments/m5-unsafe/examples/c20_bounds_check.rs)

---

## 与后续学习的关联

FR-014 要求每项能力说明与后续 Linux / eBPF / Aya 的关联点，或显式标注为仅是基础。
US5 AS4 / T004：C-20 的对应关系是**书面推导**，MUST NOT 编写 eBPF 程序（FR-017）。

| C-ID | 关联点 |
|------|-------|
| **C-15 Unsafe Rust** | 直接对应。后续读内核 / 驱动 / Aya 绑定里的 `unsafe` 块，第一件事就是写五要素，
  而不是猜"这里为什么要 unsafe"。本模块把义务钉住。 |
| **C-16 Raw pointer** | 直接对应。报文缓冲、DMA、map value 最终都是地址。释放后再读是用户态
  把内核缓冲寿命搞错时的典型形态。 |
| **C-17 Pointer arithmetic** | 直接对应。packet parsing 就是在一块缓冲里按偏移走路。
  `add` 越界本身犯规，和"读到垃圾"不是同一时刻。 |
| **C-18 Alignment** | 直接对应。协议头按 2/4 字节对齐；x86_64 上未对齐访问常常不崩，
  弱序 / 嵌入式 / 部分加速器上会 SIGBUS。本实验证明"没崩 ≠ 合法"。 |
| **C-19 Aliasing** | 仅为理解基础，不直接对应后续某一条 API。但共享 map 值、内部可变的
  用户态缓存，背后都是"同一时刻几份可变"。C-12 的 `UnsafeCell: !Sync` 从这里长出来。 |
| **C-20 Memory safety** | **AS4 书面对应关系（三项合取）**： |
| | **(i)** `c20_mem_safety_ub` / `c18_alignment_ub` 说明越界或未对齐在本机不一定崩溃
  （打印 `0` 或 `2`）。那是 UB 的表象，不是"没问题"。 |
| | **(ii)** 证明义务的三个阶段：Rust **运行时**靠 `get` / `parse_u16_be_checked` 这类显式检查；
  Rust **编译期**靠类型系统不让两份 `&mut` 叠上；eBPF verifier 在**加载时**做静态边界证明，
  拒绝无法证明安全的程序。三者把同一件"不要走出缓冲"放在不同阶段。 |
| | **(iii)** 因此同一段"从偏移读两个字节"的解析，在 eBPF 环境会被要求在加载前就能证明
  `off + 2 <= 包长度`。本实验对应 `c20_bounds_check.rs` 里 `parse_u16_be_checked(&pkt, 3)`
  返回 `None` 的那一行 —— 拿掉这一行检查、改走 unchecked，就是对照侧的 UB。
  本 Feature 不写 eBPF，也不把 verifier 输出当证据。强度：书面推导，低于本模块其余 Miri 判定。 |

C-19 标了"仅为理解基础"中的别名模型细节；内部可变性根类型本身仍是读后续共享代码的入口。
C-15…C-18、C-20 都不是"仅为理解基础"。读不懂 Safety 不变量和"没崩 ≠ 没 UB"，
就不能进 Feature 002（FR-012）。
