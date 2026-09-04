# Feynman: Module 1 — 所有权、移动与生命周期

**Capabilities covered**: C-01, C-02, C-03, C-04

---

## 1. 用自己的话解释

面向"没学过 Rust 但懂 C"的同事。

你写 C 的时候，每次 `malloc` 都得自己记着在哪 `free`。这件事之所以难，
不是因为 `free` 难写，而是因为"**谁负责释放**"这个信息**不在代码里** ——
它在你脑子里、在注释里、在文档里，编译器完全不知道。
于是有了三类经典 bug：忘了释放（泄漏）、释放两次（double free）、
释放后还用（use-after-free）。

Rust 的做法是把这个信息**写进类型系统**，让编译器知道。具体是四件事：

**第一，每个值在任何时刻都有唯一的一个"负责人"。**
在 C 里，一块内存可以被五个指针指着，谁都能 `free` 它。
在 Rust 里，一个值只归一个变量所有，别人只能"看"不能"负责"。
负责人所在的那对花括号结束时，释放代码自动执行 —— 这段代码是编译器**替你生成**的，
源码里找不到，就像 C 编译器替你生成函数序言（prologue）一样。
关键在于：释放的**位置**是编译期算出来的，不是运行时判断的。
所以这里既没有 GC 那样的运行期开销，也没有引用计数的原子操作 —— 什么都没有，
就是在正确的那一行插了一次调用。

**第二，把值赋给另一个变量，默认是把"负责权"交出去，而不是复制一份。**
`b = a;` 之后 `a` 就不能再用了 —— 不是因为 `a` 的内存被清空了
（那块栈空间原样躺在那儿），而是因为编译器记下了"`a` 已经不负责了"，
此后你读它就是编译错误，花括号结束时也不会对它执行释放。
所以"移动"这个词有点误导：**被搬走的不是数据，是那份责任**。
数据在机器层面可能拷贝了一次，也可能一次都没拷 —— 那是优化器的事。

对于像 `int` 这种"复制一份完全等价、也不需要释放"的类型，赋值就是普通复制，
原变量照用。语言用一个标记（`Copy`）来区分这两类。
而这个标记跟"有释放行为"是**互斥**的 —— 想想为什么：
如果一个值既能随便复制、又有释放行为，那五份副本各释放一次，
就是你在 C 里最怕的那个 double free。

**第三，可以"借"值来用，但借的规则很死。**
借用就是 C 里的取地址 `&x`，只不过 Rust 规定：
同一时刻，要么很多人一起**只读**，要么恰好一个人可以**改**，不能混。
这条规则防的就是 C 里那个经典陷阱：你拿了 `vector` 里某个元素的指针，
然后往 `vector` 里 push 了一个东西，触发了 realloc，
你手上那个指针立刻变成野指针。Rust 里这段代码根本编译不过。

这里有个很值得注意的点：编译器做这个检查时**不运行你的程序**，
它靠静态分析证明"不存在冲突"。这意味着它必然是**保守**的 ——
有些逻辑上完全安全的写法，它证明不出来，于是拒绝。
被拒绝不等于你的代码有错，可能只是它推不出来。
（这个性质待会儿在 eBPF 里会以完全一样的形式再出现一次。）

如果你确实需要"多人共享还要能改"，语言提供了一个盒子（`RefCell`），
它把同一条规则的检查**挪到运行期**：内部用一个整数记录当前借用状态，
违规就崩溃退出。规则一字没改，只是发现问题的时机从编译期变成了运行期 ——
换来了灵活性，代价是错误跑到线上才暴露。

**第四，编译器需要知道"借来的东西能用多久"。**
考虑一个 C 函数：`char* pick(char* a, char* b);`
它返回的指针指向 `a` 还是 `b`？函数签名**没说**。
调用方只能读文档、读实现，或者猜。猜错了就是 use-after-free。

Rust 要求你在签名里说清楚。它的写法是给引用加个标签
（写成 `'a`，读作 "tick a"），意思是"返回的这个引用，
其有效期不超过带同样标签的那些参数"。
标签不产生任何机器码 —— 它编译完就消失了，纯粹是给检查器看的一句声明。
大多数情况下编译器能自己推出来（只有一个参数是引用时，答案没有歧义），
所以你平时不用写。只有**歧义真实存在**时它才要求你说明，
而这时它选择"报错"而不是"猜一个" —— 因为猜错的代价是野指针。

