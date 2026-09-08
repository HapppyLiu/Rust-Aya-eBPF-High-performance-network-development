# Module 4: 并发、Send/Sync 与原子操作

**Story**: US4（P2） | **Capabilities**: C-12…C-14 | **Prerequisite**: m3（accepted）+ T024 题集已冻结

> 本文件属于 **Answer Track**。对应的 Learner Track 是
> [`learner/m4-concurrency/guide.md`](../../learner/m4-concurrency/guide.md)。
> 如果你还没做过那边的预测表，先去做 —— 这份文件读过之后就不能再自测了。

## 这个模块回答什么问题

1. 把值**移动**到另一个线程，和让两个线程**同时持有引用**，分别要求哪条标记？
2. 含内部可变性是否等于不能跨线程？容器会提升还是原样传递内部类型的能力？
3. 安全 Rust 里写数据竞争，编译器拒绝的是哪一条规则？
4. 放宽内存序之后哪些执行顺序变为可能？本机一次打印看见"正确"结果证明了什么？

---

## 概念

### C-12 Send / Sync

- **一句话定义**：`Send` 约束的是值能否被**移动**到另一个线程；`Sync` 约束的是
  `&T` 是否 `Send`，即多个线程能否**同时持有引用**。两者独立，不是同一件事。

- **底层机制**：

  `Send` 与 `Sync` 是 **unsafe auto trait**（`core/src/marker.rs:92` / `:657`）。
  方法体是空的：约束不写在方法里，而写在**字段自动推导**和**显式 impl** 上。
  所有字段都 `Send` 则该类型 `Send`；所有字段都 `Sync` 则该类型 `Sync`。
  除非有显式正向 / 负向 impl 覆盖。

  两者约束的不是同一件事（US4 AS1）：

  | | 约束的是 | 直觉 |
  |---|---------|------|
  | `Send` | 值可以换到另一个线程持有 | 所有权换线程，安全吗 |
  | `Sync` | `&T` 是 `Send` | 两个线程同时读它，安全吗 |

  由此有一条常被忽略的等价关系：**`T: Sync` ⟺ `&T: Send`**
  （`marker.rs:105` 的 `unsafe impl<T: Sync> Send for &T`）。

  内部可变性是本模块的试金石，不是"一律不能跨线程"：

  - `Cell<T>`：`T: Send` 则 `Send`（`cell.rs:317`），显式 `!Sync`（`:325`）。
    移动整个 `Cell` 之后原线程碰不到它，没有两线程同时改的问题。
    `move_cell_to_thread` 就是这一侧。
  - 共享 `&Cell`：`Cell: !Sync` → `&Cell: !Send` → scoped 闭包不是 `Send` → **E0277**
    （`c12_cell_across_threads.rs`）。
  - `Mutex<T>` 的 `Sync` 条件是 **`T: Send`**，不是 `T: Sync`
    （`std/src/sync/poison/mutex.rs:257`）。`Cell<u32>` 是 `Send`，
    所以 `Mutex<Cell<u32>>` 两者皆是 —— 锁保证同一时刻最多一个线程拿到内部数据，
    于是把 `Send` **提升**成了 `Sync`。
  - `UnsafeCell<T>` 是整条内部可变性链的根：显式 `!Sync`（`cell.rs:2328`）。
    需要并发安全的类型（`Mutex`、`AtomicUsize`）再自己 `unsafe impl Sync`。

  裸指针两者皆非（`marker.rs:97-99` / `:672-674`）：编译器对指向的东西一无所知，
  保守拒绝，把判断权交还给写 `unsafe impl` 的人。

- **常见误解**：
  - **误解**："含内部可变性 = 不能跨线程" → **实际**：不能的是**共享**，不是**移动**。
    `Cell<u32>` 是 `Send`。
    → **证据**：`cell_is_send_not_asserted_sync` + `moving_cell_preserves_value`。
  - **误解**："容器原样传递内部类型的能力" → **实际**：`Mutex` 把 `T: Send` 提升为
    `Mutex<T>: Sync`；`Arc` 两边都要求 `T: Send + Sync`，不提升。
    → **证据**：`mutex_of_cell_is_send_and_sync` 对照题集第 6 题 `SharedMut`。

