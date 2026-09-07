# Module 3 —— OBSERVATIONS

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

## CountingAllocator 的全局性（T056 / harness-api）

`tests/c09_iterator.rs` 与 `tests/c11_smart_ptr.rs` 各自声明了

```rust
#[global_allocator]
static A: CountingAllocator = CountingAllocator::new();
```

集成测试是**各自独立的二进制**，所以分配器对**该测试二进制的全部代码**生效，
包括测试框架自己的分配。这是刻意的：

- `measure` 只统计调用线程在闭包窗口内的增量，框架启动期的分配不进断言；
- 计数器是 thread-local，并行 `#[test]` 不会互相污染数字；
- 额外用 `m3_composition::MEASURE_LOCK` 把本 crate 所有 `measure` 调用串行化，
  防止同一线程被测试框架复用时窗口重叠。

本说明满足 harness-api："使用 `CountingAllocator` 的 crate MUST 在 OBSERVATIONS 中
说明它对该 crate 全局生效"。

---

## 记录块

### C-08 / c08_error  [NON-ASSERTION]

命令：`cargo run -p m3-composition --example c08_error`

输出：

```text
=== 同一组输入：手写 match vs ? ===
  "合法端口"
    manual = Ok(443)
    ?      = Ok(443)
  "空串"
    manual = Err(Parse(Empty))
    ?      = Err(Parse(Empty))
  "非数字"
    manual = Err(Parse(BadDigit))
    ?      = Err(Parse(BadDigit))
  "本层拒绝（0）"
    manual = Err(OutOfRange)
    ?      = Err(OutOfRange)
  "本层拒绝（太大）"
    manual = Err(OutOfRange)
    ?      = Err(OutOfRange)
```

解释：
  为什么会这样：两条路径走的是同一份 `parse_u16` + 同一份本层范围检查。
  内层错误经 `From<ParseErr> for AppErr` 变成 `AppErr::Parse`；
  `?` 在 `FromResidual` 里调用同一个 `From::from`。本层拒绝（0 / 8080）
  已经是 `AppErr`，不再经过 `From`。所以五条输入的 `Ok`/`Err` 一致。
  这不能证明什么：打印一致不能证明两种写法生成同一份机器码，也不能证明
  所有 `?` 都零开销。它只证明**这五个输入的行为**一致。缺 `From` 的拒绝
  根本不会出现在这段输出里 —— 那是编译失败，见下一块。

架构相关性：可跨架构推广。`Result` + `From` + `?` 是语言规则，与 endian / 字长无关。

---

### C-08 / c08_missing_from  [NON-ASSERTION]

命令：`cargo test -p m3-composition --test c08_error missing_from_impl_is_e0277`

完整 stderr 落盘于 `target/compile-fail/c08_missing_from.stderr`（§C4.3，禁止相等比较）。
实际错误码集合：**E0277**（仅此一个，已写入样本首行 `//! EXPECT`）。

诊断要点（NON-ASSERTION 措辞）：`` `?` couldn't convert the error to `Outer` ``，
并注明 `From<Inner>` is not implemented for `Outer`。

解释：
  为什么会这样：`?` 的失败路径要求 `Outer: From<Inner>`。两个空枚举之间没有这份 impl，
  候选选择在单态化之前失败，错误码是 E0277。
  这不能证明什么：诊断英文全文会随 rustc 版本漂移，不能当稳定断言；
  也不能由"报了 E0277"推出"运行期会 panic"——这段代码从未被执行。

架构相关性：可跨架构推广。错误码是 rustc 稳定契约。

---

### C-09 / c09_iterator  [NON-ASSERTION]

命令：`cargo run -p m3-composition --example c09_iterator`

输出：

```text
=== 1. 惰性：副作用出现在消费时，且按元素走完整条链 ===
  链已构造，尚未消费
  开始 for_each：
    map 0
    filter 0
    map 1
    filter 2
    map 2
    filter 4
    for_each 4
    map 3
    filter 6
    for_each 6

=== 2. 同一逻辑：手写循环 vs 迭代器，元素结果 ===
  sum  loop=12 iter=12
  collect loop=[0, 2, 4, 6] iter=[0, 2, 4, 6]
```

解释：
  为什么会这样：`Map::next` 是 `self.iter.next().map(&mut self.f)`，一次取一个上游元素。
  所以"链已构造"那一行没有 `map` 打印；消费开始后，每个元素先 `map` 再 `filter`，
  通过的才进 `for_each`。`0*2=0` 与 `1*2=2` 被 filter 丢掉，没有 `for_each`。
  `sum`/`collect` 数字一致，是因为两种写法对 `(0..4)` 做了同一件事。
  这不能证明什么：打印顺序不能证明"迭代器更快或更慢"（耗时是禁止断言的）；
  元素结果一致不能证明分配次数一致 —— 那是 `iterator_chain_allocation_counts`
  用 `measure` 钉住的另一件事。`n=4` 的示例输出也不能外推到别的 `n` 的打印行数以外的结论。