**这四件事合起来是一句话**：C 把"谁负责、能用多久"放在程序员脑子里，
Rust 把它写进类型，让编译器检查。代价是你得把这些事说清楚，
收益是那三类 bug 在编译期就没了，而且运行期不多花一分钱。

---

## 2. 最小示例

### C-01 Ownership — 销毁时刻由静态作用域决定，顺序为声明的逆序

完整文件：[`examples/c01_ownership.rs`](../experiments/m1-ownership/examples/c01_ownership.rs)

```rust
let log = DropLog::new();
{
    let _first = Noisy::new("first", &log);
    let _second = Noisy::new("second", &log);
    let _third = Noisy::new("third", &log);
}
// log.events() == ["third", "second", "first"]
```

### C-02 Move semantics — 移动转移销毁责任，总次数不变

完整文件：[`examples/c02_move.rs`](../experiments/m1-ownership/examples/c02_move.rs)

```rust
let drops = Cell::new(0);
{
    let first = Tracked::new(1, &drops);
    let second = first;   // move
    let _third = second;  // move again
    // drops.get() == 0
}
// drops.get() == 1 —— 移动了两次，仍只销毁一次
```

### C-03 Borrowing — 同一规则的两种执行时机

完整文件：[`examples/c03_borrow.rs`](../experiments/m1-ownership/examples/c03_borrow.rs)

```rust
// 编译期：compile_fail/c03_two_mut_borrows.rs → E0499
let first = &mut c.value;
let second = &mut c.value;
*second += 1;
*first += 1;

// 运行期：同一条规则，改由 RefCell 检查
let _a = cell.borrow_mut();
let _b = cell.borrow_mut(); // panic: RefCell already borrowed
```

### C-04 Lifetime — 省略规则失效处，与零成本的类型级区分

完整文件：[`examples/c04_lifetime.rs`](../experiments/m1-ownership/examples/c04_lifetime.rs)

```rust
// 两个入参引用 → 省略规则不适用，必须标注
pub fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

// PhantomData：类型层面可区分，运行期零占用
assert_eq!(size_of::<PhantomData<Ingress>>(), 0);
assert_eq!(size_of::<Tagged<Ingress>>(), size_of::<u32>());
```

---

## 3. 底层机制

每条论断都带实验断言名或源码符号作为依据。

### 销毁代码由编译器生成，源码中不存在

`Drop` trait 带 `#[lang = "drop"]`（`core/src/ops/drop.rs:206`），
是**语言项**而非普通 trait —— 编译器对它有内建认知，
在代码生成阶段合成一段递归销毁字段的代码（drop glue）。

MIR 层可以直接看到调用点：OBSERVATIONS 的
「C-01：drop glue 的插入位置与顺序」记录块显示三个 `Noisy`
对应三条 `drop` terminator，顺序为 `_35` → `_33` → `_31`（逆序），
而源码里没有任何一行写了这三次销毁。

drop glue 必须由编译器生成的**根本理由**也在同一个记录块里：
清理块构成一条链（`bb91` → `bb92` → `bb93`），
且每个构造进度点的 unwind 入口**不同** ——
构造 `third` 时的 unwind 目标 `bb91` 不销毁 `_35`，
因为那一刻它还没构造出来。编译器为每个中间时刻算出了确切的清理集合。
这正是 C 里要用 `goto cleanup` 手工维护、C++ 里要靠异常安全规则约束的那件事。

`Drop::drop` 不能手动调用（`compile_fail/c01_manual_drop_call.rs` → **E0040**），
理由是编译器已在作用域末尾安排了一次销毁，手动再调一次即第二次。

### 移动是簿记，不是搬运

OBSERVATIONS 的「C-02：移动后的源 local 没有 drop」记录块给出了最干净的证据：
移动源 local 的 `drop` terminator 数量为 **0**，目标 local 为 2
（正常路径 + unwind 路径各一）。移动本身在 MIR 里只是一条 `_22 = move _20;`。

对应的稳定断言是 `tests/c02_move.rs::move_does_not_duplicate_drops`：
链式移动三次后销毁计数为 1。
对照组 `tests/c02_move.rs::clone_creates_an_independent_value` 为 2 ——
`Clone` 真的造了第二个值，于是有第二份责任。

