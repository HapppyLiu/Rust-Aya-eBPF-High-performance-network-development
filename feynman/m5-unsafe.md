# Feynman: Module 5 — Unsafe Rust 与裸指针内存模型

**Capabilities covered**: C-15, C-16, C-17, C-18, C-19, C-20

---

## 1. 用自己的话解释

面向"没学过 Rust 但懂 C"的同事。

C 程序员写指针，规则在脑子里：这块还在吗、对齐了吗、有没有别人同时在写、
有没有走出这块缓冲。编译器不怎么管。Rust 把其中大部分收走，改由类型系统在编译期做。
`unsafe` 是把其中几条**交还给你**，并要求你写下来。它不是"这段没有规则"。

**未定义行为不是崩溃的别名。** 语言说这类操作没有意义。你的机器打印出一个漂亮的 `2`，
只说明这一次硬件没以崩溃的形式表现出来。换一台、换一个检查器，故事会变。
没跑检查器的时候，你不能写"所以没有问题"。

裸指针记下地址，不延长这块内存的寿命。所有权走了，指针值可能还在 —— 再读就是
use-after-free。在同一块分配里挪一步可以；挪出这块，`add` 本身已经犯规，
不必真的去碰那个地址。`wrapping_add` 是另一件事：算出外指针没事，用它去读才出事。

对齐是类型的要求，不是"我量了这块内存碰巧能读"。x86_64 常常忍未对齐的 8 字节写，
并让你打印出刚写进去的数。那不是合法。弱一点的架构可能直接 SIGBUS。

共享引用默认不许改里面。有一个透明包装把"改"重新变成你自己协调的事：
同一时刻不要叠两份可变。安全语言里编不过去；用裸指针绕过去之后，
两套实验性的别名模型会在不同的那一行抓住你。本模块这段对照，两套都报了问题。

最后，对外安全的意思是：调用方即使用尽你公开的函数，也拼不出未定义行为。
对内可以有 `unsafe`。若存在一条看起来完全合法的调用会穿帮，那是封装在撒谎，
不是调用方不会用。边界检查就是那条防线：拿掉它，本机不一定崩，
但检查器会告诉你走出了缓冲。后续在加载时做静态证明的那套系统，
会把同一条"不要走出缓冲"提前到加载那一刻。本模块不写那种程序，只把阶段对应写清。

---

## 2. 最小示例

### C-15 Unsafe —— 先证明下标，再走不检查的路

```rust
fn get_in_bounds(s: &[u8], i: usize) -> Option<u8> {
    let _ = s.get(i)?;
    Some(*unsafe { s.get_unchecked(i) })
}
```

完整观察：[`examples/c15_unsafe.rs`](../experiments/m5-unsafe/examples/c15_unsafe.rs)

### C-16 Raw pointer —— 取址、写入、读回

```rust
let mut x = 1i32;
let p = std::ptr::addr_of_mut!(x);
unsafe { std::ptr::write(p, 9) };
assert_eq!(unsafe { std::ptr::read(p) }, 9);
```

完整观察：[`examples/c16_raw_ptr.rs`](../experiments/m5-unsafe/examples/c16_raw_ptr.rs)

### C-17 Pointer arithmetic —— 分配内走一步

```rust
let a = [10u32, 20, 30];
let q = unsafe { a.as_ptr().add(1) };
assert_eq!(unsafe { std::ptr::read(q) }, 20);
```

完整观察：[`examples/c17_ptr_arith.rs`](../experiments/m5-unsafe/examples/c17_ptr_arith.rs)

### C-18 Alignment —— 对齐关系式

```rust
let x = 7u64;
let p = std::ptr::addr_of!(x);
assert_eq!((p as usize) % align_of::<u64>(), 0);
assert_eq!(unsafe { std::ptr::read(p) }, 7);
```

完整观察：[`examples/c18_alignment.rs`](../experiments/m5-unsafe/examples/c18_alignment.rs)

### C-19 Aliasing —— 顺序写，不要叠

```rust
let cell = UnsafeCell::new(0i32);
let p = cell.get();
unsafe { *p = 1 };
unsafe { *p = 2 };
assert_eq!(unsafe { *p }, 2);
```

完整观察：[`examples/c19_aliasing.rs`](../experiments/m5-unsafe/examples/c19_aliasing.rs)

### C-20 Memory safety —— 越界返回空，不硬闯

```rust
fn parse_u16_be(buf: &[u8], off: usize) -> Option<u16> {
    let b = buf.get(off..off.checked_add(2)?)?;
    Some(u16::from_be_bytes([b[0], b[1]]))
}
```

完整观察：[`examples/c20_mem_safety.rs`](../experiments/m5-unsafe/examples/c20_mem_safety.rs)
 / 检查对照：[`examples/c20_bounds_check.rs`](../experiments/m5-unsafe/examples/c20_bounds_check.rs)

---

## 3. 底层机制

