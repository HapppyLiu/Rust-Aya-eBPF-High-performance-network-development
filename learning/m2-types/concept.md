# Module 2: 类型系统与抽象

**Story**: US2（P2） | **Capabilities**: C-05…C-07 | **Prerequisite**: m1（accepted）

> 本文件属于 **Answer Track**。对应的 Learner Track 是
> [`learner/m2-types/guide.md`](../../learner/m2-types/guide.md)。
> 如果你还没做过那边的预测表，先去做 —— 这份文件读过之后就不能再自测了。

## 这个模块回答什么问题

1. `struct` 的大小是不是字段大小之和？字段顺序会不会凭空"增加数据"？
2. `enum` 怎么同时装下"我是哪个变体"和"这个变体带的数据"？为什么没有 GC 也能这么做？
3. 同一个 trait，写成泛型参数和写成 trait 对象，调用时走的是同一条路径吗？
4. 指向 trait 对象的引用为什么可能比普通引用宽？宽出来的那个字是什么？
5. 源码里一份泛型函数，编译之后对应几份机器码？缺 bound 的拒绝发生在这一步之前还是之后？

---

## 概念

### C-05 Struct / Enum

- **一句话定义**：`struct` 把字段按对齐规则排进一块连续内存；`enum` 在同一块内存里
  同时放下**判别式**（我是谁）和**最大变体的载荷**（我带着什么）。

- **底层机制**：

  布局是**编译期**算出来的。`size_of` / `align_of` / `offset_of` 都是 `const`
  （`core/src/mem/mod.rs:373 / :540 / :1617`），查询本身不读任何运行期值。

  **结构体**的大小 ≥ 各字段大小之和，多出来的是**对齐空位**。
  `#[repr(C)]` 把算法钉成 C ABI：字段按声明顺序排，每个字段的偏移必须是其对齐的倍数，
  结构体对齐取其字段的最大对齐，末尾再垫到对齐的倍数。

  所以调换字段顺序可以改变大小，而**有效载荷一个字节都没变**。
  `LooseHeader`（`u8, u16, u8`）是 6 字节，有效数据只有 4；
  多出的 2 字节是 `u16` 前面的 1 字节空位，加上末尾为对齐补的 1 字节。
  `offset_of` 把空位的位置变成可断言的数字。

  **枚举**在默认 / `repr(u8)` 布局下通常是：判别式 + 填充 + 最大载荷。
  `WireKind` 没有载荷，`size_of == 1`，大小就是判别式。
  `ConnState` 最大载荷是 `u32`（4 字节），但 `size_of` 是 8，
  **严格大于** `size_of::<u32>()` —— 多出来的是判别式及其对齐填充。
  四个变体共用这一块空间，同一时刻只有一个变体有效。
  这就是"没有 GC 也能用 tagged union"的原因：tag 是静态的、编译期就排好的整数，
  不需要运行期类型信息，也不需要堆上的对象头。

  **niche / null-pointer optimization** 是这条规则的例外，而且是**文档化保证**
  （`core/src/option.rs:118-153`）。当 `T` 的位型没有用满时，编译器可以把 `None`
  编码进那个非法位型。`&U` 的非法位型是空指针，所以
  `size_of::<Option<&u8>>() == size_of::<&u8>()`。
  `u8` 的 256 个位型都合法，没有空位，所以 `Option<u8>` 必须另辟一个判别式字节。

  `Option` 的源码形状不变（`:598` 仍是 `None | Some(T)`）。优化改布局，不改语法。

- **常见误解**：
  - **误解**："结构体大小等于字段大小之和" → **实际**：还要加上对齐空位；
    字段顺序会改变空位，不改变有效数据。
    → **证据**：`tests/c05_layout.rs::field_order_changes_padding_not_payload`。
  - **误解**："`Option<T>` 永远比 `T` 多一个判别式字节" → **实际**：名单内的 `T`
    （引用、`NonZero*`、`Box`…）保证同宽；`u8` 这种值域已满的类型才会增宽。
    → **证据**：`option_ref_has_the_same_width_as_the_reference` 与
    `option_u8_needs_an_extra_discriminant_byte`。
  - **误解**："enum 的大小等于最大变体" → **实际**：还要给判别式留空间
    （除非走 niche）。`repr(u8)` 钉死判别式之后，`ConnState` 严格大于 `u32`。
    → **证据**：`enum_occupies_discriminant_plus_payload_space`。

