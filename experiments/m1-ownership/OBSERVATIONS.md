# Module 1 —— OBSERVATIONS

<!-- 本文件里的一切都是 NON-ASSERTION：其差异不参与一致性判定（experiment-contract §C8.1）。
     稳定断言在 tests/，两者物理隔离（R-05）。 -->

**Story**: US1 | **Capabilities**: C-01…C-04

## 环境记录

| 字段 | 值 |
|------|-----|
| `rustc_stable` | 1.98.0 (88d9e12ae 2026-08-18) |
| `rustc_nightly` | 1.100.0-nightly (17fd5b8a3 2026-08-28) |
| `edition` | 2024 |
| `kernel` | 6.6.114.1-microsoft-standard-WSL2 |
| `arch` | x86_64 |
| `target` | x86_64-unknown-linux-gnu |
| `command` | `cargo test -p m1-ownership` / `cargo run -p m1-ownership --example <name>` |

> 基线：[../../acceptance/environment-baseline.md](../../acceptance/environment-baseline.md) —— 一致，无差异。

---

## 记录块

### C-01 / c01_ownership  [NON-ASSERTION]

命令：`cargo run -p m1-ownership --example c01_ownership`

输出：

```text
=== 1. 销毁时刻由静态作用域决定 ===
  进入内层作用域后，已销毁：[]
  离开内层作用域后，已销毁：["inner"]

=== 2. 同一作用域内，销毁顺序是声明顺序的逆序 ===
  声明顺序 first, second, third
  销毁顺序 ["third", "second", "first"]

=== 3. 所有权转移后，销毁发生在新所有者的作用域末尾 ===
  调用 consume 之前，已销毁：[]
  consume 返回之后，已销毁：["moved"]
  —— 注意：销毁发生在 consume 内部，而非本函数末尾

=== 4. 嵌套所有权：容器销毁时递归销毁其元素 ===
  Vec 尚在作用域内，已销毁：[]
  Vec 离开作用域后，已销毁：["elem0", "elem1", "elem2"]
  —— 这段递归销毁的代码是编译器生成的 drop glue，源码里并不存在
```

解释：

  为什么会这样：
  销毁时刻由**静态作用域**决定，编译器在编译期就知道每个值的作用域末尾在哪一行，
  于是把 drop glue 调用作为 MIR 的 `drop` terminator 插在那里（见下方 IR 观察，
  三个 `Noisy` 对应三个连续的 `drop` terminator）。
  逆序销毁不是任意约定：后声明的值可能借用了先声明的值 —— 本实验里 `Noisy<'log>`
  就借用了 `DropLog`，若正序销毁，日志会先失效，`Noisy::drop` 里访问的就是已死数据。
  编译器用 `'log` 这个约束反推出必须逆序，并据此拒绝把 `Noisy` 声明在 `DropLog` 之前。
  第 3 段里销毁发生在 `consume` 内部，是因为所有权转移把**销毁责任**一并转移了：
  值在新所有者的作用域末尾被销毁，`consume` 的函数体末尾就是那个位置。
  第 4 段的递归销毁没有任何一行 Rust 源码对应，它是编译器为 `Vec<Noisy>` 合成的：
  先逐元素调用元素的 drop glue，再释放缓冲区。

  这不能证明什么：
  **不能**证明 drop glue 里做了什么以外的事 —— 它只显示了"被调用了、以什么顺序"，
  没有显示 glue 内部的结构（那需要看 MIR，见下）。
  **不能**证明 `Vec` 元素的销毁顺序在语言层面被保证为下标升序；
  这里观察到的是当前实现的行为，标准库文档并未把它列为稳定契约
  （相对地，**局部变量**的逆序销毁是 Reference 明文规定的）。
  **不能**推出"所有值都在作用域末尾销毁"：被 `mem::forget` 的值、
  被移动走的值、`ManuallyDrop` 包裹的值都不适用（前两者见 C-02）。
  也**不能**由第 3 段推出"传参一定发生销毁"—— `consume` 恰好丢弃了参数，
  若它把参数返回出来，销毁点就又转移了。

架构相关性：可跨架构推广。销毁时刻与顺序由 MIR 层的控制流决定，
在 codegen 之前就已确定，与目标架构的寄存器、调用约定、栈布局均无关。
唯一与架构有关的是 drop glue 最终被编译成什么指令，而那不影响这里观察的顺序。

