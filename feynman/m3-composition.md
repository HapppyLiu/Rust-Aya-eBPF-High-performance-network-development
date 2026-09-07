# Feynman: Module 3 — 组合能力

**Capabilities covered**: C-08, C-09, C-10, C-11

---

## 1. 用自己的话解释

面向"没学过 Rust 但懂 C"的同事。

C 程序员每天都在做四件事：看函数返回值是不是错误、写 `for` 循环加工数组、
写个回调函把外面的变量用掉、用指针共享一块堆内存。Rust 把这四件事收进类型里，
并且多了一件 C 没有的：用**分配次数**而不是"感觉快不快"来谈代价。

**第一，失败不再是一个碰巧叫 `errno` 的整数。**
C 里你返回 `-1`，调用方可以假装没看见。Rust 把成功值和失败值塞进同一个带标签的结果里，
调用方必须说清两条路怎么走。那个问号运算符，成功时把值拿出来继续用；
失败时不是直接把内层失败值扔回去 —— 它会先问"外层失败类型知不知道怎么从内层变过来"。
这份"怎么变"是一个转换接口。两边枚举对不上、又没写转换，编译器在程序开跑之前就拒绝。
手写 `switch` 式匹配和问号，对同一组输入可以给出同一个成功或失败；
变的是你还能不能漏处理，不是这几个输入的答案。

**第二，加工序列的那一串调用，默认什么都不做。**
C 里 `for (i=0; i<n; i++) a[i] = f(a[i]);` 写下来就已经跑了。
Rust 的"对每个元素做 f、再过滤"先只是在栈上放了两个小结构体：上游在哪、闭包是谁。
真正取下一个元素的时候，才对**这一个**元素做完 f 再做过滤，然后才取下一个。
所以副作用的顺序是交错的，不是"先把第一段全部做完"。
求和一条只含整数的链，堆上一次都不用分配；收集成动态数组，如果长度事先知道，
就按长度一次分配。从空动态数组一个一个 `push`，起点容量是 0，要增长 ——
元素可以相同，分配次数不必相同。

**第三，回调对外部变量做了哪一种动作，决定它能被怎么调用、能不能送到另一个线程。**
只看、不改，就可以重复调用；要改，就得独占地改；要把值拿走，就只能调用一次。
这三个不是平级的三种回调，是一层包一层：能只看的，也能被用在"只调一次"的位置。
送到另一个线程有两问。第一问：你是不是只借了当前函数的局部变量？借的话，
那个线程可能比当前函数活得长，编译器直接拒绝。第二问：就算你拿走了所有权，
这个类型允不允许跨线程？不允许，还是拒绝，只是错误换成另一条规则。
第一问在本模块用"借用逃出线程"演示；第二问留给智能指针。

**第四，堆上的指针有三份合同，不是三个档次。**
独占：一块堆内存一个主人，主人走了内存就还。
单线程共享：好几只手抓同一块，clone 只是再加一只手，不把值再拷一份。
跨线程共享：合同看起来一样，但计数要用原子操作，类型还得允许被送到别的线程。
选错的后果是具体的：该跨线程却用了单线程那份，编译失败；
该独占却用了共享，编译能过，合同却是假的。
"大家都用跨线程那份"不是选择依据 —— 依据是你到底要几个主人、几个线程。

---

## 2. 最小示例

### C-08 Error handling — `?` 经 `From` 传播

```rust
impl From<ParseErr> for AppErr { fn from(e: ParseErr) -> Self { AppErr::Parse(e) } }
fn parse_port(s: &str) -> Result<u16, AppErr> {
    let n = parse_u16(s)?;
    check_port(n)
}
```

完整观察：[`examples/c08_error.rs`](../experiments/m3-composition/examples/c08_error.rs)（20 行）

### C-09 Iterator — 惰性链，分配发生在终端

```rust
let s: i32 = (0..8).map(|x| x * 2).sum();          // 0 次堆分配
let v: Vec<i32> = (0..8).map(|x| x * 2).collect(); // 1 次（精确 hint）
```

完整观察：[`examples/c09_iterator.rs`](../experiments/m3-composition/examples/c09_iterator.rs)（36 行）

### C-10 Closure — 三种捕获，三种 `self`

```rust
let f = || s.len();            // 共享借 → Fn
let mut g = || { n += 1; n };  // 可变借 → FnMut
let h = || owned;              // 按值   → FnOnce
```

完整观察：[`examples/c10_closure.rs`](../experiments/m3-composition/examples/c10_closure.rs)（29 行）

### C-11 Smart pointer — clone 加计数，不拷贝值

```rust
let a = Rc::new(7u32);
let b = Rc::clone(&a);
assert_eq!(Rc::strong_count(&a), 2);
```

完整观察：[`examples/c11_smart_ptr.rs`](../experiments/m3-composition/examples/c11_smart_ptr.rs)（29 行）

---

## 3. 底层机制

每条论断带依据。审查时优先核对这里。