每条论断带依据。审查时优先核对这里。

1. `get_unchecked` 越界即使结果不用也是 UB。
   **依据**：`core/src/slice/mod.rs:612-619`（`source-refs.md`）。
2. 先 `get` 再 `get_unchecked`，读到的值与直接索引相同。
   **依据**：`tests/c15_unsafe.rs::get_unchecked_in_bounds_matches_index`。
3. 越界 `get_unchecked` 被 Miri 判定为 UB（W1 + W2 别名 `assume`）。
   **依据**：`tests/c15_unsafe_ub.rs::get_unchecked_oob_is_ub_under_miri`。
4. 两次 `addr_of!` 指向同一位置；写再读圆回写入的值。
   **依据**：`tests/c16_raw_ptr.rs::two_addr_of_are_the_same_place` /
   `write_then_read_roundtrips`。
5. 释放后再 `ptr::read`，Miri 报 `has been freed`。
   **依据**：`tests/c16_raw_ptr_ub.rs::use_after_free_is_ub_under_miri`。
6. `wrapping_add` 本身永远安全；`add` 越出分配立刻犯规。
   **依据**：`const_ptr.rs:985-988/838`；
   `tests/c17_ptr_arith.rs::wrapping_add_agrees_inside_allocation`；
   `tests/c17_ptr_arith_ub.rs::add_past_allocation_is_ub_under_miri`。
7. 分配内走一步的字节差等于 `size_of::<u32>()`。
   **依据**：`tests/c17_ptr_arith.rs::one_step_is_one_element_wide`。
8. 独立 `u64` 满足 `(p as usize) % align_of::<u64>() == 0`。
   **依据**：`tests/c18_alignment.rs::local_u64_is_aligned`；`mem/mod.rs:540`。
9. 8 字节缓冲从偏移 1 写 `u64`，本机打印 2，Miri 报 `memory access failed`。
   **依据**：`tests/c18_alignment_ub.rs::misaligned_u64_is_ub_under_miri`；
   OBSERVATIONS「C-18 UB：打印 2」。
10. `UnsafeCell::get` 是安全函数，只交出裸指针。
    **依据**：`cell.rs:2443`。
11. 顺序写两次读回 2；叠两份 `&mut` 在 SB 下报 borrow stack，在 TB 下报 foreign write。
    本实验两套都报 UB，属 plan.md 情形 1，不是模型敏感。
    爆发点不同：SB 在写第一份（tag 已没），TB 在写第二份（tag 已 Disabled）。
    **依据**：`tests/c19_aliasing.rs::unsafecell_sequential_writes`；
    `tests/c19_aliasing_ub.rs` 两条（绑定各自模型）；
    OBSERVATIONS「C-19 双模型」。
12. `from_raw_parts` 包装拒绝奇数长度；`set_len` 撒谎后读 `v[0]` 是未初始化。
    **依据**：`tests/c20_mem_safety.rs::odd_len_rejected`；
    `tests/c20_mem_safety_ub.rs::lying_set_len_is_ub_under_miri`；
    `vec/mod.rs:2224`。
13. 边界检查挡住 `off=3`；拿掉检查就是 UB 路径。
    **依据**：`tests/c20_mem_safety.rs::checked_parse_out_of_bounds_is_none`；
    OBSERVATIONS「C-20 bounds_check」。

本节共 **13** 条论断，每条带一处依据。核对：

```bash
grep -c '\*\*依据\*\*' feynman/m5-unsafe.md   # 期望 13
```

---

## 4. 常见误区

### C-15

- **误解**：`unsafe` 就是关闭检查。
  → **实际**：检查改由你保证。五要素写不出来就不要写这块。
  → **证据**：`get_unchecked_in_bounds_matches_index` 对照
  `get_unchecked_oob_is_ub_under_miri`。
- **误解**：没崩溃就是没有 UB。
  → **实际**：判定看 Miri 类别。debug abort 也只是可选检查。
  → **证据**：OBSERVATIONS「C-15 UB：debug 会 abort」。

### C-16

- **误解**：指针还在，所以内存还在。
  → **实际**：指针值不等于分配还活着。
  → **证据**：`use_after_free_is_ub_under_miri`。

### C-17

- **误解**：不解引用，指针运算怎么挪都没事。
  → **实际**：`add` 越界立刻犯规。`wrapping_add` 才推迟到解引用。
  → **证据**：`add_past_allocation_is_ub_under_miri`。

### C-18

- **误解**：x86_64 打印出 2，所以未对齐写合法。
  → **实际**：硬件容忍 ≠ 语言合法。同一源码 Miri 报 UB。
  → **证据**：`misaligned_u64_is_ub_under_miri`；OBSERVATIONS「C-18 UB：打印 2」。

### C-19

- **误解**：`UnsafeCell::get` 是 unsafe，用它就自动合法。
  → **实际**：`get` 是安全的。犯规在叠两份 `&mut`。
  → **证据**：`cell.rs:2443`；`overlapping_mut_is_ub_under_stacked_borrows`。