- **对应实验**：[`c12_send_sync`](../../experiments/m4-concurrency/examples/c12_send_sync.rs)
  / [tests](../../experiments/m4-concurrency/tests/c12_send_sync.rs)
  / [题集裁判](../../experiments/m4-concurrency/tests/c12_send_sync_quiz.rs)

---

### C-13 Concurrency

- **一句话定义**：`thread::scope` 允许借用非 `'static` 的环境，但闭包仍须 `Send`；
  `Mutex` 保证互斥，不保证完成顺序。安全 Rust 里的数据竞争被借用规则拦下。

- **底层机制**：

  `thread::spawn`（`std/src/thread/functions.rs:125`）要求
  `F: Send + 'static`。寿命和标记是两问：C-10 覆盖了 `'static`（E0373），
  C-11 / C-12 覆盖了 `Send`（E0277）。

  `thread::scope`（`std/src/thread/scoped.rs:141`）的 `Scope::spawn`
  （`:201`）要求 `F: Send + 'scope` —— 寿命收到 scope 结束，标记要求还在。
  所以 C-12 的 `Cell` 样本用 `scope` 而不是 `spawn`：排除寿命干扰，只剩 `Sync`。

  `Mutex<T>`（`std/src/sync/poison/mutex.rs:227`）内部是系统锁 + `UnsafeCell<T>`。
  `lock` 拿到的 `MutexGuard` 显式 `!Send`（`:289`，POSIX 要求同一线程解锁），
  却可以是 `Sync`（`:294`，`&MutexGuard` 只能读 `T`，不能触发 `Drop`）。
  这打破"能共享就一定能移动"的错觉 —— 题集第 7 题专门打它。

  不变量：`scoped_mutex_sum(4, 25) == 100`。四个线程各加 25，总和与谁先结束无关。
  example 打印的编号排列是 NON-ASSERTION（§C2.2 禁止断言线程交错）。

  数据竞争（US4 AS3）：两个 scoped 线程对同一局部 `u64` 做 `+=`，
  每边都要 `&mut hits`。借用规则同一时刻至多一个可变借用 → **E0499**
  （`c13_data_race.rs`）。拒绝的不是"忘了加锁"这条 API 建议，
  而是别名规则。绕过它需要 `unsafe`，并承担"不制造数据竞争"的义务。
  本模块不绕。

- **常见误解**：
  - **误解**："加了锁，完成顺序也就定了" → **实际**：锁保证互斥，不保证调度顺序。
    → **证据**：`scoped_mutex_sum_is_order_independent`；
    OBSERVATIONS「C-13：排列是 NON-ASSERTION」。
  - **误解**："安全 Rust 没有数据竞争是因为线程 API 会检查" → **实际**：
    拦截点是借用规则（E0499），与 C-12 的 E0277 不是同一条。
    → **证据**：`data_race_in_safe_rust_is_e0499`。

- **对应实验**：[`c13_concurrency`](../../experiments/m4-concurrency/examples/c13_concurrency.rs)
  / [tests](../../experiments/m4-concurrency/tests/c13_concurrency.rs)

---

### C-14 Atomic

- **一句话定义**：原子类型保证单次读写不撕裂，并且显式 `Send + Sync`；
  内存序是另一问 —— 周围那些**别的**读写何时对其他线程可见。