- **对应实验**：[`c05_layout`](../../experiments/m2-types/examples/c05_layout.rs)
  / [tests](../../experiments/m2-types/tests/c05_layout.rs)

---

### C-06 Trait

- **一句话定义**：trait 是一份**接口契约**。编译器在每个调用点选定实现：
  类型在编译期已知 → 直接调用（静态分发）；只知道"实现了这个 trait" →
  经 vtable 间接调用（动态分发）。

- **底层机制**：

  两条路径的**返回值可以相同**，选定实现的**时机**不同。

  | | 静态分发 `fn f<S: Score>(s: &S)` | 动态分发 `fn f(s: &dyn Score)` |
  |---|---|---|
  | 调用点是否知道具体类型 | 知道 | 不知道 |
  | 引用宽度 | 一个 `usize`（数据指针） | 两个 `usize`（数据指针 + vtable） |
  | LLVM IR | `call @score_static::<Ping>(ptr)` | `call @score_dynamic(ptr, ptr @vtable.Ping)` |
  | 实现何时选定 | 单态化时，编译期 | 每次调用，从 vtable load |

  `&dyn Score` 是胖指针。第二个字指向一张编译器生成的函数指针表。
  `c06_trait` 的 IR 里能直接看到两张表：

  ```text
  @vtable.1 = { …, ptr @<Ping as Score>::score, ptr @<Ping as Score>::label }
  @vtable.2 = { …, ptr @<Http as Score>::score, ptr @<Http as Score>::label }
  ```

  `score_dynamic` 的函数体（库 IR）把这件事写死了：入参是两个指针，
  从 vtable 偏移 24 处 load 函数指针，然后 `call i32 %fn(ptr %data)`。
  这不是类比，是机器码形状。

  **对象安全**限制了谁能做成 `&dyn Trait`。方法若返回 `Self`，调用方在运行期
  不知道 `Self` 有多大、该按什么 ABI 接返回值，所以这类方法通常不能进 vtable。
  `Score` 只返回 `u32` / `&'static str`，所以对象安全。

  `Display` 与 `Deref` 是同一套机制的语法糖入口：

  - `{}` 解析到 `Display::fmt`（`fmt/mod.rs:1187 / :1212`）。输出槽是 `Formatter`，
    不是返回 `String`，所以格式化可以不分配。
  - `label.len()` 在 `Label` 自己没有 `len` 时，沿 `Deref::Target = str` 继续找
    （`deref.rs:139 / :150`）。`deref` 交出的是 `&Target`，所有权不走。

  它们不创造第三种分发。`format!("{label}")` 对具体的 `Label` 仍是静态分发。

- **常见误解**：
  - **误解**："两种分发返回值一样，所以代价一样" → **实际**：结果相等只说明
    实现选对了；静态路径是直接 `call`，动态路径每次多一次 vtable load + 间接 call。
    → **证据**：`static_and_dynamic_dispatch_agree_on_results`（结果）+
    OBSERVATIONS「C-06：直接调用 vs vtable」（代价）。
  - **误解**："`&dyn Trait` 和 `&T` 一样宽，都是指针" → **实际**：前者是胖指针，
    宽度为 `2 * size_of::<usize>()`。
    → **证据**：`dyn_trait_reference_is_two_usizes`。
  - **误解**："`Deref` 把包装类型变成了内部类型" → **实际**：它只在**方法查找**
    时借出 `&Target`，所有权与类型身份都还在包装上。
    → **证据**：`display_and_deref_resolve_to_the_impls` —— `format!` 走的是
    `Label` 的 `Display`，不是 `String` 的。

- **对应实验**：[`c06_trait`](../../experiments/m2-types/examples/c06_trait.rs)
  / [tests](../../experiments/m2-types/tests/c06_trait.rs)

---

### C-07 Generic

- **一句话定义**：泛型是**编译期复制**：源码一份，编译器按每个具体类型各生成一份
  机器码（单态化）。缺了 trait bound，复制还没开始就被拒绝。