---

### C-02 / c02_move  [NON-ASSERTION]

命令：`cargo run -p m1-ownership --example c02_move`

输出：

```text
=== 1. Copy 类型：赋值后原值仍可用 ===
  a = Meters(42), b = Meters(42)  —— 两个独立的值

=== 2. 非 Copy 类型：移动后销毁责任转移，总销毁次数不变 ===
  移动完成，作用域尚未结束，销毁次数 = 0
  离开作用域，销毁次数 = 1

=== 3. Clone 产生独立的值，销毁次数随之增加 ===
  一次 duplicate 之后，销毁次数 = 2

=== 4. mem::forget 抑制销毁 ===
  forget 之后销毁次数 = 0
  —— forget 是**安全**函数：泄漏不违反内存安全

=== 5. replace / take：搬走值的同时不留下无效状态 ===
  replace 取出 "original"，原处现在是 "replacement"
  take 取出 "to-be-taken"，原处现在是 ""
  —— take 要求 T: Default，因为它必须自己找一个替补值放回去

=== 6. 传参：移动 vs 借用 ===
  borrow_label 返回 11，之后仍可用："hello world"
  consume_label 返回 11，之后 label 不可再用
```

解释：

  为什么会这样：
  第 2 段的销毁次数 = 1 是本模块**最关键的一个数**。移动在机器层面就是一次按位拷贝
  （而且常被优化掉），运行期行为与 `Copy` 完全相同；差别只在编译器的**簿记**：
  移动之后源位置被标记为"已移出"，作用域末尾不再对它插入 drop 调用。
  MIR 里可以直接看到这一点 —— 源 local 的 `drop` terminator 数量为 **0**（见 IR 观察）。
  所以"移动"不是一个动作，而是一次责任转移的记账；被搬走的不是数据，是销毁义务。
  第 3 段的 2 次销毁提供了对照：`Clone` 真的造了第二个值，于是有第二份销毁责任。
  第 4 段的 0 次销毁则是另一个方向：`mem::forget` 的实现只有一行
  `ManuallyDrop::new(t)`（`core/src/mem/mod.rs:189`），它取得所有权后既不销毁也不归还，
  于是那次销毁永远不会发生。它是**安全**函数，因为泄漏不违反内存安全 ——
  没人会读到无效数据。
  第 5 段里 `take` 之后原处是空串而非某种"无效状态"，这是 `T: Default` 约束的用途：
  必须有个合法的替补值可放。`replace` 不需要这个约束，因为替补值由调用方自带。

  这不能证明什么：
  **不能**证明移动"没有发生任何数据拷贝"。销毁计数只能证明销毁责任没被复制，
  它对是否发生 memcpy 完全不敏感 —— 那要看 codegen 后的机器码，
  且 `-C opt-level` 会改变结论。本次观察在 dev profile（未优化）下取得。
  **不能**由第 4 段推出"`forget` 一定造成内存泄漏"：本实验的 `Tracked`
  不持有堆内存，被 forget 掉的只是一次计数自增。若换成 `Vec`，泄漏才真会发生。
  **不能**推出"泄漏永远无害"—— 泄漏锁守卫、泄漏文件描述符都会造成实际问题，
  只是它们不属于**内存安全**问题。UB 与泄漏是两个范畴，这个区分在 US5 会反复出现。
  第 6 段也**不能**证明 `consume_label` 之后 `label` 不可用 ——
  那是编译期事实，由 `compile_fail/c01_use_after_move.rs`（E0382）断言，
  在这份运行期输出里根本无法体现。

架构相关性：可跨架构推广。销毁计数取决于 MIR 层插入的 `drop` terminator，
与架构无关。真正与架构有关的是"移动是否被编译成实际的 memcpy"——
而本条观察刻意**不**依赖那一点（见"这不能证明什么"）。

---

### C-03 / c03_borrow  [NON-ASSERTION]

命令：`cargo run -p m1-ownership --example c03_borrow`

输出：