`Copy` 与 `Drop` 互斥的理由由标准库自己写明：
`Copy` 文档中的 "any type implementing `Drop` can't be `Copy`"
（`core/src/marker.rs:419`），实施于
`compile_fail/c02_copy_with_drop.rs` → **E0184**。

`mem::forget` 的实现只有一行 `ManuallyDrop::new(t)`
（`core/src/mem/mod.rs:189`），签名**无 `unsafe`**。
`tests/c02_move.rs::forget_suppresses_drop` 显示销毁计数为 0 且程序照常运行 ——
泄漏不违反内存安全。

`replace`（`core/src/mem/mod.rs:953`）无额外约束，
`take`（`:886`）要求 `T: Default`：差别在于替补值由谁提供。
两者的共同不变量由 `tests/c02_move.rs::neither_replace_nor_take_leaves_an_invalid_state`
断言 —— 原位置**从不**留下无效状态。

### 借用规则有两个执行时机，规则本身相同

运行期版本的**全部状态**是一个有符号整数：
`type BorrowCounter = isize`（`core/src/cell.rs:945`）。
编码方案见同文件 `:946`（`UNUSED = 0`）、`:949`（`is_writing`，负数）、
`:954`（`is_reading`，正数）。违规走 `:924` 的 panic 路径。

编译期版本由 `rustc_borrowck` 实现，属编译器而非 `library/`，
因此 source-refs 中按 §B2 记为 **reference-fallback**。
它的产出是错误码：`compile_fail/c03_two_mut_borrows.rs` → **E0499**，
`compile_fail/c03_mut_while_shared.rs` → **E0502**。

两者是同一条规则而非两条，依据是
`tests/c03_borrow.rs::sequential_borrows_are_legal_in_both_regimes`：
合法的顺序访问在两种时机下都通过，说明 `RefCell` 没有放宽规则。

借用的存活区间到**最后一次使用**为止（NLL），
依据是 `tests/c03_borrow.rs::nll_ends_borrow_at_last_use` ——
它**能编译**这一事实本身即为证明：若借用持续到作用域末尾，该测试无法编译。

### 生命周期标注在借用检查之后被完全擦除

OBSERVATIONS 的「C-04：生命周期在 MIR 之前已被完全擦除」记录块显示：
`elided` 与 `annotated` 的 MIR 函数体逐字符相同，
且 `annotated` 的 MIR 签名打印为 `fn annotated(_1: &str) -> &str` ——
源码里的 `'a` 已不存在。

擦除发生在检查**之后**，顺序不可颠倒理解：
`compile_fail/c04_dangling_ref.rs` → **E0597** 就是标注起作用的证据。
省略规则失效时编译器拒绝推断：
`compile_fail/c04_missing_lifetime.rs` → **E0106**，
其 `help` 原文为 "does not say whether it is borrowed from `a` or `b`"
（抄录于 OBSERVATIONS 的 `c04_missing_lifetime` 诊断块）。

`PhantomData<T>` 的定义是无字段的单元结构体
（`pub struct PhantomData<T: PointeeSized>;`，`core/src/marker.rs:811`），
零大小是**文档化保证**（`:805`）而非实现巧合，故可作稳定断言：
`tests/c04_lifetime.rs::phantom_data_is_zero_sized` 与
`tests/c04_lifetime.rs::phantom_tagging_costs_no_space`。

类型级区分与运行期占用互相独立，两侧各有证据：
不可互换性由 `compile_fail/c04_phantom_type_mismatch.rs` → **E0308** 断言，
运行期无差别由
`tests/c04_lifetime.rs::differently_tagged_values_share_the_same_runtime_representation` 断言。

---

## 4. 常见误区

写的是我在做本模块时**真的想错过**的地方。

### C-01

- **误解**：作用域结束时 Rust 调用的是我写的那个 `drop()` 方法。
  → **实际**：调用的是编译器合成的 drop glue，它递归销毁所有字段；
  `Drop::drop` 只是其中**可选**的一环，绝大多数类型根本没实现它。
  → **证据**：`tests/c01_ownership.rs::drop_glue_handles_fields_without_drop_impl` ——
  `DropLog` 内部的 `RefCell<Vec<_>>` 没有手写 `Drop` 却被正确清理。

