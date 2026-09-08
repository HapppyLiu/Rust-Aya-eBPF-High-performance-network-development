# Feynman: Module 4 — 并发、Send/Sync 与原子操作

**Capabilities covered**: C-12, C-13, C-14

---

## 1. 用自己的话解释

面向"没学过 Rust 但懂 C"的同事。

C 程序员谈线程，通常混成一句："这个指针能不能传到另一个 pthread"。
Rust 把这句话拆成两问，并且多问了第三问：周围那些别的读写，别人什么时候看得见。

**第一，换主人 和 同时看，不是同一件事。**
把一个值的所有权交给另一个线程，问的是：原来的线程再也碰不到它了，这样安全吗？
让两个线程同时拿着指向它的引用，问的是：他们会不会同时改坏同一个格子？
这两条标记是自动推的：看每个字段，最弱的那个说了算。方法体是空的，
没有"实现一个 send 函数"这种事。
含内部可变性也不等于不能跨线程。一个只能在单线程里改的格子，
整颗移走是安全的 —— 移走之后就剩一只手。两只手同时改它才不行。
互斥锁做的事更奇怪：它不要求里面的东西自己能被同时看，
只要求里面的东西能被交给拿到锁的那只手。于是一把锁可以把"能移动"
升级成"能共享"。共享指针做不到这种升级，它两边都要里面的东西自己既可移动又可共享。

**第二，锁保证的是互斥，不是谁先跑完。**
两个线程经同一把锁各加一串数字，总和必须对。谁先打印"我结束了"，
操作系统说了算，一次运行看见的顺序不能当成规则。
若你在安全语言里让两个线程同时改同一个普通整数，编译器根本不让你写出来：
那是两份可变借用，不是"线程函数忘了检查"。
想绕过这条，就得自己保证不制造两线程同时写 —— 那是后面 unsafe 模块的义务。
本模块不绕。

**第三，原子操作先保证这一次读写不会撕成两半。**
内存序是另一张账单：你先写载荷再立旗标，对面看见旗标的时候，
能不能看见载荷？最强的那一档、以及"先发布再获取"那一对，答案是能。
两边都放到最弱，语言规则允许旗标已经立了、载荷还是旧的。
本机这种强顺序的处理器常常把窗口藏起来，打印里看起来总是对的。
那只能说明这一次没看见窗口，不能说明弱序架构上也没有。

---

## 2. 最小示例

### C-12 Send / Sync — 能移动 ≠ 能共享

```rust
fn assert_send<T: Send>() {}
assert_send::<std::cell::Cell<u32>>();          // 移动可以
// 两个 scope 线程碰同一个 Cell → 编译拒绝（共享不行）
```

完整观察：[`examples/c12_send_sync.rs`](../experiments/m4-concurrency/examples/c12_send_sync.rs)（20 行）

### C-13 Concurrency — 总和确定，顺序不确定

```rust
let acc = Mutex::new(0u64);
thread::scope(|s| {
    for _ in 0..4 { s.spawn(|| { *acc.lock().unwrap() += 25; }); }
});
assert_eq!(acc.into_inner().unwrap(), 100);
```

完整观察：[`examples/c13_concurrency.rs`](../experiments/m4-concurrency/examples/c13_concurrency.rs)（20 行）

### C-14 Atomic — 看见旗标就看见载荷

```rust
data.store(42, Relaxed);
flag.store(1, Release);
while flag.load(Acquire) == 0 { spin(); }
assert_eq!(data.load(Relaxed), 42);
```

完整观察：[`examples/c14_atomic.rs`](../experiments/m4-concurrency/examples/c14_atomic.rs)（22 行）

---

## 3. 底层机制

每条论断带依据。审查时优先核对这里。

1. `Send` / `Sync` 是空方法体的 unsafe auto trait。
   **依据**：`core/src/marker.rs:92/657`（`source-refs.md`）。
2. `&T: Send` 要求 `T: Sync`，不是 `T: Send`。
   **依据**：`core/src/marker.rs:105`。
3. `Cell<T>` 在 `T: Send` 时是 `Send`，显式 `!Sync`。
   **依据**：`core/src/cell.rs:317/325`；
   `tests/c12_send_sync.rs::cell_is_send_not_asserted_sync`。
4. 共享 `&Cell` 到 scoped 线程报 E0277。
   **依据**：`tests/c12_send_sync.rs::cell_across_threads_is_e0277`。
5. 把 `Cell` 移动到另一线程，值保留。
   **依据**：`tests/c12_send_sync.rs::moving_cell_preserves_value`。
