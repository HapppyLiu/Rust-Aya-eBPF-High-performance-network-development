# Module 1 —— Learner Track §1 问题逐条作答

**Story**: US1 | **Capabilities**: C-02, C-03, C-04 |
**对应问题**：[`learner/m1-ownership/guide.md`](../../learner/m1-ownership/guide.md) §1

> 本文件属于 **Answer Track**。
>
> 它按 `guide.md` §1 的题号**逐条**作答，并指向承载完整论述的既有产物
> （`concept.md` / `source-refs.md` / `tests/` / `OBSERVATIONS.md`）。
> 按契约 §H1.1，本文件**不新增**任何答案义务，只是既有 Answer Track 内容的一个
> 按题号索引的视图 —— 因为 `concept.md` 是按**概念**组织的，
> 而 `guide.md` §1 是按**问题**组织的，两者需要一层映射。
>
> C-01 的五题未收录：那五题已在 `guide.md` 中自行作答完毕。

---

## C-02 Move semantics（移动语义）

### Q1. `let b = a;` 这一行，对不同类型可能有两种完全不同的含义。是哪两种？分界线在哪？

两种含义是**移动**（move）与**复制**（copy）。

分界线是 **`Copy` trait** —— 不是 `Clone`。

- 类型实现了 `Copy`：赋值是按位复制，`a` **仍然可用**，两个独立的值。
- 类型没有实现 `Copy`：赋值是移动，`a` **失效**，此后读它是编译错误 `E0382`。

`Clone` 完全不参与这个判断。这是本模块最容易踩的一个坑，值得说清楚：

| | `Copy` | `Clone` |
|---|-------|--------|
| 是什么 | **标记** trait（marker trait），无方法体 | 普通 trait，提供 `.clone()` 方法 |
| 谁触发 | 编译器，在 `=` / 传参 / 返回时**隐式**触发 | 你**显式**写 `.clone()` 才发生 |
| 影响 `=` 的语义吗 | **是** | **否** |

最直接的反例：`String` 实现了 `Clone` 但**没有** `Copy`。

```rust
let a = String::from("x");
let b = a;        // 移动，不是复制
a.len()           // error[E0382]: borrow of moved value: `a`
```

本仓库的 `Label` 同样如此：它 `#[derive(Clone)]` 但不是 `Copy`，
所以 `consume_label(label)` 之后 `label` 不可用。

**为什么 `Copy` 是分界线而 `Clone` 不是**：`Copy` 的语义承诺是
"按位复制即语义正确，且复制品不需要额外清理"。编译器可以据此安全地把 `=` 当作复制。
`Clone` 没有这个承诺 —— `.clone()` 可能要分配堆内存、复制文件描述符，
是一次可能失败、可能昂贵的操作。把它设成隐式会让 `=` 变成不可预测的开销。

- 完整论述：[`concept.md` § C-02](./concept.md)
- 断言：`tests/c02_move.rs::copy_types_remain_usable_after_assignment`、
  `::copy_bound_distinguishes_the_two_type_families`（用 `T: Copy` 约束把差别变成编译期事实）
- 反面：`compile_fail/c01_use_after_move.rs` → **E0382**

---

### Q2. 「移动」在机器层面到底发生了什么？是不是"把数据搬到另一个地方"？

**不是**。这个问题要把"栈上的值"和"堆上的数据"分开看。

- **栈上部分**：可能发生一次按位拷贝（对 `String` 就是那个 `(ptr, len, cap)` 三元组），
  也可能被优化器完全消除（两个 local 被分配到同一位置）。
- **堆上部分**：**一个字节都没动**。`ptr` 指向的缓冲区原地不动。

而语义上真正关键的变化**根本不在机器层面**，而在编译器的簿记：
移动之后源位置被标记为"已移出"，此后读它是编译错误，
且**作用域末尾不再为它插入 drop**。

MIR 给出了最干净的证据（OBSERVATIONS「C-02：移动后的源 local 没有 drop」）：

```text
_22 = move _20;        // 整个"移动"就是这一条语句

$ grep -c 'drop(_20)' …   # 移动的源
0
$ grep -c 'drop(_22)' …   # 移动的目标
2                          # 正常路径 + unwind 路径各一份
```

