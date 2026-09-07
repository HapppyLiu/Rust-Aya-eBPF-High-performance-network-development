# Module 3: 组合能力（错误处理、迭代器、闭包、智能指针）

**Story**: US3（P2） | **Capabilities**: C-08…C-11 | **Prerequisite**: m2（accepted）

> 本文件属于 **Answer Track**。对应的 Learner Track 是
> [`learner/m3-composition/guide.md`](../../learner/m3-composition/guide.md)。
> 如果你还没做过那边的预测表，先去做 —— 这份文件读过之后就不能再自测了。

## 这个模块回答什么问题

1. `?` 失败时为什么能把一种错误变成另一种？缺了转换，拒绝发生在哪一步？
2. 迭代器适配器在被写出的那一行就开始跑了吗？`sum` 和 `collect` 的分配次数差在哪？
3. 三种闭包捕获分别满足哪一层调用 trait？捕获方式怎样决定能不能进线程？
4. `Box` / `Rc` / `Arc` 各自签的是哪一份所有权合同？`clone` 会不会新分配？

---

## 概念

### C-08 Error handling

- **一句话定义**：`Result<T, E>` 把成功值和失败值收进同一个枚举；`?` 在成功路径上
  留下 `T`，在失败路径上先做 `From` 转换再提前返回。

- **底层机制**：

  `Result` 是两变体枚举（`core/src/result.rs:557`）：`Ok(T)` / `Err(E)`。
  它比 C 的整数错误码多一条语言级约束——调用方必须处理这两种可能，
  少了一种 C 里常见的失误：看了返回值却把它当成功数据用。

  `?` 不是语法糖成 `return Err(e)` 那么简单。失败时走
  `FromResidual`（`result.rs:2185`），函数体里写着 `Err(From::from(e))`
  （`:2192`）。转换 trait 是 `From`（`core/src/convert/mod.rs:589`），
  要求**外层错误类型**实现 `From<内层错误类型>`。
  缺了这份 impl，编译器在单态化之前就报 **E0277**
  （`compile_fail/c08_missing_from.rs`），不是运行起来结果不对。

  手写 `match` + `AppErr::from(e)` 与 `?` 对同一组输入给出相同的 `Result`
  （`manual_and_question_mark_agree_on_every_path`）。
  变的是传播怎么写，不是这五个输入的行为。

  `std::error::Error` 在 `std/src/error.rs:4` 再导出 `core::error::Error`
  （`core/src/error.rs:59`）。除了 `Debug + Display`，它还提供 `source()`，
  用来指向更底层的原因。本实验的 `ParseErr`/`AppErr` 没有实现它——
  教学上要的是 `Result` + `From` + `?` 这条链，不是完整的错误生态。

- **常见误解**：
  - **误解**："`?` 就是提前 return" → **实际**：失败路径上还有一次 `From::from`。
    内外错误类型不同时，缺 `From` 会在编译期被拒。
    → **证据**：`missing_from_impl_is_e0277`；`result.rs:2192`。
  - **误解**："改成 `Result` 之后行为会变" → **实际**：本实验五条路径
    手写与 `?` 的 `Ok`/`Err` 完全一致。变的是类型强制你处理失败。
    → **证据**：`manual_and_question_mark_agree_on_every_path`。

- **对应实验**：[`c08_error`](../../experiments/m3-composition/examples/c08_error.rs)
  / [tests](../../experiments/m3-composition/tests/c08_error.rs)

---

### C-09 Iterator

- **一句话定义**：迭代器适配器是惰性的——写出链只构造结构体，
  拉动 `next` 时才按**元素**走完整条链；堆分配发生在终端消费者，不发生在 `map` 本身。

- **底层机制**：

  `Iterator`（`core/src/iter/traits/iterator.rs:42`）的推进方法是 `next`
  （`:78`），返回 `Option<Item>`。`map`（`:831`）只是 `Map::new(self, f)`。
  `Map`（`adapters/map.rs:61`）的 `next`（`:106`）是：

  ```text
  self.iter.next().map(&mut self.f)
  ```

  一次拉动取 **一个**上游元素，立刻调用闭包。所以 example 里副作用的顺序是
  `map 0 / filter 0 / map 1 / filter 2 / …`，而不是"先全部 map 再全部 filter"。
  链构造完毕那一行看不到任何 `map` 打印。

  分配次数是确定性量，由 `CountingAllocator` 计数（US3 AS1 / R-07）：

  | 闭包 | allocs（n=8） | 原因 |
  |------|---------------|------|
  | `map` + `sum` | **0** | 整数累加，不构造堆上容器 |
  | `map` + `collect::<Vec<_>>` | **1** | `Range` 的 `size_hint` 精确，`Map` 原样转发（`map.rs:111`），一次按长度分配 |
  | 只构造 `map`+`filter`、不消费 | **0** | 适配器在栈上 |
  | `Vec::new` + `push` 同样 8 个 `i32` | **多于 1 次分配事件** | 起点容量 0，要增长 |

  手写循环与迭代器的**元素结果**一致（`loop_and_iterator_agree_on_values`）。
  元素一致不等于分配次数一致。