架构相关性：可跨架构推广。惰性与 `next` 的形状是语言 / 标准库规则。
具体分配次数在 pinned 1.98.0 + 本机 allocator 下是确定的，增长策略随 `Vec`
实现，故断言锁的是关系（`sum` 为 0、精确 `collect` 为 1、空 Vec+push 更多），
而不是"所有 Rust 版本 push 都是两次"。

---

### C-10 / c10_closure  [NON-ASSERTION]

命令：`cargo run -p m3-composition --example c10_closure`

输出：

```text
=== 1. 按共享引用捕获：可重复调用 ===
  len twice: 3 3

=== 2. 按可变引用捕获：每次调用改计数 ===
  after two calls: counter=2 last=2

=== 3. 按值捕获：调用一次就消费 String ===
  took = "pkt"

=== 4. move + 拥有 String：可以进线程（对照 compile_fail 的借用版） ===
  spawned len = 2
```

解释：
  为什么会这样：共享借不修改捕获物，两次 `len` 都是 3；可变借每次 `+= 1`，
  从 0 起两次后是 2；按值拿走 `String`，调用之后原变量不可用。
  第 4 段 `move` 把 `String` 的所有权送进闭包，`String: Send + 'static`，
  满足 `spawn` 的两个 bound，所以能 `join` 到 `2`（`"ok".len()`）。
  这不能证明什么：第 4 段成功不能证明"加 `move` 就一定能 spawn"——
  捕获 `Rc` 时 `move` 仍然 E0277（C-11）。线程 `join` 的返回值也不能
  用来断言线程调度顺序。

架构相关性：可跨架构推广。`Fn*` 分层与 `spawn` 的 bound 是语言规则。
`join` 到 2 只说明这次运行完成了，不是并发正确性的证明（那是 US4）。

---

### C-10 / c10_borrow_escapes_thread  [NON-ASSERTION]

命令：`cargo test -p m3-composition --test c10_closure borrow_escaping_to_thread_is_e0373`

完整 stderr 落盘于 `target/compile-fail/c10_borrow_escapes_thread.stderr`。
实际错误码集合：**E0373**（仅此一个）。

诊断要点：`closure may outlive the current function, but it borrows local`；
help 建议加 `move`。

解释：
  为什么会这样：`spawn` 要求 `F: 'static`。按引用捕获局部 `String`，借用活不过
  `spawn_borrow` 返回，所以是 E0373，还没轮到检查 `Send`。
  这不能证明什么：help 建议的 `move` 只解决这一问；加上 `move` 之后若捕获物
  不是 `Send`，会换成 E0277，不是"编译就过了"。诊断全文不可断言。

架构相关性：可跨架构推广。

---

### C-11 / c11_smart_ptr  [NON-ASSERTION]

命令：`cargo run -p m3-composition --example c11_smart_ptr`

输出：

```text
=== 1. Box：独占，解引用得到那个值 ===
  box = 7

=== 2. Rc：clone 加计数，不复制堆上的值 ===
  strong after clone = 2
  both see 7 7
  strong after drop  = 1

=== 3. Arc：合同相同，但可以进线程 ===
  strong = 2
  other thread saw 7
  strong after join = 1
```

解释：
  为什么会这样：`Rc::clone` / `Arc::clone` 走 `inc_strong`，两个句柄看到同一个 7，
  计数从 1 到 2；`drop` 一个句柄回到 1，值还在。`Arc<u32>` 满足 `Send`，
  所以另一个线程能读到 7；`join` 后只剩主线程那一个强引用。
  这不能证明什么：打印 `strong = 2` 不能证明 `clone` 零分配 ——
  那是 `smart_pointer_allocation_counts` 用 `measure` 钉的。
  另一个线程看到 7 不能证明 `Arc` 比 `Rc`"更快"，也不能证明任意 `T` 都能进线程
  （还要 `T: Send + Sync`）。耗时一律不作为证据。

架构相关性：可跨架构推广。引用计数语义与 `Send`/`!Send` 是语言 / 标准库规则。
原子操作的内存序表现是架构敏感的，但本实验不断言内存序（那是 C-14）。

---

### C-11 / c11_rc_across_threads  [NON-ASSERTION]

命令：`cargo test -p m3-composition --test c11_smart_ptr rc_across_threads_is_e0277`

完整 stderr 落盘于 `target/compile-fail/c11_rc_across_threads.stderr`。
实际错误码集合：**E0277**（仅此一个）。

诊断要点：`` `Rc<u32>` cannot be sent between threads safely ``，
并引用 `spawn` 的 `F: Send + 'static` bound（`std/src/thread/functions.rs:128`）。

解释：
  为什么会这样：样本已经 `move`，所有权在闭包里，`'static` 满足了；
  挡下来的是 `Rc` 的显式 `!Send`（`rc.rs:330`）。这是 C-10 E0373 的另一侧。
  这不能证明什么：E0277 在本 Feature 里还会因缺 trait bound 出现（C-07、C-08），
  看到 E0277 不能自动推出"这是 Send 问题"——要对准诊断里的 trait 名字。
  全文不可断言。

架构相关性：可跨架构推广。