源 local 的 drop terminator 数量是 **0**。不是"三次移动里只有一次生效"，
而是编译器**根本不为已移出的 local 生成 drop**。

所以一句话：**移动搬走的不是数据，是销毁责任。**

> 顺带纠正一个常见说法："移动就是让新变量指向这块资源，旧变量不再有效"。
> 这个"指向"的措辞容易被读成指针别名，从而以为两个变量曾经同时指向同一块内存。
> 实际上移动之后**只有一个**有效的所有者，旧位置不是"指向但失效的指针"，
> 而是编译器视角下"不存在的值"—— 那块栈空间原样躺着，没有被清零，也没有被回收。

- 完整论述：[`concept.md` § C-02](./concept.md)
- 断言：`tests/c02_move.rs::move_does_not_duplicate_drops`（链式移动三次，销毁计数仍为 1）
- IR 观察：`OBSERVATIONS.md`「C-02：移动后的源 local 没有 drop」

---

### Q3. 为什么有些类型可以在赋值后继续用原变量，有些不行？谁决定的？

由**类型是否实现 `Copy`** 决定（见 Q1）。

"谁决定"有两层，两层都要说：

**第一层：类型的作者**决定是否 `derive`/`impl Copy`。

**第二层：语言限制了作者的选择空间**，两个硬约束：

1. **所有字段都必须是 `Copy`**。含 `String` / `Vec` / `Box` 字段的类型无法是 `Copy`。
2. **不能同时实现 `Drop`** → `error[E0184]`。

第 2 条的理由是本模块的核心之一：若一个值既能被随意复制、又有销毁行为，
那么 N 份副本各自销毁一次，**同一份资源被释放 N 次** —— 正是 C 里的 double free。

标准库把这个理由写在了 `Copy` 的文档里（`core/src/marker.rs:419`）：

> any type implementing `Drop` can't be `Copy`

所以互斥不是语法洁癖，是防 double free 的语言级机制。

- 完整论述：[`concept.md` § C-02](./concept.md)
- 源码：`core/src/marker.rs:454`（`Copy` 定义）、`:419`（互斥理由原文）
- 反面：`compile_fail/c02_copy_with_drop.rs` → **E0184**

---

### Q4. 有没有办法"把值从一个位置取走，但那个位置还得留着能用"？这需要什么前提条件？

有，`core::mem` 里提供了三个，前提条件各不相同：

| 函数 | 做什么 | 前提条件 |
|------|-------|---------|
| `mem::replace(&mut dest, src)` | 取出 `dest` 的值，放入 `src` | **无任何 trait 约束** —— 调用方自带替补值 |
| `mem::take(&mut dest)` | 取出 `dest` 的值，放入默认值 | `T: Default` |
| `mem::swap(&mut a, &mut b)` | 两个位置互换 | **无任何 trait 约束** |

前提条件**不是** `Clone`。`replace` 与 `swap` 对 `T` 没有任何约束
（已实测：一个既不 `Clone` 也不 `Default` 的裸结构体，`mem::replace` 照常编译通过）。
`take` 之所以要 `Default`，只是因为它得**自己**找一个替补值放回去，
而 `Default::default()` 是它唯一能凭空造出的值。

三者共同体现的原则是本题真正的答案：

> **原位置绝不允许留下无效状态。**

这正是 Rust 与 C 的分野。C 里 `free(p)` 之后 `p` 是悬垂指针，语言不管；
Rust 里你**没法制造**出那个中间状态 —— 要取走，就必须同时放回一个合法值。

- 完整论述：[`concept.md` § C-02](./concept.md) 的三函数对照表
- 源码：`core/src/mem/mod.rs:953`（`replace`）、`:886`（`take`）、`:822`（`swap`）
- 断言：`tests/c02_move.rs::replace_swaps_in_a_caller_supplied_value`、
  `::take_leaves_the_default_value_behind`、
  `::neither_replace_nor_take_leaves_an_invalid_state`

---

### Q5. 有没有办法让一个值**永远不被销毁**？如果有，那它占的资源去哪了？这算内存泄漏还是 UB？

**有**，而且不止一种，全部是**安全**代码：