### C-20

- **误解**：函数没标 `unsafe`，怎么调用都安全。
  → **实际**：要看有没有合法调用序列会穿帮。`lying_bytes` 就是撒谎。
  → **证据**：`lying_set_len_is_ub_under_miri`。
- **误解**：去掉边界检查本机没崩，所以检查多余。
  → **实际**：本机不一定崩正是 UB 的表象。检查对应后续加载时的静态证明义务。
  → **证据**：OBSERVATIONS「C-20 bounds_check」；`concept.md` C-20 关联小节。

---

## 5. 验证性问题

1. 你去掉字节解析里的边界检查，用 `off=3` 去读两个字节，Miri 会报告哪一类 UB？
   **回答**：越界读。本模块对照侧用 W1 + W2（`get_unchecked` 越界是 `assume` 别名）
   或 W1 + W10（未初始化）演示同类"走出了该在的范围"。检查挡住的那一行是
   `parse_u16_be_checked(&pkt, 3) == None`。
   **指向**：(a) `tests/c20_mem_safety.rs::checked_parse_out_of_bounds_is_none`；
   (a) `tests/c15_unsafe_ub.rs::get_unchecked_oob_is_ub_under_miri`

2. 普通运行打印了一个"合理"的 `2`，你据此写"所以没有未定义行为"。缺了哪一步？
   **回答**：缺 Miri。x86_64 把未对齐/短缓冲写藏成打印 2。FR-019：未跑工具只能记 `n/a`。
   **指向**：(a) `tests/c18_alignment_ub.rs::misaligned_u64_is_ub_under_miri`；
   (c) OBSERVATIONS「C-18 UB：打印 2」

3. Safety 注释里有一项你写了"不适用"。若其实适用，漏掉它会让哪一类问题滑走？
   **回答**：例如对 `u64` 写漏掉对齐，会把 C-18 的核心对照当成"长度够就行"。
   本实验 Miri 先报越界长度，未对齐是同一操作的另一面。不适用必须写原因。
   **指向**：(c) OBSERVATIONS「C-18 UB：打印 2」；
   (b) `acceptance/safety-invariant-audit.md`

4. 两套别名模型一个在写 `*a` 时报，一个在写 `*b` 时报。你若把"两轮结果相同"写成稳定断言，错在哪？
   **回答**：那会把一个已知的语言未决问题固化成回归测试。本实验两套都报 UB（情形 1），
   但仍分两条断言绑定模型。爆发点不同正是要解释的机制，不是要抹平的噪声。
   **指向**：(a) `tests/c19_aliasing_ub.rs::overlapping_mut_is_ub_under_stacked_borrows` 与
   `overlapping_mut_is_ub_under_tree_borrows`；
   (c) OBSERVATIONS「C-19 双模型」

5. 一个函数对外是安全的，对内用了 `unsafe`。怎样判断它是真封装？
   **回答**：找有没有"调用方没做错仍穿帮"的序列。`lying_bytes` + `v[0]` 就是。
   真封装像 `parse_u16_be_checked`：越界只能得到 `None`。
   **指向**：(a) `tests/c20_mem_safety_ub.rs::lying_set_len_is_ub_under_miri`；
   (a) `tests/c20_mem_safety.rs::checked_parse_out_of_bounds_is_none`

6. 同一段解析，运行时检查、类型系统、加载时静态证明，各把义务放在哪个阶段？
   **回答**：运行时 = `get` / `parse_u16_be_checked`；编译期 = 不让两份 `&mut` 叠上；
   加载时 = eBPF verifier 的静态边界证明。本实验对应拿掉检查的那一行，不写 eBPF 程序。
   **指向**：(c) OBSERVATIONS「C-20 bounds_check」；
   (b) `learning/m5-unsafe/concept.md` C-20 关联小节

---

## 检验结果

| # | 检验项 | 合格标准（可判定） | 状态 |
|---|-------|-----------------|------|
| 1 | 自述概念 | 第 1 节存在；Rust 术语（`unsafe`/未定义行为/对齐/别名/封装）均在首次出现时用 C 世界的话解释 | **pass** |
| 2 | 最小示例 | C-15…C-20 各一段，均 ≤15 行，并链接到 `examples/` | **pass** |
| 3 | 底层机制 | 第 3 节 13 条论断，每条带依据标记；`grep -c '\*\*依据\*\*'` = 13 | **pass** |
| 4 | 常见误区 | 每个 covered capability ≥1 条，三段式齐备 | **pass** |
| 5 | 回答问题 | 6 个问题（≥5），含"去掉边界检查 Miri 报哪类 UB"；每题指向断言 / 源码 / 观测块 | **pass** |

**五项是合取。** 本表全部 pass → m5 的 Capability 可以进入 `accepted`。