```text
=== 1. 多个不可变借用可以共存 ===
  三个 &Counter 同时存活：10 10 10

=== 2. 可变借用是独占的，但 NLL 让它在最后一次使用后即结束 ===
  可变借用结束后可以再取不可变借用：5

=== 3. 借用不转移所有权 ===
  first_half(&data) = [1, 2, 3]
  data 仍然可用：[1, 2, 3, 4, 5, 6]

=== 4. RefCell：同一条规则，检查挪到运行期 ===
  通过 &self 修改内部值，当前值 = 7
  —— 注意 add 的签名是 &self，可变性由 RefCell 在运行期把关

=== 5. 运行期借用冲突的表现形式 ===

thread 'main' (91343) panicked at experiments/m1-ownership/src/c03.rs:76:34:
RefCell already borrowed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
  发生 panic —— 与编译期版本的错误码是同一条规则的两种后果
  （完整 panic 消息属非断言产物，其措辞随版本变化）
```

解释：

  为什么会这样：
  第 2 段能编译**本身**就是 NLL 的证明：`m` 这个绑定还在作用域里，
  但它的借用在最后一次使用后就结束了，所以紧接着取 `&c` 不冲突。
  借用的存活区间是"到最后一次使用为止"，不是"到作用域末尾"。
  第 5 段的 panic 来自 `RefCell` 的运行期检查。它的全部状态是**一个 `isize`**
  （`core/src/cell.rs:945` 的 `BorrowCounter`）：`0` = 无借用，正数 = 不可变借用计数，
  负数 = 存在可变借用。第一次 `borrow_mut()` 把它变成负数，第二次发现符号已为负，于是 panic。
  把它和编译期版本并排看，是本能力最有价值的对照：编译期版本要做全函数控制流分析
  （`rustc_borrowck`），运行期版本只需一次加减和一次符号判断。
  同一条规则、两种执行时机 —— 换来的是能表达编译器证明不了的共享模式，
  付出的是错误从"编译不过"变成"线上崩溃"。
  另外注意 panic 措辞是 `already borrowed` 而非 `already mutably borrowed`：
  `borrow_mut` 在发现**任何**既存借用时报前者，`borrow` 在发现既存**可变**借用时报后者。
  这里两次都是 `borrow_mut`，所以是前者。

  这不能证明什么：
  **不能**把 panic 措辞当作稳定契约。它是诊断文本，随版本漂移；
  `tests/c03_borrow.rs` 里的断言只匹配子串 `"already"`，且断言的真正对象是
  "**是否** panic"这一确定性事实，而非措辞。
  **不能**由第 1 段推出"不可变借用没有代价"—— 它只说明借用检查允许它们共存，
  与运行期是否有开销无关（事实上 `&T` 在 codegen 后就是一个指针，
  但那是另一回事，本观察不涉及）。
  **不能**由第 5 段推出"`RefCell` 不安全"。它是**内存安全**的：
  冲突时 panic 而非产生别名可变引用。它牺牲的是"错误在编译期被发现"这一性质，
  不是内存安全性。
  **不能**推出"`RefCell` 放宽了借用规则"—— 规则一字未改，只是检查时机变了。
  `tests/c03_borrow.rs::sequential_borrows_are_legal_in_both_regimes`
  正是为断言这一点而写：合法的顺序访问在两种时机下都通过。

架构相关性：可跨架构推广。借用检查（无论编译期还是运行期）都不涉及任何架构特定行为：
编译期版本是纯静态分析，运行期版本是对一个 `isize` 的加减与符号判断。
panic 的**行内位置**（`c03.rs:76:34`）会随源码编辑漂移，但那与架构无关。

---

### C-04 / c04_lifetime  [NON-ASSERTION]

命令：`cargo run -p m1-ownership --example c04_lifetime`

输出：

```text
=== 1. 需要显式标注 vs 省略规则足够 ===
  longest(a, b) = "longer string"   （两个入参引用，必须显式标注 'a）
  first_word(a)  = "longer"   （一个入参引用，省略规则可推出）

=== 2. 标注只影响编译期：等价函数返回相同结果 ===
  elided    = "longer string"
  annotated = "longer string"
  —— 二者生成的机器码没有区别；生命周期在编译后被完全擦除

=== 3. 持有引用的结构体 ===
  excerpt.part() = "first sentence."
  —— 'a 表达的是：excerpt 不得比 text 活得更久

=== 4. PhantomData：零大小的类型级区分 ===
  size_of::<PhantomData<Ingress>>()  = 0
  size_of::<Tagged<Ingress>>()       = 4
  size_of::<u32>()                   = 4
  ingress.raw = 1, egress.raw = 2
  —— 两个类型的运行期表示完全相同，却不能互相赋值
```