- **常见误解**：
  - **误解**："写出 `map` 那一行就已经跑完了" → **实际**：适配器是惰性的，
    `must_use` 警告的就是"构造了但不消费"。
    → **证据**：OBSERVATIONS「C-09：副作用出现在消费时」；`map.rs:106`。
  - **误解**："迭代器一定比手写循环更会分配 / 一定少分配" → **实际**：
    看终端消费者和 `size_hint`。`sum` 零次；精确 hint 的 `collect` 一次到位；
    空 `Vec`+`push` 要增长。
    → **证据**：`iterator_chain_allocation_counts`。

- **对应实验**：[`c09_iterator`](../../experiments/m3-composition/examples/c09_iterator.rs)
  / [tests](../../experiments/m3-composition/tests/c09_iterator.rs)

---

### C-10 Closure

- **一句话定义**：闭包对捕获物做的所有权动作（只读借 / 可变借 / 拿走）
  决定它实现 `Fn` / `FnMut` / `FnOnce` 的哪一层，也决定它能不能被送到另一个线程。

- **底层机制**：

  三个调用 trait 是一条链，不是平级接口（`core/src/ops/function.rs`）：

  | trait | 超 trait | `self` | 对应捕获 |
  |-------|----------|--------|---------|
  | `FnOnce`（`:242`） | — | 按值 `self` | 拿走所有权，只能调一次 |
  | `FnMut`（`:163`） | `FnOnce` | `&mut self` | 可变借，可重复调、可改捕获物 |
  | `Fn`（`:76`） | `FnMut` | `&self` | 共享借，可重复调、不改捕获物 |

  因此 `Fn` 自动满足后两层；按值拿走 `String` 的闭包只保证 `FnOnce`。
  `shared_ref_capture_is_fn` / `mut_ref_capture_is_fn_mut` / `by_value_capture_is_fn_once`
  把这三层钉成可重复调用的行为。

  **与 US4 Send/Sync 的衔接（US3 AS2）**：
  `thread::spawn` 要求闭包 `F: Send + 'static`（`std/src/thread/functions.rs:128`）。
  捕获方式在这里分出两问，本模块覆盖第一问，C-11 覆盖第二问：

  1. **按引用捕获局部变量**（不 `move`）→ 借用活不过当前函数 → **E0373**
     （`c10_borrow_escapes_thread.rs`）。缺的是 `'static`，还没轮到 `Send`。
  2. **`move` 拿走所有权，但捕获物是 `Rc`** → 所有权已经在闭包里，仍被拒 → **E0277**
     （`Rc: !Send`，见 C-11）。缺的是线程安全标记。

  example 第 4 段用 `move` + 拥有的 `String`（`String: Send + 'static`）成功进线程，
  作为第 1 问的正向对照。US4 会把"为什么 `String` 是 `Send`、`Rc` 不是"展开成题集。

- **常见误解**：
  - **误解**："三个 Fn trait 是三种互斥的闭包种类" → **实际**：继承链。
    `Fn` 是最强的一层，可以当 `FnMut`/`FnOnce` 用。
    → **证据**：`function.rs:76/163/242`；`shared_ref_capture_is_fn` 把同一个闭包
    交给 `apply_fn` 两次。
  - **误解**："加上 `move` 就能进线程" → **实际**：`move` 只解决借用 / `'static`。
    捕获物还得是 `Send`。本模块 E0373 样本的 help 建议加 `move`，
    那只覆盖第 1 问。
    → **证据**：`borrow_escaping_to_thread_is_e0373` 对照 `c11` 的 E0277。

- **对应实验**：[`c10_closure`](../../experiments/m3-composition/examples/c10_closure.rs)
  / [tests](../../experiments/m3-composition/tests/c10_closure.rs)

---

### C-11 Smart pointer

- **一句话定义**：`Box` 独占一块堆内存；`Rc` 在单线程里用引用计数共享；
  `Arc` 在跨线程时用原子计数共享。`clone` 后两者只加计数，不复制堆上的值。