- **底层机制**：

  `fn max_of<T: PartialOrd + Copy>(a: T, b: T) -> T` 在源码里是一份。
  example 对 `u8` / `u16` / `u32` 各调用一次之后，nightly
  `-Z print-mono-items=yes` 打出三条：

  ```text
  MONO_ITEM fn m2_types::c07::max_of::<u8>
  MONO_ITEM fn m2_types::c07::max_of::<u16>
  MONO_ITEM fn m2_types::c07::max_of::<u32>
  ```

  实例数是 **3**，不是 1。这是"一份源码 ≠ 一份机器码"的直接观察。
  即便优化器稍后把三个实例都内联掉，**单态化已经发生过** ——
  打印开关看到的是单态化清单，不是最终二进制里还剩几份。

  单态化的布局后果可以稳定断言：`Wrapper<T>` 的大小等于 `T`，
  `Wrapper<u8>` 与 `Wrapper<u64>` 不同宽。
  功能后果也可以：三个实例各自算出正确的 `max`。

  **缺 bound 发生在单态化之前。** 编译器还在做候选选择：
  `T` 上有没有 `>` 对应的方法。本模块的 `compile_fail` 走的是
  "调用一个已经写了 `T: PartialOrd` 的函数"，因此错误码是 **E0277**
  （trait bound not satisfied）。若直接对无 bound 的 `T` 写 `a > b`，
  会先落成 **E0369**（运算符找不到）。两条路径说的是同一条规则，
  对准的是选择过程的不同环节。

  `PartialOrd` 以 `PartialEq` 为超 trait（`cmp.rs:1366`）。
  `Iterator::Item` 是关联类型（`iterator.rs:46`）：一份实现一种元素。
  `count_gt` 的 bound 写在 `I::Item: PartialOrd`，约束元素能比较，
  不要求迭代器本身能比较。

  同一套比较写成 trait 对象（`max_cmp_dyn`）与写成泛型（`max_cmp`），
  **返回值可以相等**。差的是：泛型在单态化时付体积，动态分发在每次调用时付间接跳转。
  可扩展性也反着来 —— 新类型要走泛型路径必须在调用点可见；走 `dyn` 路径只要
  实现了 trait，就可以在运行期才交进去。

- **常见误解**：
  - **误解**："泛型函数编译后仍是一份，运行时再按类型分发" → **实际**：
    按类型复制发生在编译期。运行期那份函数已经不带 `T`。
    → **证据**：OBSERVATIONS「C-07：max_of 的三条单态化实例」。
  - **误解**："缺 bound 是运行期错误 / 结果不对" → **实际**：编译失败，
    单态化尚未发生。
    → **证据**：`missing_partial_ord_bound_is_e0277`。
  - **误解**："泛型与 trait 对象是两种逻辑" → **实际**：逻辑可以相同，
    分发与代价不同。
    → **证据**：`generic_and_dyn_comparison_are_behaviorally_equivalent`。

- **对应实验**：[`c07_generic`](../../experiments/m2-types/examples/c07_generic.rs)
  / [tests](../../experiments/m2-types/tests/c07_generic.rs)

---

## 与后续学习的关联

FR-014 要求每项能力说明与后续 Linux / eBPF / Aya 的关联点，或显式标注为仅是基础。

| C-ID | 关联点 |
|------|-------|
| **C-05 Struct / Enum** | 直接对应。eBPF map 的 key/value、XDP / TC 看到的包头，本质都是**按字节解释的结构**。字段顺序导致的填充会让用户态 `size_of` 与内核侧 `bpf_map` 的 `value_size` 对不上；`Option<&T>` 那种 niche 则**不能**指望出现在 C 侧的 map 值里。协议状态机用 enum 表达，是因为无 GC：判别式是静态整数，不需要对象头。 |
| **C-06 Trait** | 直接对应，但要分开两侧看。Aya **用户态**大量用 trait（加载器、map 句柄）；调用点的 `Map<K, V>` 几乎总是静态分发。eBPF **内核程序**侧通常不能、也不该走 `&dyn Trait`：verifier 要的是一张看得见的调用图，vtable 间接跳转正好把它变成不透明。学会看签名上的 `<T: Trait>` vs `&dyn Trait`，就是在判断"这一跳 verifier / 编译器能不能看穿"。 |
| **C-07 Generic** | 直接对应。Aya 的 `HashMap<K, V>`、解析组合器都是单态化出来的具体函数。包解析器每支持一种协议头就多一份实例 —— 这是体积账单，不是抽象税。内核程序里一旦写成泛型，LLVM 必须能把实例内联到 verifier 看得到的直白控制流；内联失败的泛型，和 trait 对象一样会被拒绝。 |

三项都不是"仅为理解基础"。读不懂分发方式，就无法判断 Aya 用户态的一次 map 查找、
以及后续 Feature 里的解析分层，实际走到了哪一份机器码。