- **底层机制**：

  `Atomic<T>`（`core/src/sync/atomic.rs:361`）内部是 `UnsafeCell`，
  但有显式 `Send` / `Sync`（`:366-368`），因为所有操作走原子指令。
  `AtomicUsize` 由宏在 `:3825` 展开。与 `Cell` 对照：同样含内部可变性，
  差别不在"有没有"，而在**修改是否原子**。

  `Ordering`（`:442`）从最弱到最强包括 `Relaxed` / `Release` / `Acquire` / `SeqCst`。
  `Relaxed` 只保证这一次操作原子，不约束周围读写的可见性。

  可断言的事实：

  | 事实 | 断言 |
  |------|------|
  | 4 线程 × 25 次 `SeqCst` `fetch_add` | 终值 100 |
  | Release 存旗标 + Acquire 读旗标 | 看见旗标 ⇒ 载荷是 42 |
  | `fetch_add` | 返回加之前的值 |
  | `Ordering` 变体 | `Relaxed != SeqCst` |
  | 布局 | `size_of::<AtomicUsize>() == size_of::<usize>()` |

  放宽之后哪些顺序变为可能（US4 AS2）：

  写侧 `data = 42; flag = 1`，读侧 `if flag == 1 { use data }`。
  释放 / 获取配对下，看见旗标就蕴含看见载荷。
  两边都改成 `Relaxed` 之后，语言规则允许**旗标已立、载荷仍是 0**。
  这在弱序架构（aarch64 / POWER）上是真实窗口。

  x86_64 的 TSO 对商店几乎按程序序提交，常常把这个窗口**掩盖**成"总是 42"。
  example 里 `relaxed seen = 42` 是 NON-ASSERTION。
  **一次打印看见 42 不能证明弱序程序正确，更不能跨架构推广**（FR-018）。

- **常见误解**：
  - **误解**："原子变量能共享，是因为它没有内部可变性" → **实际**：
    它有 `UnsafeCell`，只是操作是原子的，所以有显式 `Sync`。
    → **证据**：`atomic.rs:361/366-368`；`atomic_is_send_and_sync` 对照
    `cell_is_send_not_asserted_sync`。
  - **误解**："本机 Relaxed 握手看见 42，所以弱序是安全的" → **实际**：
    x86_64 TSO 掩盖了窗口。断言只钉 Release/Acquire 与 SeqCst 不变量。
    → **证据**：OBSERVATIONS「C-14：Relaxed 看见 42 不能推广」。

- **对应实验**：[`c14_atomic`](../../experiments/m4-concurrency/examples/c14_atomic.rs)
  / [tests](../../experiments/m4-concurrency/tests/c14_atomic.rs)

---

## 与后续学习的关联

FR-014 要求每项能力说明与后续 Linux / eBPF / Aya 的关联点，或显式标注为仅是基础。

| C-ID | 关联点 |
|------|-------|
| **C-12 Send / Sync** | 直接对应。Aya **用户态**把 BPF map 句柄、`Ebpf` 对象送到工作线程时，
  编译器问的就是这两条标记。map 的并发访问首先是"这个句柄是 `Send` 还是 `Sync`"，
  不是"大家是不是都加了锁"。eBPF **程序侧**没有这两条 auto trait；
  内核里的共享是 map / percpu。本 Feature 不写 eBPF（FR-017），
  只把用户态共享合同钉住。 |
| **C-13 Concurrency** | 直接对应。用户态加载器、事件循环、多线程读 map，日常就是 `scope` / `Mutex`。
  锁保证互斥，不保证完成顺序 —— 后面谈 lock-free 时，这个区分是入口。
  数据竞争被借用规则拦截，是读用户态并发代码时的第一道检查。 |
| **C-14 Atomic** | 直接对应。后续 lock-free 与无锁 ring / AF_XDP 完成标志，全部是原子 + 内存序。
  本模块只建立"原子 ≠ 顺序"和"x86_64 会掩盖弱序"这两条。
  深化（具体 ring 协议、eBPF 侧原子）留给后续 Feature。 |

三项都不是"仅为理解基础"。读不懂 Send/Sync、互斥与顺序的差别、内存序窗口，
就读不了用户态对 BPF map 的并发访问，也进不了 lock-free。