- **底层机制**：

  | 类型 | 源码 | 所有权合同 | `clone` |
  |------|------|-----------|---------|
  | `Box<T>` | `alloc/src/boxed.rs:234`（内部 `Unique<T>`） | 一个所有者 | 仅当 `T: Clone` 时复制值到新分配 |
  | `Rc<T>` | `alloc/src/rc.rs:320`；`:330` `!Send`、`:338` `!Sync` | 单线程、多句柄 | `:2513` `inc_strong()`，**不分配** |
  | `Arc<T>` | `alloc/src/sync.rs:269`；`:279` `Send`、`:281` `Sync`（`T: Send + Sync`） | 多线程、多句柄 | `:2399` 同样加计数，**不分配** |

  `rc_clone_shares_and_bumps_strong_count`：`clone` 之后 `strong_count == 2`，
  两个句柄看到同一个 `7`；`drop` 一个句柄回到 1，值还在。

  `smart_pointer_allocation_counts`：`Box::new` / `Rc::new`+`clone` / `Arc::new`+`clone`
  各 **1** 次 `allocs` —— 新建那一次，随后的 `clone` 不计。

  误用的具体后果（US3 AS3）：
  - 该独占却用 `Rc`：编译能过，但单线程外的共享合同是假的；
  - 该跨线程却用 `Rc`：**E0277**（`c11_rc_across_threads.rs`），编译失败；
  - 该单线程却用 `Arc`：行为对，多付一次原子计数（耗时是 NON-ASSERTION，
    本模块用"能否进 `spawn`"这条编译期事实区分，不用时钟）。

  `arc_clone_shares_and_is_send` 把 `Arc<u32>` 送进另一个线程，`join` 后看到同一个值。
  这就是选 `Arc` 的依据：合同要求跨线程共享，不是习惯。

- **常见误解**：
  - **误解**："`Rc::clone` 会把堆上的值 memcpy 一份" → **实际**：只加强引用计数，
    分配次数仍是 1。
    → **证据**：`rc_clone_shares_and_bumps_strong_count` +
    `smart_pointer_allocation_counts`。
  - **误解**："智能指针都能进线程 / 都不能" → **实际**：`Box`/`Arc` 在 `T: Send` 时可以，
    `Rc` 显式 `!Send`。选择依据是所有权合同。
    → **证据**：`rc_across_threads_is_e0277` 对照 `arc_clone_shares_and_is_send`。

- **对应实验**：[`c11_smart_ptr`](../../experiments/m3-composition/examples/c11_smart_ptr.rs)
  / [tests](../../experiments/m3-composition/tests/c11_smart_ptr.rs)

---

## 与后续学习的关联

FR-014 要求每项能力说明与后续 Linux / eBPF / Aya 的关联点，或显式标注为仅是基础。

| C-ID | 关联点 |
|------|-------|
| **C-08 Error handling** | 直接对应。Aya **用户态**把 syscall / netlink / 加载失败收成 `Result`，
  用 `?` 和 `From` 把 `io::Error`、Aya 自己的错误类型接到一层。
  eBPF **内核程序**侧没有 `Result` 这条语言设施，失败是返回码 / 把包丢掉。
  学会看 `?` 的 `From` 边界，就是在判断"这一层丢掉了哪些原始错误信息"——
  后面 m6 封装 `errno` 会把这件事再钉一次。 |
| **C-09 Iterator** | 直接对应，但两侧用法不同。用户态解析包、扫 map 可以用惰性链；
  **verifier 要看得见的循环**，eBPF 程序里一般不能指望 `map`+`filter` 的适配器链
  被优化成它能证明终止的形式。本实验的要点是：先能说出链在什么时候跑、
  在哪一次堆分配，再决定这段逻辑能不能下到内核。 |
| **C-10 Closure** | 直接对应。Aya 用户态的回调、`thread::spawn` 加载器、异步完成闭包，
  全部是捕获方式问题。按引用捕获局部缓冲区再送进线程，就是本模块的 E0373；
  这是 US4 Send/Sync 的入口，不是"闭包语法题"。 |
| **C-11 Smart pointer** | 直接对应。Aya 用户态的 map 句柄、`Ebpf` 对象常要跨线程共享 —— 合同是 `Arc`，
  不是 `Rc`。选错会在编译期被本实验同样的 `E0277` 拦住。
  eBPF 程序侧没有 `Rc`/`Arc` 这套堆共享；内核里的共享是 map / percpu 那一套，
  本 Feature 不写 eBPF（FR-017），只把"用户态句柄的所有权合同"钉住。 |

四项都不是"仅为理解基础"。读不懂 `?` 的 `From`、迭代器何时分配、闭包捕获、
`Rc` vs `Arc`，就读不了 Aya 用户态的加载与共享路径。