| 手段 | 机制 |
|------|------|
| `mem::forget(t)` | 取得所有权后既不销毁也不归还。实现只有一行 `ManuallyDrop::new(t)` |
| `ManuallyDrop<T>` | 包裹之后 drop glue 跳过它 |
| `Box::leak(b)` | 交出 `&'static mut T`，堆内存永不释放 |
| `Rc` / `Arc` 循环引用 | 强引用计数永不归零 |

**资源去哪了**：还占着。堆内存没有还给分配器，文件描述符没有关闭，锁没有释放。
进程退出时操作系统会回收内存，但在进程存活期间它就是被占住了。

**这算泄漏，不算 UB。** 这个区分是本模块最反直觉、也最重要的一条：

| | UB（未定义行为） | 泄漏 |
|---|---------------|------|
| 含义 | 程序的**含义未定义**，编译器可以做任何事 | 资源未回收，但程序含义**完全明确** |
| 会不会读到无效数据 | 会（这正是危险所在） | **不会** |
| 是否破坏内存安全 | **是** | **否** |

`mem::forget` 之所以**不需要** `unsafe`，理由就在这里：泄漏不违反内存安全。
没有任何人会读到无效数据 —— 那块内存只是没人再管它了。
查一眼签名就能确认：`core/src/mem/mod.rs:189` 上没有 `unsafe`。

但**泄漏不等于无害**。泄漏一个 `MutexGuard` 会让那把锁永远不释放，
泄漏 socket 会耗尽文件描述符。它只是不属于**内存安全**问题这个范畴。

> 这一题若答"不会（没有办法）"，是把"Rust 保证内存安全"过度推广成了
> "Rust 保证资源一定被回收"。后者从来不是 Rust 的承诺 ——
> 所有权系统保证的是"不会用到已释放的东西"，而不是"一定会释放"。
> 两者的区别在 US5（`unsafe` 与 UB）会被反复用到。

- 完整论述：[`concept.md` § C-02](./concept.md)
- 源码：`core/src/mem/mod.rs:189`（`forget`，注意签名无 `unsafe`）
- 断言：`tests/c02_move.rs::forget_suppresses_drop`（销毁计数为 0，程序照常运行）

---

## C-03 Borrowing（借用）

### Q1. 借用规则是什么？用一句话说清楚"什么组合是允许的"。

> 同一时刻，对同一个值，**要么**存在任意多个不可变借用 `&T`，
> **要么**存在恰好一个可变借用 `&mut T` —— 两者不可共存。

口诀是"共享不可变，可变不共享"。

有一个关键限定词容易被忽略：**"同一时刻"的判定标准是借用的活跃区间是否重叠**，
不是变量绑定是否还在作用域内（详见 Q4）。很多"我明明只用了一个 `&mut`"的困惑
都出在这里。

- 完整论述：[`concept.md` § C-03](./concept.md)
- 断言：`tests/c03_borrow.rs::many_shared_borrows_coexist`

---

### Q2. 为什么要有这条规则？它防的具体是哪一类 bug？举一个 C 里的例子。

防的是**在别人持有引用期间改变数据的结构**。

C 里的经典例子（迭代器失效 / 悬垂指针）：

```c
int *p = &vec->data[0];   // 拿到内部元素的指针
vector_push(vec, 42);     // 内部 realloc → 旧缓冲区被 free
printf("%d\n", *p);       // p 已悬垂 → use-after-free
```

这段代码在 C 里编译得干干净净，运行时可能"看起来正常"（旧内存还没被复用），
在生产环境的某个负载下才崩 —— 是最难查的一类 bug。

同样的形状在 Rust 里编译不过：`compile_fail/c03_mut_while_shared.rs` → **E0502**。
`&data[0]` 是不可变借用，`data.push(4)` 需要可变借用，两者区间重叠。

**还有更深一层**，值得现在就点出来：这条规则同时是**数据竞争**的免疫机制。
数据竞争的成立需要三个条件 —— 至少两个线程、至少一个是写、没有同步。
而"可变不共享"直接排除了"共享 + 写"这个组合，
所以单线程的别名规则和多线程的竞争防护是**同一条规则的两个后果**。
这是 US4（`Send`/`Sync`）的地基。

- 完整论述：[`concept.md` § C-03](./concept.md)
- 反面：`compile_fail/c03_mut_while_shared.rs` → **E0502**

---

