# Send / Sync 判定题集

**Feature**: 001-rust-foundation | **能力**: C-12 | **Story**: US4 |
**依据**: SC-007 / research.md R-10 / [learning-artifact-contract §F](../specs/001-rust-foundation/contracts/learning-artifact-contract.md)

**冻结日期**: 2026-09-04（以版本控制的提交时间为准）

---

## ⚠️ 使用规则

1. **本题集在 US4 学习开始前定稿**（规则 F1）。题目 MUST NOT 在验收时另行挑选或增删。
2. 学完 m4 后**一次性**作答，把作答与推导写入 `send-sync-quiz.result.md`。
3. **作答完成并提交之前，MUST NOT 打开 [`send-sync-quiz.answers.md`](./send-sync-quiz.answers.md)。**
4. "一次性"的定义：`send-sync-quiz.result.md` 的**首次提交**即为作答结果，
   之后再改答案不改变判定（规则 F6）。

## 通过线（规则 F5，两条合取）

1. **判定正确性**：每题的 `Send` 与 `Sync` 两项**都**答对才算该题正确；全卷错题数 **≤ 1**。
2. **推导依据**：每题 MUST 写出推导依据。**无依据的题即使判定正确也计为错题** ——
   SC-007 要的是"能给出推导依据而非结论"。

> 口径说明：SC-007 写的是"正确率不低于 90%（即最多错 1 道）"。本题集 12 题，
> 以**绝对错题数 ≤ 1** 为准（规则 F5 声明冲突时以此为准，因为它更严格，
> 且 SC-007 自己用括号把 90% 注释成了"最多错 1 道"）。

## 客观校验

作答之后，由编译器做最终裁判（规则 F4）：

```bash
cargo test -p m4-concurrency --test c12_send_sync_quiz
```

- 正向：`fn assert_send<T: Send>()` / `fn assert_sync<T: Sync>()`
- 负向：`compile_fail/quiz_*.rs` 断言 `E0277`

编译器说了算 —— 你的推导写得再漂亮，与编译器判定冲突时以编译器为准，
然后回头找出推导错在哪一步。

---

## 作答表（复制到 `send-sync-quiz.result.md` 填写）

| # | 类型 | Send? | Sync? | 推导依据 |
|---|-----|-------|-------|---------|
| 1 | `Tick` | | | |
| 2 | `Slot` | | | |
| 3 | `Shared` | | | |
| 4 | `RawSlot` | | | |
| 5 | `Guarded` | | | |
| 6 | `SharedMut` | | | |
| 7 | `Held<'a>` | | | |
| 8 | `Marked` | | | |
| 9 | `Ticker` | | | |
| 10 | `Callback` | | | |
| 11 | `RawCell` | | | |
| 12 | `Peek<'a>` | | | |

---

## 题目

以下 12 个类型全部为**自定义类型**（规则 F2）。每题给出完整定义。
对每个类型判定它是否 `Send`、是否 `Sync`，并写出推导依据。

**推导依据要写成什么样**：指出该类型的哪个**字段**决定了结论、依据的是哪条 auto trait
推导规则或哪条显式 impl。只写"因为它不能跨线程"不算依据。

```rust
use core::cell::{Cell, RefCell, UnsafeCell};
use core::marker::PhantomData;
use core::sync::atomic::AtomicUsize;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};
```

### 1. `Tick`

```rust
pub struct Tick {
    count: u64,
}
```

### 2. `Slot`

```rust
pub struct Slot {
    value: Cell<u32>,
}
```

### 3. `Shared`

```rust
pub struct Shared {
    handle: Rc<u32>,
}
```

### 4. `RawSlot`

```rust
pub struct RawSlot {
    ptr: *mut u8,
    len: usize,
}
```

### 5. `Guarded`

```rust
pub struct Guarded {
    inner: Mutex<Cell<u32>>,
}
```

### 6. `SharedMut`

```rust
pub struct SharedMut {
    inner: Arc<RefCell<u32>>,
}
```

### 7. `Held<'a>`

```rust
pub struct Held<'a> {
    guard: MutexGuard<'a, u32>,
}
```

### 8. `Marked`

```rust
pub struct Marked {
    id: u32,
    _marker: PhantomData<*const u8>,
}
```

### 9. `Ticker`

```rust
pub struct Ticker {
    hits: AtomicUsize,
}
```

### 10. `Callback`

```rust
pub struct Callback {
    f: Box<dyn Fn() + Send>,
}
```

### 11. `RawCell`

```rust
pub struct RawCell {
    inner: UnsafeCell<u32>,
}
```

### 12. `Peek<'a>`

```rust
pub struct Peek<'a> {
    view: &'a Cell<u32>,
}
```

---

## 答题提示（不是答案）

- `Send` 与 `Sync` 是 **auto trait**：编译器按字段**自动推导**，除非有显式的 `impl` 或负向 impl。
  所以每一题的答案都取决于"哪个字段最弱"。
- 两者约束的**不是同一件事**。想清楚"把值**移动**到另一个线程"与
  "让两个线程**同时持有引用**"分别要求什么 —— 这两句话是 US4 AS1 的核心。
- 第 5、6 题放在一起看：容器对内部类型的要求不一定是"原样传递"。
  有的容器会**提升**能力，有的会**削弱**。想清楚提升的那个凭什么能提升。
- 第 8 题：零大小的字段也参与推导。
- 第 12 题：`&T` 自身的 `Send`/`Sync` 由 `T` 的哪一个属性决定？这两个方向是否相同？