解释：

  为什么会这样：
  `longest` 需要显式标注，是因为三条省略规则在这里全部不适用：
  规则 2 要求"恰好一个入参生命周期"（这里有两个），规则 3 要求有 `&self`（这里没有）。
  编译器此时**拒绝猜** —— 猜错的后果是悬垂引用，而两个候选之间没有可靠依据可分辨。
  `first_word` 只有一个入参引用，规则 2 直接给出答案，所以不必标注。
  第 2 段的两个函数返回相同结果，是因为标注是**纯编译期**的：它不生成任何机器码。
  MIR 里可以看到更强的版本 —— 两个函数体**逐字符相同**，且签名里连 `'a` 都不见了
  （见下方 IR 观察，生命周期在 MIR 之前就已被擦除）。
  第 4 段的 `size_of::<PhantomData<_>>() == 0` 不是实现巧合，而是标准库
  **文档化的保证**（`core/src/marker.rs:805`），所以可以拿来做稳定断言。
  它成立的原因很直白：`PhantomData<T>` 的定义是
  `pub struct PhantomData<T: PointeeSized>;` —— 一个**没有字段**的单元结构体
  （`marker.rs:811`）。它不"存了个假的 T"，运行期什么都没有；
  `T` 只出现在类型签名里，供类型检查器使用。
  于是 `Tagged<Ingress>` 与 `Tagged<Egress>` 的大小都等于 `u32`：
  类型层面的区分与运行期空间占用**完全无关**。

  这不能证明什么：
  第 2 段**不能**证明"生成的机器码相同"——那需要比对 codegen 产物，
  这里只观察了返回值。真正支持该结论的是下方的 MIR 摘录（两个函数体逐字符相同），
  而即便那样，严格说也只能证明**MIR 层**相同，不能排除后续 pass 的差异
  （实际上不会有，但本观察不提供证据）。
  **不能**由第 4 段推出"所有零大小类型都不影响布局"：ZST 的对齐要求仍会参与布局计算，
  只是 `PhantomData` 的对齐是 1，恰好不产生影响。换成
  `PhantomData<T>` 之外的高对齐 ZST（如 `[u64; 0]`）结论会变。
  **不能**由第 4 段推出 `Tagged<Ingress>` 与 `Tagged<Egress>` 不可互换 ——
  那是编译期事实，由 `compile_fail/c04_phantom_type_mismatch.rs`（E0308）断言。
  这里的输出恰恰只能证明它的**另一半**：运行期确实没有差别。
  **不能**推出"生命周期标注永远不影响行为"：标注错误会导致编译失败，
  这当然影响结果 —— 只是它不影响**已编译成功的程序**的运行期行为。

架构相关性：`size_of::<PhantomData<_>>() == 0` 可跨架构推广 ——
它是标准库文档化的保证，不依赖任何目标平台。
`size_of::<Tagged<Ingress>>() == 4` 也可跨架构推广，因为 `u32` 的大小由语言定义为 4，
且 `PhantomData` 不增加大小或对齐。
生命周期擦除同样与架构无关：它发生在 MIR 生成之前，早于任何 codegen 决策。

---

## 编译器诊断抄录（compile_fail 样本）

<!-- 完整 stderr 落盘在 target/compile-fail/<case>.stderr。
     MUST NOT 对 stderr 全文做相等比较 —— 断言对象是错误码。 -->

九个样本的**期望错误码与实际错误码逐一相符**，且每个样本的实际错误码集合
恰好只含一项（无追加的派生诊断），因此 §C4.4 的子集语义在本模块退化为集合相等。

| 样本 | 期望（`//! EXPECT:`） | 实际（`rf_harness` 提取） | 命中? |
|------|---------------------|------------------------|------|
| `c01_use_after_move.rs` | E0382 | E0382 | ✅ |
| `c01_manual_drop_call.rs` | E0040 | E0040 | ✅ |
| `c02_copy_with_drop.rs` | E0184 | E0184 | ✅ |
| `c02_move_out_of_borrow.rs` | E0507 | E0507 | ✅ |
| `c03_two_mut_borrows.rs` | E0499 | E0499 | ✅ |
| `c03_mut_while_shared.rs` | E0502 | E0502 | ✅ |
| `c04_missing_lifetime.rs` | E0106 | E0106 | ✅ |
| `c04_dangling_ref.rs` | E0597 | E0597 | ✅ |
| `c04_phantom_type_mismatch.rs` | E0308 | E0308 | ✅ |