### Q3. 同一条规则，在 Rust 里其实有**两个**执行时机。分别是哪两个？各自失败时会发生什么？

| | 编译期 | 运行期 |
|---|-------|-------|
| 执行者 | borrow checker（`rustc_borrowck`，**编译器内建**） | `RefCell` 的 `BorrowCounter` |
| 数据结构 | 全函数控制流图 + 借用区间分析 | **一个 `isize`**（`core/src/cell.rs:945`） |
| 失败形式 | **编译失败**：`E0499`（两个 `&mut`）/ `E0502`（`&mut` 与 `&` 共存） | **panic**：`already borrowed` / `already mutably borrowed` |
| 代价 | 编译时间 | 一次加减 + 一次符号判断；**错误推迟到线上** |

运行期版本的编码方案简单得出人意料（`core/src/cell.rs:945-956`）：

- `0`（`UNUSED`）= 没有借用
- **正数** = 当前不可变借用的个数（每次 `borrow()` 加一）
- **负数** = 存在可变借用（`borrow_mut()` 减一）

整个"运行期借用检查"就是对这一个整数做加减和符号判断。
拿它和编译期版本（全函数控制流分析）对比，能直接量化这笔交换：
换来的是**能表达编译器证明不了的共享模式**，付出的是错误从"编译不过"变成"线上崩溃"。

**关键：规则本身一字未改**，只是检查时机不同。
`tests/c03_borrow.rs::sequential_borrows_are_legal_in_both_regimes` 正是为断言这一点而写 ——
合法的顺序访问在两种时机下都通过。所以 `RefCell` 不是"绕过"借用规则，是"换个执行者"。

**还有一条容易忽略的性质**：borrow checker 是**保守**的。
它拒绝的是"它无法证明安全"的代码，而不是"它证明了不安全"的代码。
被拒绝不代表你的代码真有 bug，可能只是它推不出来。
这个失败模式与 eBPF verifier 完全同源 —— 这也是 C-03 被称为"理解 verifier 的最佳类比"的原因。

- 完整论述：[`concept.md` § C-03](./concept.md) 的两时机对照表
- 源码：`core/src/cell.rs:945`（`BorrowCounter = isize`）、`:946`/`:949`/`:954`（编码方案）、
  `:924`（panic 路径）
- 编译期一侧属 **reference-fallback**：`rustc_borrowck` 是编译器的一部分，不在 `library/` 下
- 断言：`::sequential_borrows_are_legal_in_both_regimes`、`::refcell_double_borrow_panics_at_runtime`

---

### Q4. 一个借用从哪一行开始算、到哪一行结束？是到作用域末尾吗？

**不是到作用域末尾。** 借用从产生的那一行开始，到**最后一次使用**的那一行结束 ——
这叫 **NLL**（non-lexical lifetimes，非词法生命周期）。
与变量绑定是否还在作用域内**无关**。

```rust
let mut c = Counter::new(0);
let m = &mut c;
m.add(5);          // ← m 的借用在这一行结束
assert_eq!(c.get(), 5);   // 合法：借用已结束，尽管 m 这个绑定还在作用域里
```

`tests/c03_borrow.rs::nll_ends_borrow_at_last_use` **能编译**这一事实本身就是证明 ——
若借用持续到作用域末尾，那段代码不可能通过编译。

反面样本 `compile_fail/c03_two_mut_borrows.rs` 刻意把 `*first += 1` 放在 `*second += 1`
**之后**，就是为了把两个借用的活跃区间强行交错。诊断的三个标注正好是判定所需的全部信息：

```text
15 |     let first = &mut c.value;
   |                 ------------ first mutable borrow occurs here
16 |     let second = &mut c.value;
   |                  ^^^^^^^^^^^^ second mutable borrow occurs here
18 |     *first += 1;
   |     ----------- first borrow later used here      ← 关键的第三条
```

第三条是关键：若删掉 18 行那次使用，第一个借用会在 15 行之后立即结束，
两个借用不再重叠，这段代码就**合法**了。

**推论**：借用冲突的成立条件是"两个借用区间**重叠**"，
而不是"存在两个 `&mut` 绑定"。