6. `Mutex<T>: Sync` 的条件是 `T: Send`。
   **依据**：`std/src/sync/poison/mutex.rs:257`；
   `tests/c12_send_sync.rs::mutex_of_cell_is_send_and_sync`。
7. `MutexGuard` 显式 `!Send`，`T: Sync` 时是 `Sync`。
   **依据**：`std/src/sync/poison/mutex.rs:289/294`；
   `tests/c12_send_sync_quiz.rs::quiz_held_is_sync`。
8. `spawn` 要求 `Send + 'static`；`scope` 的 spawn 要求 `Send + 'scope`。
   **依据**：`std/src/thread/functions.rs:128`；
   `std/src/thread/scoped.rs:203`。
9. 4 线程经 Mutex 各加 25，总和是 100，与完成顺序无关。
   **依据**：`tests/c13_concurrency.rs::scoped_mutex_sum_is_order_independent`。
10. 安全 Rust 里两个线程 `+=` 同一局部变量，拒绝码是 E0499。
    **依据**：`tests/c13_concurrency.rs::data_race_in_safe_rust_is_e0499`。
11. `Atomic<T>` 内含 `UnsafeCell`，但有显式 `Send`/`Sync`。
    **依据**：`core/src/sync/atomic.rs:361/366/368`；
    `tests/c14_atomic.rs::atomic_is_send_and_sync`。
12. `SeqCst` 自增 4×25 终值是 100。
    **依据**：`tests/c14_atomic.rs::seqcst_increments_sum_to_product`。
13. Release/Acquire 握手看见旗标后载荷是 42。
    **依据**：`tests/c14_atomic.rs::release_acquire_handshake_sees_payload`。
14. 两边 Relaxed 时语言允许旧载荷；本机看见 42 不能跨架构推广。
    **依据**：OBSERVATIONS「C-14：Relaxed 看见 42 不能推广」。

本节共 **14** 条论断，每条带一处依据。核对：

```bash
grep -c '\*\*依据\*\*' feynman/m4-concurrency.md   # 期望 14
```

---

## 4. 常见误区

### C-12

- **误解**：含内部可变性就不能跨线程。
  → **实际**：不能的是共享。`Cell<u32>` 可以移动。
  → **证据**：`moving_cell_preserves_value` 对照 `cell_across_threads_is_e0277`。
- **误解**：容器原样传递内部类型的能力。
  → **实际**：`Mutex` 把 `T: Send` 提升为 `Sync`；`Arc` 不提升。
  → **证据**：`mutex_of_cell_is_send_and_sync`；题集第 5 / 第 6 题。

### C-13

- **误解**：加了锁，完成顺序也就定了。
  → **实际**：锁保证互斥。一次打印的排列是 NON-ASSERTION。
  → **证据**：`scoped_mutex_sum_is_order_independent`；
  OBSERVATIONS「C-13：排列是 NON-ASSERTION」。
- **误解**：安全语言没有数据竞争是因为线程 API 做了检查。
  → **实际**：拦截点是借用规则 E0499。
  → **证据**：`data_race_in_safe_rust_is_e0499`。

### C-14

- **误解**：原子类型能共享是因为它没有内部可变性。
  → **实际**：它有 `UnsafeCell`，只是操作是原子的。
  → **证据**：`atomic.rs:361/366-368`；`atomic_is_send_and_sync`。
- **误解**：本机 Relaxed 握手看见 42，所以弱序安全。
  → **实际**：x86_64 TSO 掩盖窗口。不能推广到 aarch64。
  → **证据**：OBSERVATIONS「C-14：Relaxed 看见 42 不能推广」。

---

## 5. 验证性问题

1. `Cell<u32>` 可以移动到另一个线程。若据此说"含内部可变性的类型都能跨线程共享"，错在哪一步？
   **回答**：把"移动"当成了"共享"。`Send` 成立只说明换主人安全；
   `Sync` 才允许两线程同时持有引用。共享 `&Cell` 是 E0277。
   **指向**：(a) `tests/c12_send_sync.rs::cell_across_threads_is_e0277`

2. `Mutex<Cell<u32>>` 可以共享。这是不是意味着 `Cell` 自己变成了可共享？
   **回答**：不是。`Cell` 仍是 `!Sync`。提升发生在 `Mutex` 的 `Sync` impl：
   条件是 `T: Send`。锁保证同一时刻最多一只手碰到内部的 `Cell`。
   **指向**：(b) `source-refs.md` `poison/mutex.rs:257`；
   (a) `tests/c12_send_sync.rs::mutex_of_cell_is_send_and_sync`