- **误解**：panic 路径上的销毁大概是运行时库统一处理的。
  → **实际**：编译器为**每个构造进度点**生成了不同的清理入口，编排成一条链。
  → **证据**：OBSERVATIONS 的「C-01：drop glue 的插入位置与顺序」——
  `bb91` 不销毁 `_35`，因为那一刻 `_35` 尚未构造完成。
  我原以为清理集合只有一套，实际上有 N 套（N = 构造点数量）。

### C-02

- **误解**：移动之后原变量那块内存被回收或清零了，所以不能再读。
  → **实际**：栈空间原样留着，什么都没变。不能读**纯粹**是编译器的簿记决定。
  → **证据**：OBSERVATIONS 的「C-02：移动后的源 local 没有 drop」——
  MIR 里源 local 仍占有 slot，变化只是不再有 `drop` terminator 指向它。

- **误解**：`mem::forget` 会造成 UB，所以它应该是 `unsafe` 的。
  → **实际**：它是安全函数。泄漏资源不违反内存安全 —— 没人会读到无效数据。
  → **证据**：`tests/c02_move.rs::forget_suppresses_drop` 中销毁计数为 0
  且程序继续正常运行；`core/src/mem/mod.rs:189` 的签名无 `unsafe`。
  我把"资源没被正确处理"和"内存不安全"混成了一件事，它们是两个范畴。

### C-03

- **误解**：借用一直持续到变量的作用域结束。
  → **实际**：NLL 让借用在**最后一次使用**后即结束，与绑定是否还在作用域内无关。
  → **证据**：`tests/c03_borrow.rs::nll_ends_borrow_at_last_use` 能编译。
  按我原来的理解那段代码不可能通过。

- **误解**：编译器拒绝我的代码，意味着我的代码真的有问题。
  → **实际**：借用检查器是**保守**的。它拒绝的是"它无法证明安全"的代码，
  而非"它证明了不安全"的代码。
  → **证据**：OBSERVATIONS 的 `c03_two_mut_borrows` 诊断块 ——
  诊断标出的三个位置（两个借用产生点 + 第一个借用的最后使用点）
  正是它判定"区间重叠"所需的全部信息；一旦区间不重叠，同样的两个 `&mut` 就合法。
  这个性质在 eBPF verifier 上会以完全相同的形式再遇到一次。

### C-04

- **误解**：`'a` 决定了引用能活多久，所以标注写长一点数据就能多活一会儿。
  → **实际**：标注只**描述**已经存在的约束关系。加标注不会让任何数据多活一纳秒；
  标注错了只会导致编译失败。
  → **证据**：`tests/c04_lifetime.rs::lifetime_annotations_are_compile_time_only`
  显示带标注与省略标注的等价函数返回完全相同的结果；
  OBSERVATIONS 的 C-04 IR 观察进一步显示 MIR 里 `'a` 已不存在。

- **误解**：`PhantomData<T>` 内部存了一个 `T`（或者至少存了个占位的什么东西）。
  → **实际**：它是**无字段**的单元结构体，运行期什么都没有。
  → **证据**：`core/src/marker.rs:811` 的定义只有一行且无字段；
  `tests/c04_lifetime.rs::phantom_data_is_zero_sized` 断言
  `size_of::<PhantomData<[u8; 4096]>>() == 0` —— 与 `T` 的大小完全无关。

---

## 5. 验证性问题

### Q1：把 `Copy` 加到一个实现了 `Drop` 的类型上，编译器报什么错？为什么这个组合必须被禁止，而不是"允许但不推荐"？

**E0184**。理由不是风格而是 double free：`Copy` 意味着赋值即按位复制且原值仍有效，
于是 N 份副本在各自的作用域末尾各触发一次 drop glue，同一份资源被释放 N 次。
标准库把这个理由写在 `Copy` 的文档里（`core/src/marker.rs:419`：
"any type implementing `Drop` can't be `Copy`"）。

**依据**：(b) `core/src/marker.rs:419` + `compile_fail/c02_copy_with_drop.rs`。

### Q2：一个值被连续移动三次，销毁几次？如果答案是 1，那么 MIR 里应该看到几条 `drop` terminator 指向最初那个 local？

销毁 1 次。指向最初那个 local 的 `drop` terminator 数量是 **0** ——
这才是"1 次"的机制解释：不是"三次移动里只有一次生效"，
而是编译器**根本不为已移出的 local 生成 drop**。
目标 local 有 2 条（正常路径 + unwind 路径），但那是两条路径各一份，不是销毁两次。