- 完整论述：[`concept.md` § C-03](./concept.md)
- 断言：`tests/c03_borrow.rs::nll_ends_borrow_at_last_use`
- 反面：`compile_fail/c03_two_mut_borrows.rs` → **E0499**
- 诊断抄录：`OBSERVATIONS.md` 的 `c03_two_mut_borrows` 记录块

---

### Q5. 有一种类型，方法签名写着 `&self`，却能改自己内部的值。这怎么可能？它靠什么保证安全？

这叫**内部可变性**（interior mutability）。

**怎么可能**：底层载体是 `UnsafeCell<T>`，标准库称它为
"The core primitive for interior mutability in Rust"（`core/src/cell.rs:2141`）。
它是**唯一**被语言承认可以"从共享引用得到内部可变指针"的类型 ——
文档明说这是获得 `*mut T` 的 "only valid way"（`:2226`）。

整条链在源码里看得很清楚：

```text
RefCell<T> { borrow: Cell<BorrowCounter>, value: UnsafeCell<T> }   // cell.rs:849
Cell<T>    { value: UnsafeCell<T> }                                 // cell.rs:312
UnsafeCell<T> { value: T }                                          // cell.rs:2323
```

`Cell` / `RefCell` / `Mutex` / `AtomicUsize` 全部建立在 `UnsafeCell` 之上。
它是这一族类型的共同地基。

**靠什么保证安全**：每种包装类型用**不同**的手段维护同一条不变量，
而共同点是 —— 可变性的**证明义务从编译器转移给了这个类型自己**：

| 类型 | 保证手段 | 冲突时 |
|------|---------|-------|
| `Cell<T>` | 只提供整值 `get`/`set`，**从不交出内部引用** → 无别名可言（`get` 要求 `T: Copy`，`cell.rs:536`） | 不可能冲突 |
| `RefCell<T>` | 交出引用，但用 `BorrowCounter` 在运行期计数把关 | **panic** |
| `Mutex<T>` | 用操作系统 / 原子操作加锁 | **阻塞**等待 |

`Cell` 的策略特别值得注意：它通过**根本不交出引用**来回避别名问题，
所以它连运行期检查都不需要。这是"把问题设计掉"而不是"把问题检查掉"。

再强调一次：这不是"绕过"借用规则。规则一字未改，
只是从"编译器静态证明"换成了"这个类型在运行期维护"。

`UnsafeCell` 是 C-19（Aliasing）的主角，US5 会展开它与 Stacked/Tree Borrows 的关系。

- 完整论述：[`concept.md` § C-03](./concept.md)
- 源码：`core/src/cell.rs:2141`（core primitive 原文）、`:2226`（only valid way）、
  `:2323`（`UnsafeCell` 定义）、`:312`（`Cell`）、`:849`（`RefCell` 字段）、`:536`（`impl<T: Copy> Cell<T>`）
- 断言：`tests/c03_borrow.rs::refcell_allows_mutation_through_shared_reference`
  （两个 `&SharedCounter` 同时存活，仍能改内部值）

---

## C-04 Lifetime（生命周期）

### Q1. 生命周期标注会生成任何运行期代码吗？如果不会，它到底在约束谁？

**不会生成任何运行期代码。** 它是纯编译期的，借用检查完成之后被**完全擦除**。

它约束的是**借用检查器的推导**，而不是任何值的寿命。
标注是一句**陈述**：告诉检查器"这个引用的有效期不超过那个数据的有效期"。
加标注**不会让任何数据多活一纳秒**；标注错了只会导致编译失败。

MIR 给出了本模块最强的一条证据（OBSERVATIONS「C-04：生命周期在 MIR 之前已被完全擦除」）：

```text
fn elided(_1: &str) -> &str {          fn annotated(_1: &str) -> &str {
    bb0: {                                 bb0: {
        _0 = copy _1;                          _0 = copy _1;
        return;                                return;
    }                                      }
}                                      }
```

两个函数体**逐字符相同**。但比"相同"更值得注意的是：
`annotated` 的源码签名是 `fn annotated<'a>(s: &'a str) -> &'a str`，
而 MIR 打印出来是 `fn annotated(_1: &str) -> &str` —— **`'a` 消失了**。

所以"擦除"的准确含义不是"生成了相同的代码"，而是**"到这一层它已经不存在了"**。