校验命令：

```bash
cargo test -p m1-ownership compile_time_violations_report_expected_codes
for f in "$CARGO_TARGET_DIR"/compile-fail/c0*.stderr; do
    printf '%s: ' "$(basename "$f")"
    grep -o 'error\[E[0-9]*\]' "$f" | sort -u | tr '\n' ' '; echo
done
```

### c03_two_mut_borrows  [NON-ASSERTION]

期望错误码（样本首行 `//! EXPECT:` 声明）：E0499
实际错误码（`rf_harness` 提取）：E0499

原样抄录自 `$CARGO_TARGET_DIR/compile-fail/c03_two_mut_borrows.stderr`，
仅把 `-->` 行的绝对路径缩短为仓库相对路径：

```text
error[E0499]: cannot borrow `c.value` as mutable more than once at a time
  --> experiments/m1-ownership/compile_fail/c03_two_mut_borrows.rs:16:18
   |
15 |     let first = &mut c.value;
   |                 ------------ first mutable borrow occurs here
16 |     let second = &mut c.value;
   |                  ^^^^^^^^^^^^ second mutable borrow occurs here
17 |     *second += 1;
18 |     *first += 1;
   |     ----------- first borrow later used here

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0499`.
```

解释：

  为什么会这样：
  诊断的三个标注位置正好画出了借用检查器判定冲突所需的全部信息：
  第一个借用**在哪里产生**（15 行）、第二个借用**在哪里产生**（16 行）、
  以及第一个借用**在哪里仍被使用**（18 行）。
  第三条是关键 —— 若没有 18 行那次使用，NLL 会让第一个借用在 15 行之后立即结束，
  两个借用就不再重叠，这段代码将合法编译。冲突的成立条件是"同时存活"，
  而"存活"的判定终点是最后一次使用。这也解释了样本为什么刻意把 `*first += 1`
  放在 `*second += 1` 之后：把两个借用的活跃区间强行交错。

  这不能证明什么：
  **不能**把诊断措辞、行号、标注箭头的位置当作稳定契约 —— 它们随 rustc 版本漂移。
  唯一可断言的是错误码 `E0499`（rustc 的稳定契约）。
  **不能**推出"两个 `&mut` 在任何情况下都冲突"—— 不重叠时完全合法，
  见 `tests/c03_borrow.rs::sequential_borrows_are_legal_in_both_regimes`。
  **不能**由"编译器拒绝"推出"这段代码真的会出错"：借用检查器是**保守**的，
  它拒绝的是"它无法证明安全"的代码，而非"它证明了不安全"的代码。
  这一点与 eBPF verifier 的失败模式完全同源（见 concept.md 的关联表）。

架构相关性：可跨架构推广。借用检查是纯静态分析，在 codegen 之前完成，
错误码与判定结果都不依赖目标架构。

### c04_missing_lifetime  [NON-ASSERTION]

期望错误码：E0106
实际错误码：E0106

原样抄录自 `$CARGO_TARGET_DIR/compile-fail/c04_missing_lifetime.stderr`，
仅把 `-->` 行的绝对路径缩短为仓库相对路径：

```text
error[E0106]: missing lifetime specifier
 --> experiments/m1-ownership/compile_fail/c04_missing_lifetime.rs:9:37
  |