3. 你跑了一次 example，四个线程按 0、1、2、3 的顺序打印结束。
   若据此断言"互斥锁保证了这个完成顺序"，错在哪？
   **回答**：把一次调度观察当成了不变量。锁保证的是互斥。
   本模块可断言的是总和 100 和编号集合，不是排列。
   **指向**：(a) `tests/c13_concurrency.rs::scoped_mutex_sum_is_order_independent`；
   (c) OBSERVATIONS「C-13：排列是 NON-ASSERTION」

4. 两边都用最弱内存序的握手，在本机打印里仍然看到了正确载荷。
   这能证明这段程序在弱序架构上仍然正确吗？
   **回答**：不能。x86_64 TSO 常常把"旗标已立、载荷仍是 0"这个窗口藏起来。
   一次打印是 NON-ASSERTION，不能当跨架构证据。
   **指向**：(c) OBSERVATIONS「C-14：Relaxed 看见 42 不能推广」

5. 题集里有一道守卫类型：一条标记成立、另一条不成立。
   若你一直以为"能共享就一定能移动"，这道题会怎样打你的脸？
   **回答**：`MutexGuard` 是 `Sync` 但 `!Send`。能同时拿着引用读，
   不能把守卫移到另一线程再析构（解锁必须在加锁的线程）。
   **指向**：(a) `tests/c12_send_sync_quiz.rs::quiz_held_is_sync`；
   (b) `poison/mutex.rs:289/294`

6. 后续读用户态里对共享映射的并发访问时，你先问"要移动还是要共享"，
   还是先问"大家是不是都加了锁"？
   **回答**：先问移动还是共享。那是 C-12 的两问。锁是 C-13 给共享这一侧用的工具，
   不是把两问混成一句的借口。
   **指向**：(a) `tests/c12_send_sync.rs::cell_is_send_not_asserted_sync` 对照
   `mutex_of_cell_is_send_and_sync`

---

## Quiz 逐题推导依据（SC-007）

完整推导在 [`acceptance/send-sync-quiz.result.md`](../acceptance/send-sync-quiz.result.md)。
这里只钉"依据指向哪条编译器判定"。

| # | 类型 | Send | Sync | 编译器裁判 |
|---|-----|------|------|-----------|
| 1 | `Tick` | 是 | 是 | `quiz_tick_is_send_sync` |
| 2 | `Slot` | 是 | 否 | `quiz_slot_is_send` + `quiz_slot_not_sync.rs` |
| 3 | `Shared` | 否 | 否 | `quiz_shared_not_send.rs` |
| 4 | `RawSlot` | 否 | 否 | `quiz_rawslot_not_send.rs` |
| 5 | `Guarded` | 是 | 是 | `quiz_guarded_is_send_sync` |
| 6 | `SharedMut` | 否 | 否 | `quiz_sharedmut_not_send.rs` |
| 7 | `Held` | 否 | 是 | `quiz_held_is_sync` + `quiz_held_not_send.rs` |
| 8 | `Marked` | 否 | 否 | `quiz_marked_not_send.rs` |
| 9 | `Ticker` | 是 | 是 | `quiz_ticker_is_send_sync` |
| 10 | `Callback` | 是 | 否 | `quiz_callback_is_send` + `quiz_callback_not_sync.rs` |
| 11 | `RawCell` | 是 | 否 | `quiz_rawcell_is_send` + `quiz_rawcell_not_sync.rs` |
| 12 | `Peek` | 否 | 否 | `quiz_peek_not_send.rs` |

---

## 检验结果

| # | 检验项 | 合格标准（可判定） | 状态 |
|---|-------|-----------------|------|
| 1 | 自述概念 | 第 1 节存在；Rust 术语（`Send`/`Sync`/互斥/内存序）均在首次出现时用 C 世界的话解释 | **pass** |
| 2 | 最小示例 | C-12 / C-13 / C-14 各一段，均 ≤15 行，并链接到 `examples/` | **pass** |
| 3 | 底层机制 | 第 3 节 14 条论断，每条带依据标记；`grep -c '\*\*依据\*\*'` = 14 | **pass** |
| 4 | 常见误区 | 每个 covered capability ≥1 条，三段式齐备（C-12×2、C-13×2、C-14×2） | **pass** |
| 5 | 回答问题 | 6 个问题（≥5），每题回答指向断言名 / 源码引用 / 观测块标题之一 | **pass** |

**五项是合取。** 本表全部 pass → m4 的 Capability 可以进入 `accepted`。