- 完整论述：[`concept.md` § C-04](./concept.md)
- 断言：`tests/c04_lifetime.rs::lifetime_annotations_are_compile_time_only`
- IR 观察：`OBSERVATIONS.md`「C-04：生命周期在 MIR 之前已被完全擦除」

---

### Q2. 大部分函数不用写标注也能编译。省略规则在什么情况下**不够用**？

先把三条省略规则（lifetime elision）列清楚：

1. 每个省略的**入参**生命周期各得一个独立的生命周期参数；
2. 若**恰好有一个**入参生命周期，它被赋给所有省略的**输出**生命周期；
3. 若有 `&self` 或 `&mut self`，`self` 的生命周期被赋给所有省略的输出。

**不够用的情况** = 返回类型含引用，而规则 2、3 都不适用：

- **有 ≥2 个入参引用，且没有 `&self`** → `E0106`。
  典型：`fn longest(a: &str, b: &str) -> &str`。
- **结构体 / 枚举定义持有引用** —— 定义处**没有**省略规则，必须显式写：
  `struct Excerpt<'a> { part: &'a str }`。

够用的情况（对照）：`fn first_word(s: &str) -> &str` 只有一个入参引用，
规则 2 直接给出答案，完整形式是 `fn first_word<'a>(s: &'a str) -> &'a str`。

**"不够用"不等于编译器无能** —— 见 Q3。

- 完整论述：[`concept.md` § C-04](./concept.md)
- 断言：`tests/c04_lifetime.rs::elision_suffices_for_a_single_input_reference`
- 省略规则本身属 **reference-fallback**：它是编译器的推导算法，写在 Rust Reference，无库代码承载

---

### Q3. `fn longest(a: &str, b: &str) -> &str` 编译不过。编译器缺的是哪一条信息？

缺的是：**返回值借自 `a` 还是 `b`**。

诊断原文把这件事说得比任何教材都直接（`compile_fail/c04_missing_lifetime.rs` → **E0106**）：

```text
error[E0106]: missing lifetime specifier
9 | pub fn longest(a: &str, b: &str) -> &str {
  |                   ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the
          signature does not say whether it is borrowed from `a` or `b`
```

注意插入符 `^` 指向**返回类型**，而两个入参用 `----` 标出 ——
诊断在说"候选来源有两个，落点在这里"。

**编译器看得出**返回值来自两者之一，但它**拒绝替作者选**。
这是设计选择而非能力不足：选错的后果是悬垂引用，
而两个候选之间不存在任何可靠依据能分辨该选哪个。

修复：`fn longest<'a>(a: &'a str, b: &'a str) -> &'a str`，
含义是"返回值的有效期不超过 `a` 与 `b` 中**较短**的那个"。

> **一个容易被忽略的点**：编译器的 `help` 给的是**能通过检查**的最简建议，
> 不一定是最佳设计。它建议把两个入参都标成 `'a`，这恰好符合 `longest` 的意图；
> 但若某个函数实际上只可能返回 `a`，那么
> `fn f<'a>(a: &'a str, b: &str) -> &'a str` 约束更松、对调用方更好用。
> 别把 `help` 当成标准答案。

- 完整论述：[`concept.md` § C-04](./concept.md)
- 反面：`compile_fail/c04_missing_lifetime.rs` → **E0106**
- 诊断抄录：`OBSERVATIONS.md` 的 `c04_missing_lifetime` 记录块
- 断言：`tests/c04_lifetime.rs::longest_returns_the_longer_input`（两次调用对调参数顺序）

---

### Q4. 一个结构体持有引用时，标注表达的约束是什么？违反了会怎样？

`struct Excerpt<'a> { part: &'a str }` 中的 `'a` 表达的约束是：

> `Excerpt` 的**实例**不得比 `part` 所指向的数据活得更久。

违反 → 编译失败 **`E0597`**（"borrowed value does not live long enough"）。
样本 `compile_fail/c04_dangling_ref.rs`：内层的 `String` 在块结束时销毁，
而 `excerpt` 在块外仍被使用。

没有这个标注，编译器**根本无从判断** `part` 指向的数据什么时候失效 ——
所以结构体定义处**必须**写，这里没有省略规则可依赖（见 Q2）。