9 | pub fn longest(a: &str, b: &str) -> &str {
  |                   ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `a` or `b`
help: consider introducing a named lifetime parameter
  |
9 | pub fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
  |               ++++     ++          ++          ++

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0106`.
```

解释：

  为什么会这样：
  `help` 那一行把省略规则失效的原因说得比任何教材都直接：
  "does not say whether it is borrowed from `a` or `b`"。
  编译器**看得出**返回值来自两者之一，但它拒绝替作者选。
  这是设计选择而非能力不足：选错的后果是悬垂引用，
  而两个候选之间不存在任何可靠依据能分辨该选哪个。
  注意 `expected named lifetime parameter` 的插入符指向的是**返回类型**，
  而两个入参用 `----` 标出 —— 诊断在告诉你"候选来源有两个，落点在这里"。

  这不能证明什么：
  **不能**认为编译器给出的 `help` 修复建议总是语义正确的。
  它建议把两个入参都标成 `'a`，这恰好符合 `longest` 的意图；
  但若函数实际只可能返回 `a`，更精确的签名是
  `fn longest<'a>(a: &'a str, b: &str) -> &'a str`，约束更松、更好用。
  编译器给的是**能通过检查**的最简建议，不是最佳设计。
  **不能**推出"多个入参引用一定需要标注"：若返回类型不含引用，
  或有 `&self`（规则 3 生效），都不需要。
  **不能**把诊断里的 `++++` 对齐、help 措辞当作稳定内容 —— 仅错误码 `E0106` 可断言。

架构相关性：可跨架构推广。生命周期省略是编译器前端的推导算法，
在类型检查阶段完成，与目标架构无关。

---

## UB 判定记录

<!-- §C5：未运行工具时 `ub_verdict` 只能记 `n/a`，MUST NOT 记 `clean`（FR-019）。 -->

| 实验 | 事前预测（`PREDICT-UB`） | 工具与命令 | 实际类别 | `ub_verdict` | 命中? |
|------|----------------------|-----------|---------|-------------|-------|
| `c01_ownership` | 无 UB（全为安全代码） | 未运行 | — | **`n/a`** | — |
| `c02_move` | 无 UB（`mem::forget` 是安全函数，泄漏≠UB） | 未运行 | — | **`n/a`** | — |
| `c03_borrow` | 无 UB（冲突以 panic 收场，非 UB） | 未运行 | — | **`n/a`** | — |
| `c04_lifetime` | 无 UB（全为安全代码） | 未运行 | — | **`n/a`** | — |

本模块**不含任何 `unsafe` 代码**，因此按 §C5 未运行 Miri，四项 `ub_verdict` 全部记
**`n/a`** 而非 `clean`。这条纪律来自 FR-019：`clean` 是"工具跑过且未报告 UB"的结论，
"没跑工具"只能记 `n/a`。把未验证写成 `clean` 会让后续模块误以为这里已有 UB 保证。

US5（`unsafe` 与 UB）才是 Miri 的正式引入点。届时本模块的安全代码可作为
"Miri 对无 `unsafe` 代码报告为空"的对照基线 —— 但那需要**实际运行**之后才能记 `clean`。

---

## IR 观察

<!-- §C3.3：MIR 文本一律 NON-ASSERTION。
     IR 中可断言的部分 MUST 先转化为确定性量再写进 tests/。 -->

### C-01：drop glue 的插入位置与顺序  [NON-ASSERTION]

命令：`tools/emit-mir.sh m1-ownership --example c01_ownership`

摘录（`main` 中对应 example 第 2 段的基本块）：

```text
    bb19: {
        _30 = DropLog::new() -> [return: bb20, unwind: bb97];
    }

    bb20: {
        _32 = &_30;
        _31 = Noisy::<'_>::new(const "first", copy _32) -> [return: bb21, unwind: bb93];
    }

    bb21: {
        _34 = &_30;
        _33 = Noisy::<'_>::new(const "second", copy _34) -> [return: bb22, unwind: bb92];
    }

    bb22: {
        _36 = &_30;
        _35 = Noisy::<'_>::new(const "third", copy _36) -> [return: bb23, unwind: bb91];
    }

    bb23: {
        drop(_35) -> [return: bb24, unwind: bb91];
    }

    bb24: {
        drop(_33) -> [return: bb25, unwind: bb92];
    }

    bb25: {
        drop(_31) -> [return: bb26, unwind: bb93];
    }
```

同一份 MIR 里的 unwind 清理块（`bb91`…`bb93`）：

```text
    bb91 (cleanup): {
        drop(_33) -> [return: bb92, unwind terminate(cleanup)];
    }

    bb92 (cleanup): {
        drop(_31) -> [return: bb93, unwind terminate(cleanup)];
    }

    bb93 (cleanup): {
        drop(_30) -> [return: bb97, unwind terminate(cleanup)];
    }
```

解释：

  为什么会这样：
  这是 concept.md 里"销毁代码由编译器生成、销毁时刻由静态作用域决定"两句话的**直接证据**。
  `first` / `second` / `third` 是 local `_31` / `_33` / `_35`，
  而 `drop` terminator 的顺序是 `_35` → `_33` → `_31` —— 逆序，且**写在 MIR 里**。
  没有任何一行源码写了这三次 drop：它们是编译器在作用域末尾插入的。
  更值得注意的是每个 terminator 的 `unwind:` 分支。清理块构成一条**链**：
  `bb91` 销毁 `_33` 后落到 `bb92`，`bb92` 销毁 `_31` 后落到 `bb93`，
  `bb93` 销毁 `_30`（`DropLog` 本身）。也就是说 panic 路径上有一套**独立的**、
  同样按逆序排列的销毁序列。
  这条链的入口位置正是它精妙之处：构造 `third`（`_35`）的那次调用把 unwind 目标定为 `bb91`，
  而 `bb91` **不**销毁 `_35` —— 因为若 `Noisy::new("third", ..)` 自己 panic 了，
  `_35` 根本还没构造出来，销毁它就是读未初始化内存。
  同理构造 `second` 时的 unwind 目标是 `bb92`（`_33` 也还不存在），
  构造 `first` 时是 `bb93`。
  换句话说，编译器为**每一个构造进度点**都算出了当时"已初始化、需要清理"的确切集合，
  并把它们编排成一条可复用的链。
  这就是 drop glue 必须由编译器生成的根本理由：作用域可能以两种方式退出
  （正常返回 / unwind），unwind 又可能发生在任意一个中间时刻，
  每个时刻要清理的集合都不同。手写这套逻辑既繁琐又极易漏 —— 这正是 C++
  需要 RAII 加异常安全规则、而 C 里干脆用 `goto cleanup` 手工维护的那个问题。

  这不能证明什么：
  **不能**把具体的 local 编号（`_31`）、基本块编号（`bb23`）当作稳定内容 ——
  它们随源码任何编辑而变，是 NON-ASSERTION 中最易漂移的部分。
  可断言的是同一事实的确定性投影：`tests/c01_ownership.rs::drop_order_is_reverse_of_declaration`
  用销毁日志断言了顺序，那才是稳定形式。
  **不能**推出"这些 drop 调用会出现在最终机器码里"—— MIR 之后还有优化 pass，
  `Noisy` 恰好有 `Drop` 实现所以不会被消除，但对没有 `Drop` 的类型，
  drop terminator 通常在 codegen 前就被移除了。
  **不能**由此推断 drop glue 的**内部**结构：这里看到的是"调用点"，
  glue 的函数体（递归销毁字段）在本次 `--emit=mir` 的输出里没有展开，
  因为它是编译器在 codegen 阶段合成的 shim，而非一个 MIR body。

架构相关性：可跨架构推广。MIR 是架构无关的中间表示，
drop terminator 的位置与顺序在此已完全确定，后续 codegen 只负责把它们翻译成指令。
唯一与架构有关的是 unwind 机制的实现方式（DWARF vs SEH），
但那不改变这里观察到的控制流结构。

### C-02：移动后的源 local 没有 drop  [NON-ASSERTION]

命令：`tools/emit-mir.sh m1-ownership --example c02_move`

摘录：

```text
    bb11: {
        _21 = &_19;
        _20 = Tracked::<'_>::new(const 1_u32, copy _21) -> [return: bb12, unwind continue];
    }

    bb12: {
        _22 = move _20;              // ← 移动：整个操作就是这一条语句
        ...
    }

    bb16: {
        drop(_22) -> [return: bb17, unwind continue];   // ← 只有目标被销毁
    }
```

计数校验（`M=target/ir/m1-ownership-c02_move.mir`）：

```text
$ grep -c 'drop(_20)' $M     # 移动的源 local（first）
0
$ grep -c 'drop(_22)' $M     # 移动的目标 local
2                            # 1 次正常路径 + 1 次 unwind 路径
$ grep -c 'drop(_67)' $M     # 被 mem::forget 的值
0
```

解释：

  为什么会这样：
  `drop(_20)` 出现 **0 次**，是"移动只是簿记"这一命题最干净的证据。
  移动本身在 MIR 里只是一条 `_22 = move _20;`——
  语义上的全部变化就是编译器此后把 `_20` 当作已移出，
  于是**根本不为它生成 drop terminator**。
  这就是为什么链式移动之后销毁总次数仍是 1（见 C-02 记录块第 2 段）：
  销毁责任不是被复制，而是被转移；MIR 层面表现为 drop 点只有一个，且在目标上。
  `drop(_22)` 出现 2 次而非 1 次，是正常路径与 unwind 路径各一份，
  与 C-01 观察到的同一机制，不代表销毁两次。
  `drop(_67)` 为 0 则解释了 `mem::forget`：它取得所有权后既不销毁也不归还，
  调用点之后源 local 已被视为移出，于是同样没有 drop terminator。
  三个计数放在一起，把"谁负责销毁"这件事在 IR 层完整地画了出来。

  这不能证明什么：
  **不能**推出"移动不产生任何机器指令"。`_22 = move _20` 在 codegen 后可能是一次
  memcpy，也可能因为两个 local 被分配到同一位置而完全消失 ——
  MIR 层看不到这个决定。本次观察在 dev profile（未优化）下取得，
  优化后的 MIR/机器码会不同。
  **不能**把 local 编号当作稳定内容（同 C-01）。
  可断言的投影是销毁计数：`tests/c02_move.rs::move_does_not_duplicate_drops`。
  **不能**由 `drop(_67) == 0` 推出"`forget` 泄漏了内存"——
  它只证明销毁没发生；本实验的 `Tracked` 不持有堆内存，实际没有内存被泄漏。

架构相关性：可跨架构推广。移动的簿记语义与 drop terminator 的生成都在 MIR 层完成，
架构无关。**不可**跨架构推广的是"移动是否被编译成实际的 memcpy"——
那取决于目标的调用约定与寄存器数量，而本条观察刻意不依赖它。

### C-04：生命周期在 MIR 之前已被完全擦除  [NON-ASSERTION]

命令：`tools/emit-mir.sh m1-ownership --example c04_lifetime`

摘录（example 第 2 段的两个等价函数）：

```text
fn elided(_1: &str) -> &str {
    debug s => _1;
    let mut _0: &str;

    bb0: {
        _0 = copy _1;
        return;
    }
}

fn annotated(_1: &str) -> &str {
    debug s => _1;
    let mut _0: &str;

    bb0: {
        _0 = copy _1;
        return;
    }
}
```

解释：

  为什么会这样：
  两个函数体**逐字符相同**。但比"相同"更值得注意的是另一件事：
  `annotated` 的源码签名是 `fn annotated<'a>(s: &'a str) -> &'a str`，
  而 MIR 里打印出来的是 `fn annotated(_1: &str) -> &str` ——
  `'a` **消失了**。生命周期在 MIR 生成之前就已被擦除，
  因为借用检查在此之前已经完成，此后再没有任何 pass 需要这个信息。
  这正是"标注是给借用检查器看的、不生成任何机器码"的确切含义：
  它不是"生成了相同的代码"，而是"到这一层它已经不存在了"。
  同一份 MIR 里还有一个相邻的证据：`Tagged::<Ingress>::new` 与
  `Tagged::<Egress>::new` 是**两个不同的单态化实例**（两条不同的 Call terminator），
  说明类型参数的区分一直保留到单态化阶段 —— 与生命周期形成对照：
  类型参数影响 codegen，生命周期不影响。

  这不能证明什么：
  **不能**由 MIR 相同推出"机器码相同"。MIR 之后还有优化 pass 与 codegen，
  本观察只能证明**到 MIR 这一层**二者已无区别。
  （实践中它们确实生成相同代码，但那需要比对 codegen 产物才能声称。）
  **不能**推出"生命周期标注没有作用"—— 它在借用检查阶段起决定性作用，
  `compile_fail/c04_dangling_ref.rs`（E0597）就是它起作用的证据。
  擦除发生在检查**之后**，顺序不能颠倒理解。
  **不能**由"两个单态化实例"推出"`PhantomData` 有运行期成本"：
  实例是两个，但每个实例里 `PhantomData` 都不占空间，
  两个 `Tagged` 的大小同为 4（见 C-04 记录块）。

架构相关性：可跨架构推广。生命周期擦除发生在 MIR 生成阶段，
早于任何 codegen 决策，与目标架构完全无关。
单态化同样是架构无关的前端行为。