**依据**：(a) `tests/c02_move.rs::move_does_not_duplicate_drops`
+ (c) OBSERVATIONS「C-02：移动后的源 local 没有 drop」的计数校验。

### Q3：`RefCell` 用什么数据结构记录借用状态？把它和编译期借用检查器所需的分析规模对比，这笔交换换来了什么、付出了什么？

**一个 `isize`**（`type BorrowCounter = isize`，`core/src/cell.rs:945`）：
`0` = 无借用，正数 = 不可变借用计数，负数 = 存在可变借用。
运行期检查就是对它做加减和符号判断。
编译期版本要做全函数控制流图上的借用区间分析。

换来的是**能表达编译器证明不了的共享模式**（多处持有、都能改）；
付出的是错误从"编译不过"变成"线上 panic"。
规则本身一字未改 —— 这一点有专门的断言。

**依据**：(b) `core/src/cell.rs:945` / `:946` / `:949` / `:954`
+ (a) `tests/c03_borrow.rs::sequential_borrows_are_legal_in_both_regimes`。

### Q4：一段代码里有两个 `&mut` 指向同一个字段，什么情况下编译器**不**报错？这说明借用冲突的成立条件是什么？

两个借用的活跃区间**不重叠**时不报错 —— 例如第一个借用在第二个产生之前
就已完成最后一次使用（NLL）。所以冲突的成立条件不是"存在两个 `&mut` 绑定"，
而是"两个借用同时**存活**"，而存活的终点是最后一次使用而非作用域末尾。

`compile_fail/c03_two_mut_borrows.rs` 之所以刻意把 `*first += 1`
放在 `*second += 1` **之后**，就是为了把两个区间强行交错；
诊断也正是用三个标注（两个产生点 + "first borrow later used here"）
画出这个判定。

**依据**：(a) `tests/c03_borrow.rs::nll_ends_borrow_at_last_use`
+ (c) OBSERVATIONS 的 `c03_two_mut_borrows` 诊断块。

### Q5：如果 `fn longest(a: &str, b: &str) -> &str` 编译不过，为什么 `fn first_word(s: &str) -> &str` 就可以？编译器在前者上"猜一个"会有什么后果？

省略规则第 2 条只在**恰好一个**入参生命周期时生效：
`first_word` 满足，返回值直接绑定到 `s`；`longest` 有两个候选，规则 2 不适用，
规则 3 也因无 `&self` 不适用，于是报 **E0106**。

猜错的后果是悬垂引用：若编译器选了 `a` 而实际返回的是 `b`，
调用方就可能在 `b` 已失效之后仍持有一个被认为有效的引用。
两个候选之间不存在任何可靠依据可分辨，所以"报错"是设计选择而非能力不足。
诊断原文说得最直接："does not say whether it is borrowed from `a` or `b`"。

**依据**：(c) OBSERVATIONS 的 `c04_missing_lifetime` 诊断块
+ (a) `tests/c04_lifetime.rs::elision_suffices_for_a_single_input_reference`。

### Q6：`Tagged<Ingress>` 与 `Tagged<Egress>` 不能互相赋值，但它们的 `size_of` 相同。这两件事各由什么证据支持？为什么必须分别验证？

不可互换性是**编译期**事实，由 `compile_fail/c04_phantom_type_mismatch.rs`
→ **E0308** 断言。运行期表示相同是**运行期**事实，由
`tests/c04_lifetime.rs::differently_tagged_values_share_the_same_runtime_representation`
和 `phantom_tagging_costs_no_space` 断言。

必须分别验证，因为二者是**独立**的命题，谁也推不出谁：
运行期相同不意味着类型可互换（正是本例），
类型不可互换也不意味着运行期一定有差别（若 `PhantomData` 占空间，结论就变了）。
"零成本抽象"这句话的完整含义恰好就是这两条同时成立。

**依据**：(a) 上述两个测试 + (b) `core/src/marker.rs:805`（零大小的文档保证）。

### Q7：如果去掉 `Noisy<'log>` 上的 `'log` 标注（假设能编译），把 `Noisy` 声明在 `DropLog` 之前会发生什么？这解释了逆序销毁的必要性吗？