> **一个反直觉的额外要点**：在**方法**返回引用时，
> 省略规则 3 会把返回值绑定到 `&self`，而这**比需要的更严格**。
>
> `Excerpt::part` 因此显式写了 `-> &'a str` 而不是省略形式：
> 返回的切片其实只依赖被引用的**原始数据**，不依赖 `Excerpt` 本身活多久。
> 显式标注在这里是为了**放宽**约束，而不是增加约束。
>
> 证据：`tests/c04_lifetime.rs::part_outlives_the_excerpt_itself` ——
> `excerpt` 已经离开作用域，`part` 仍然有效。若签名是省略形式，该测试无法编译。

- 完整论述：[`concept.md` § C-04](./concept.md)
- 反面：`compile_fail/c04_dangling_ref.rs` → **E0597**
- 断言：`tests/c04_lifetime.rs::struct_can_hold_a_reference_with_an_explicit_lifetime`、
  `::part_outlives_the_excerpt_itself`

---

### Q5. 有一种类型参数不占任何运行期空间，却能影响类型检查。它是什么？为什么需要它？

是 **`PhantomData<T>`**（`core/src/marker.rs:811`）。

定义只有一行，且**没有字段**：

```rust
pub struct PhantomData<T: PointeeSized>;
```

它是一个**单元结构体**（unit struct）—— 运行期什么都没有。
它不"存了个假的 `T`"，`T` 只出现在类型签名里供类型检查器使用。
大小为 0 是标准库**文档化的保证**（`core/src/marker.rs:805`），
不是实现巧合，因此**可以**拿来做稳定断言。

**为什么需要它** —— 这一问有个很具体的答案：
Rust 要求结构体的每个类型参数都被**实际使用**，否则报错。已实测：

```rust
pub struct Unused<T> { v: u32 }
// error[E0392]: type parameter `T` is never used
```

当你想在**类型层面**携带一个参数、但运行期不需要存储它时，
`PhantomData` 就是那个"使用"它的零成本占位。

它的三类典型用途：

1. **类型级区分**：`Tagged<Ingress>` 与 `Tagged<Egress>` 运行期逐字节相同，
   却不能互相赋值（`compile_fail/c04_phantom_type_mismatch.rs` → **E0308**）。
2. **携带生命周期**：裸指针包装器用 `PhantomData<&'a T>` 表达"我借用了这块数据"，
   让借用检查器能管住一个本来它看不见的关系。
3. **影响自动 trait 与 drop check**：`PhantomData<T>` 让类型在
   `Send`/`Sync` 推导和析构检查中表现得像"拥有一个 `T`"。

**它证明的核心事实**：**类型层面的区分与运行期的空间占用完全无关。**
这是"零成本抽象"最纯粹的形态，也是 Aya 区分 map 类型
（`HashMap<K, V>` vs `PerfEventArray`）所用的手法。

注意结论的**边界**：零大小这条**不能**推广到所有 ZST。
`PhantomData` 的对齐是 1，恰好不影响布局；换成高对齐的 ZST（如 `[u64; 0]`）结论会变。

- 完整论述：[`concept.md` § C-04](./concept.md)
- 源码：`core/src/marker.rs:811`（定义）、`:805`（零大小的文档保证）
- 断言：`tests/c04_lifetime.rs::phantom_data_is_zero_sized`、
  `::phantom_tagging_costs_no_space`、
  `::differently_tagged_values_share_the_same_runtime_representation`
- 反面：`compile_fail/c04_phantom_type_mismatch.rs` → **E0308**

---

## 一句话汇总

| C-ID | 最容易错的那一点 |
|------|---------------|
| C-02 | 移动/复制的分界线是 **`Copy`**，不是 `Clone`（`String` 有 `Clone` 无 `Copy`，赋值仍是移动） |
| C-02 | 让值永不销毁是**可以**的（`mem::forget` 等），结果是**泄漏而非 UB**，所以它不需要 `unsafe` |
| C-03 | 借用到**最后一次使用**结束（NLL），不是到作用域末尾；冲突条件是**区间重叠** |
| C-03 | `RefCell` 没有放宽规则，只是把执行者从编译器换成了一个 `isize` 计数器 |
| C-04 | 标注**约束的是检查器**，不是值的寿命；MIR 里 `'a` 已经不存在 |
| C-04 | `PhantomData` 存在的直接理由是"未使用的类型参数会报 **E0392**" |