1. `Result` 是两变体枚举，失败值有自己的类型参数。
   **依据**：`core/src/result.rs:557`（`source-refs.md`）。
2. `?` 失败路径调用 `From::from`，不是裸 `return Err(e)`。
   **依据**：`core/src/result.rs:2185/2192`。
3. 缺 `From` 时编译器报 E0277，拒绝发生在运行之前。
   **依据**：`tests/c08_error.rs::missing_from_impl_is_e0277`。
4. 手写 `match` 与 `?` 对五条输入的 `Ok`/`Err` 一致。
   **依据**：`tests/c08_error.rs::manual_and_question_mark_agree_on_every_path`。
5. `std::error::Error` 是 `core::error::Error` 的再导出。
   **依据**：`std/src/error.rs:4`、`core/src/error.rs:59`。
6. `map` 只构造适配器，不拉动 `next`。
   **依据**：`core/src/iter/traits/iterator.rs:831`；
   OBSERVATIONS「C-09：链已构造，尚未消费」。
7. 一次 `Map::next` 取一个上游元素并立刻调用闭包，故副作用按元素交错。
   **依据**：`core/src/iter/adapters/map.rs:106`；
   OBSERVATIONS「C-09：副作用出现在消费时」。
8. `Map` 转发精确 `size_hint`，因此 `collect::<Vec<_>>` 可以一次按长度分配。
   **依据**：`core/src/iter/adapters/map.rs:111`；
   `tests/c09_iterator.rs::iterator_chain_allocation_counts`。
9. `map`+`sum` 与只构造链不消费，堆分配都是 0 次；空 `Vec`+`push` 的分配事件更多。
   **依据**：`tests/c09_iterator.rs::iterator_chain_allocation_counts`。
10. 手写循环与迭代器的元素结果一致，这推不出分配次数一致。
    **依据**：`tests/c09_iterator.rs::loop_and_iterator_agree_on_values`。
11. `Fn` / `FnMut` / `FnOnce` 是继承链，`self` 分别是 `&self` / `&mut self` / 按值。
    **依据**：`core/src/ops/function.rs:76/163/242`。
12. 共享引用捕获可重复调用；可变引用捕获两次各加一；按值捕获调用一次即消费。
    **依据**：`tests/c10_closure.rs::shared_ref_capture_is_fn` /
    `mut_ref_capture_is_fn_mut` / `by_value_capture_is_fn_once`。
13. 按引用捕获局部变量再 `spawn`（不 `move`）报 E0373：缺的是 `'static`。
    **依据**：`tests/c10_closure.rs::borrow_escaping_to_thread_is_e0373`；
    `std/src/thread/functions.rs:128`。
14. `Rc` 显式 `!Send` / `!Sync`；`clone` 走 `inc_strong`，不新分配。
    **依据**：`alloc/src/rc.rs:330/338/2513`；
    `tests/c11_smart_ptr.rs::rc_clone_shares_and_bumps_strong_count`。
15. `Box::new`、`Rc::new`+`clone`、`Arc::new`+`clone` 各 1 次堆分配。
    **依据**：`tests/c11_smart_ptr.rs::smart_pointer_allocation_counts`。
16. `Rc` 即使 `move` 进 `spawn` 也报 E0277；`Arc<u32>` 可以。
    **依据**：`tests/c11_smart_ptr.rs::rc_across_threads_is_e0277` /
    `arc_clone_shares_and_is_send`；`alloc/src/sync.rs:279`。

本节共 **16** 条论断，每条带一处依据。核对：

```bash
grep -c '\*\*依据\*\*' feynman/m3-composition.md   # 期望 16
```

---

## 4. 常见误区

### C-08

- **误解**：`?` 就是提前 `return Err(e)`。
  → **实际**：失败路径上还有 `From::from`；缺这份 impl 是 E0277，不是运行期错误。
  → **证据**：`missing_from_impl_is_e0277`；`result.rs:2192`。
- **误解**：改成 `Result` 之后行为会变。
  → **实际**：本实验五条路径手写与 `?` 的结果一致。变的是类型强制处理失败。
  → **证据**：`manual_and_question_mark_agree_on_every_path`。

### C-09

- **误解**：写出 `map` 那一行就已经跑完了。
  → **实际**：适配器惰性；"链已构造"那一行没有副作用。
  → **证据**：OBSERVATIONS「C-09：链已构造，尚未消费」；`map.rs:106`。
- **误解**：迭代器一定比手写循环少分配，或元素一致则分配次数也一致。
  → **实际**：看终端消费者和 `size_hint`。`sum` 零次，精确 `collect` 一次，空 `Vec`+`push` 更多。
  → **证据**：`iterator_chain_allocation_counts`。

### C-10

- **误解**：三个 `Fn*` 是互斥的三种闭包。
  → **实际**：继承链。`Fn` 最强，可以当后两层用。
  → **证据**：`function.rs:76/163/242`；`shared_ref_capture_is_fn`。