`Noisy::drop` 里会访问已经被销毁的 `DropLog` —— 即 use-after-free。
`'log` 标注表达的正是"日志必须活得比记录它的值更久"，
编译器据此拒绝把 `Noisy` 声明在 `DropLog` 之前。

这确实解释了逆序销毁：后声明的值可能借用了先声明的值，
只有逆序销毁才能保证被引用者始终活得更久。
所以"逆序"不是任意约定，而是被生命周期约束**逼出来**的唯一可行顺序。

**依据**：(c) OBSERVATIONS「C-01 / c01_ownership」记录块的机制解释
+ (a) `tests/c01_ownership.rs::drop_order_is_reverse_of_declaration`。

### Q8：本模块四个能力的 `ub_verdict` 都记 `n/a` 而不是 `clean`。这个区别为什么重要？

`clean` 的含义是"UB 检测工具实际运行过，且未报告 UB"。
本模块不含任何 `unsafe` 代码，因此没有运行 Miri ——
"没跑工具"只能记 `n/a`。把未验证的东西写成 `clean`，
会让后续模块误以为这里已经有 UB 保证，从而在此基础上建立错误的推论。

这条纪律来自 FR-019，`rf_harness::miri` 在实现上强制了它：
`MiriOutcome::reported_ub()` 与 `stderr_contains()` 在 Miri 被跳过时**直接 panic**，
使"跳过"无法被静默当成"干净"。
若它们改为返回 `false`，那么 `assert!(!out.reported_ub())`
在 Miri 缺席时会照常通过，验收就变成了自欺。
只有 `skipped()` 与 `skip_reason()` 在跳过状态下可安全查询 ——
它们是给出 `n/a` 的正当路径。

**依据**：(c) OBSERVATIONS 的「UB 判定记录」表
+ (a) `harness/tests/harness_selfcheck.rs::reported_ub_panics_when_miri_skipped`、
`::stderr_contains_panics_when_miri_skipped`、
`::skip_reason_is_queryable_without_panic`。

---

## 检验结果

| # | 检验项 | 合格标准（可判定） | 状态 |
|---|-------|-----------------|------|
| 1 | 自述概念 | 第 1 节存在，且不含未解释的 Rust 术语 | **pass** |
| 2 | 最小示例 | 每个 covered capability 各有一段 ≤15 行代码且链接到 `examples/` | **pass** |
| 3 | 底层机制 | 第 3 节每条论断都带断言名或源码符号 | **pass** |
| 4 | 常见误区 | 每个 covered capability ≥1 条，且三段式齐备 | **pass** |
| 5 | 回答问题 | ≥5 个问题，且每题的回答指向一条断言 / 一处源码引用 / 一个观测块 | **pass** |

### 逐项判定依据

1. **自述概念** — 第 1 节以 C 程序员为受众，全篇未使用未解释的 Rust 术语：
   `Copy` / `Drop` / `RefCell` / `'a` 首次出现处均以"标记""盒子""标签"等
   已解释的说法引入，或当场给出解释。术语 `borrow checker`、`NLL`、`drop glue`
   在第 1 节被刻意回避，留到第 3 节（面向已懂 Rust 的读者）才使用。
2. **最小示例** — 四段代码分别为 7 / 8 / 9 / 8 行，均 ≤15 行，
   且各自链接到对应的 `examples/` 文件。
3. **底层机制** — 该节每条论断都带
   `tests/…::<断言名>`、`core/src/…:<行号>`、`compile_fail/… → E0xxx`
   或 OBSERVATIONS 记录块标题之一，合计 26 处依据标记；
   规则 C2 点名的三个失败模式措辞（表示传闻或泛化的那三个词）一处未用。
4. **常见误区** — C-01 两条、C-02 两条、C-03 两条、C-04 两条，
   共 8 条，均为"误解 → 实际 → 证据"三段式，且证据指向具体断言或记录块。
5. **回答问题** — 8 个问题（≥5），每题末尾标注 (a)/(b)/(c) 类依据，
   且问题均为可暴露理解缺口的形式（Q2 追问 MIR 层计数、
   Q3 要求做代价对比、Q4 追问"什么情况下**不**报错"、
   Q6 追问"为什么必须分别验证"、Q8 追问纪律的理由），
   而非第 1 节的重复。

**五项全部 pass** → 本模块 `feynman_status = passed`，
C-01…C-04 可从 `experiment-passed` 进入 `accepted`。