- **误解**：加上 `move` 就能进线程。
  → **实际**：`move` 只解决借用 / `'static`。捕获物还得是 `Send`。
  → **证据**：`borrow_escaping_to_thread_is_e0373` 对照 C-11 的 E0277。

### C-11

- **误解**：`Rc::clone` 会 memcpy 堆上的值。
  → **实际**：只加计数，`allocs` 仍是 1。
  → **证据**：`rc_clone_shares_and_bumps_strong_count` + `smart_pointer_allocation_counts`。
- **误解**：智能指针都能进线程，或选 `Arc` 只是习惯。
  → **实际**：`Rc` 显式 `!Send`；选 `Arc` 是因为合同要求跨线程。
  → **证据**：`rc_across_threads_is_e0277` 对照 `arc_clone_shares_and_is_send`。

---

## 5. 验证性问题

1. 手写 `match` 和 `?` 对同一输入返回值相同。若据此断言"两种写法的机器码也相同"，错在哪一步？
   **回答**：把"结果"当成了"路径"。行为一致只说明走了同一份 `From` 和同一份检查；
   问号还要经 `FromResidual` 选 impl，生成代码可以不同。本实验不断言 IR。
   **指向**：(a) `tests/c08_error.rs::manual_and_question_mark_agree_on_every_path`

2. 一条 `map`+`filter` 链，在 `collect` 之前打印"链构造完毕"。
   若此时已经看到 `map` 的副作用，说明对惰性的哪一处判断错了？
   **回答**：错在把"写出适配器"当成"拉动 `next`"。`Map::next` 才调用闭包；
   构造函数只保存上游和闭包。
   **指向**：(c) OBSERVATIONS「C-09：链已构造，尚未消费」；
   (b) `source-refs.md` `map.rs:106`

3. `collect` 一条带精确长度提示的 `map` 链，和手写 `Vec::new`+`push` 同样多个元素，
   分配次数为什么可以不同？
   **回答**：差别来自"事先知不知道容量"，不是元素个数。`Map::size_hint` 原样转发
   `Range` 的精确 hint，一次 `with_capacity`；空 `Vec` 从 0 增长。
   **指向**：(a) `tests/c09_iterator.rs::iterator_chain_allocation_counts`

4. 一个闭包按引用捕获了局部变量。给它加 `move` 再 `spawn`，就能编译了吗？
   **回答**：不一定。`move` 解决 `'static` / E0373。若捕获物是 `Rc`，换成 E0277。
   本模块 C-10 样本覆盖第一侧，C-11 样本覆盖第二侧。
   **指向**：(a) `tests/c10_closure.rs::borrow_escaping_to_thread_is_e0373` 与
   `tests/c11_smart_ptr.rs::rc_across_threads_is_e0277`

5. `Arc::clone` 的分配次数与 `Rc::clone` 相同，是否就能说两者代价相同？
   **回答**：不能。本模块禁止用耗时证明代价。确定性量只能说明两者都**不新分配**；
   `Arc` 加计数走原子操作，那笔账要到 C-14 才展开，且仍不能靠时钟断言。
   能断言的差别是合同：`Arc<u32>` 能进 `spawn`，`Rc<u32>` 不能。
   **指向**：(a) `tests/c11_smart_ptr.rs::smart_pointer_allocation_counts` 与
   `arc_clone_shares_and_is_send`

6. 后续在用户态持有一份要跨线程共享的句柄时，选 `Rc` 还是 `Arc`？
   **回答**：选 `Arc`。依据是合同要求多个线程、多个句柄，不是习惯。
   `Rc` 会在 `spawn` 处被同一条 E0277 拦住。
   **指向**：(a) `tests/c11_smart_ptr.rs::rc_across_threads_is_e0277`；
   (b) `alloc/src/rc.rs:330` 与 `alloc/src/sync.rs:279`

---

## 检验结果

| # | 检验项 | 合格标准（可判定） | 状态 |
|---|-------|-----------------|------|
| 1 | 自述概念 | 第 1 节存在；Rust 术语（`Result` / `?` / 惰性 / `Fn*` / `Rc` / `Arc`）均在首次出现时用 C 世界的话解释 | **pass** |
| 2 | 最小示例 | C-08 / C-09 / C-10 / C-11 各一段，均 ≤15 行，并链接到 `examples/` | **pass** |
| 3 | 底层机制 | 第 3 节 16 条论断，每条带依据标记；`grep -c '\*\*依据\*\*'` = 16 | **pass** |
| 4 | 常见误区 | 每个 covered capability ≥1 条，三段式齐备（C-08×2、C-09×2、C-10×2、C-11×2） | **pass** |
| 5 | 回答问题 | 6 个问题（≥5），每题回答指向断言名 / 源码引用 / 观测块标题之一 | **pass** |

**五项是合取。** 本表全部 pass → m3 的 Capability 可以进入 `accepted`。
