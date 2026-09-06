# Feynman: Module 2 — 类型系统与抽象

**Capabilities covered**: C-05, C-06, C-07

---

## 1. 用自己的话解释

面向"没学过 Rust 但懂 C"的同事。

C 里你每天都在和三样东西打交道：结构体怎么排字节、`union` 怎么省空间、
函数指针怎么实现"同一套接口、不同实现"。Rust 把这三件事都写进了类型，
并且多了一条 C 没有的编译期复制。

**第一，结构体的大小不是字段大小加起来那么简单。**
C 程序员其实已经知道：`char` 后面紧跟 `int`，中间往往有三个空字节。
Rust 默认布局不保证字段顺序，但你加上和 C 相同的那份约定之后，算法就一样了：
每个字段的起始地址必须能被它对齐要求整除，结构体本身再垫到对齐的倍数。
所以调换两个字段，**数据一个字节没变，空位会搬家**，总大小可能跟着变。
只看大小、不看偏移，你会把空位误当成数据。

**第二，枚举就是带标签的联合，而且标签由语言管。**
C 里你手写 `enum tag + union payload`，编译器不管你有没有先看 tag 再读对应字段。
看错了就是未定义行为。Rust 的枚举把 tag（判别式）和最大那个变体的数据
排进同一块内存，四个变体共用这一块，同一时刻只有一个有效。
读的时候必须通过匹配，编译器不让你"当成另一个变体来读"。
没有垃圾回收也能这么做：tag 就是一个编译期排好的整数，
不需要对象头，不需要运行期类型信息。

有一个例外值得单独说。有些类型的全部位型里，有一个是非法的
（比如"指向对象的指针不能是空"）。编译器会把"没有值"这个状态
塞进那个非法位型里，于是 `Option<&T>` 和 `&T` 一样宽。
`unsigned char` 没有非法位型，256 个值都合法，所以它的可选版必须多占一个字节。
这不是魔法，是"值域有没有空位"。

**第三，接口可以在编译期选定实现，也可以在运行期查表。**
C 里这对应"直接调用具名函数"和"通过函数指针调用"。
Rust 的 trait 就是那张接口。写成泛型参数时，调用点已经知道具体类型，
编译器把调用改成对那个具体函数的直接跳转 —— 和 C 里调用 `ping_score()` 没两样。
写成"指向接口的引用"时，调用点只带着一个数据指针外加一张函数指针表，
每次先从表里取出地址再跳 —— 和 C 里的 `ops->score(obj)` 没两样。
指向接口的引用因此比普通指针多一个字：那就是表的地址。

两种写法对同一输入可以给出同一个数字。那只说明选对了实现，
不说明跳转一样贵。直接跳和"先 load 再跳"不是一回事。

**第四，泛型不是运行时的模板，是编译期的复印机。**
C++ 模板程序员对这不陌生：你写一份，编译器按用到的类型各印一份。
一份源码对 `uint8_t` / `uint16_t` / `uint32_t` 各调用一次，
中间表示里就会出现三个函数，不是一个。
缺了"这个类型必须能比较大小"这种前提，复印还没开始，编译器就拒绝 ——
这是编译错误，不是运行起来结果不对。

把同一套比较改写成"通过接口引用调用"，返回值可以和泛型版一样。
账单不同：泛型把体积付在编译期（每多一种类型多一份机器码），
接口引用把跳转付在每次调用。后面读 Aya 的 map 和解析器时，
看签名上是"类型参数"还是"指向接口的引用"，就是在判断这笔账记在哪一侧。

---

## 2. 最小示例

### C-05 Struct / Enum — 判别式与载荷共享空间，niche 可省掉判别式

```rust
#[repr(u8)]
enum ConnState { Idle, Established { peer: u32 } }

assert!(size_of::<ConnState>() > size_of::<u32>());
assert_eq!(size_of::<Option<&u8>>(), size_of::<&u8>());
assert!(size_of::<Option<u8>>() > size_of::<u8>());
```

完整观察：[`examples/c05_layout.rs`](../experiments/m2-types/examples/c05_layout.rs)（69 行）

### C-06 Trait — 同一接口，两条分发路径

```rust
fn static_s<S: Score>(s: &S) -> u32 { s.score() }
fn dynamic_s(s: &dyn Score) -> u32 { s.score() }

assert_eq!(static_s(&Ping), dynamic_s(&Ping));
assert_eq!(size_of::<&dyn Score>(), 2 * size_of::<usize>());
```

完整观察：[`examples/c06_trait.rs`](../experiments/m2-types/examples/c06_trait.rs)（45 行）

### C-07 Generic — 一份源码，按类型复制

```rust
fn max_of<T: PartialOrd + Copy>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

assert_eq!(max_of(3u8, 9u8), 9);
assert_eq!(max_of(3u32, 9u32), 9);
assert_ne!(size_of::<Wrapper<u8>>(), size_of::<Wrapper<u64>>());
```

完整观察：[`examples/c07_generic.rs`](../experiments/m2-types/examples/c07_generic.rs)（38 行）

---

## 3. 底层机制

每条论断带依据。审查时优先核对这里。

1. 布局查询发生在编译期：`size_of` / `align_of` / `offset_of` 都是 `const`。
   **依据**：`core/src/mem/mod.rs:373` / `:540` / `:1617`（`source-refs.md`）。
2. `repr(C)` 包头的 `length` 偏移为 4、总长 8，由字段顺序和对齐决定，不是猜测。
   **依据**：`tests/c05_layout.rs::repr_c_header_has_deterministic_layout`。
3. 同样的有效字段换顺序，变的是空位：`LooseHeader` 的 `u16` 落在偏移 2，总长 6 > 4。
   **依据**：`tests/c05_layout.rs::field_order_changes_padding_not_payload`。
4. `repr(u8)` 状态机必须给判别式留空间，大小严格大于最大载荷。
   **依据**：`tests/c05_layout.rs::enum_occupies_discriminant_plus_payload_space`。
5. `Option<&U>` 与 `&U` 同宽、同对齐，是标准库文档化保证，不是实现巧合。
   **依据**：`core/src/option.rs:118-153`；
   `tests/c05_layout.rs::option_ref_has_the_same_width_as_the_reference`。
6. `u8` 值域已满，`Option<u8>` 必须另辟判别式。
   **依据**：`tests/c05_layout.rs::option_u8_needs_an_extra_discriminant_byte`。
7. `&dyn Trait` 是胖指针，宽度为两个 `usize`。
   **依据**：`tests/c06_trait.rs::dyn_trait_reference_is_two_usizes`。
8. 静态分发与动态分发对同一输入返回值相等；差的是选定实现的时机。
   **依据**：`tests/c06_trait.rs::static_and_dynamic_dispatch_agree_on_results`。
9. 静态路径在 IR 里是对 `score_static::<Ping>` 的直接 `call`（一个数据指针）；
   动态路径把 `@vtable.N` 作为第二实参传入。
   **依据**：OBSERVATIONS「C-06：静态分发是直接调用，动态分发带 vtable 实参」。
10. `score_dynamic` 函数体从 vtable load 函数指针后做间接 `call`。
    **依据**：OBSERVATIONS「C-06：`score_dynamic` 函数体是 load-from-vtable + 间接 call」。
11. `{}` 解析到 `Display::fmt`，输出槽是 `Formatter`；`Deref` 交出 `&Target`，不交所有权。
    **依据**：`core/src/fmt/mod.rs:1187/1212`、`core/src/ops/deref.rs:139/150`；
    `tests/c06_trait.rs::display_and_deref_resolve_to_the_impls`。
12. `max_of` 对 `u8`/`u16`/`u32` 各有一份单态化实例，实例数是 3 不是 1。
    **依据**：OBSERVATIONS「C-07：`max_of` 对三个整数类型各有一份单态化实例」。
13. 单态化改变布局：`Wrapper<T>` 大小等于 `T`。
    **依据**：`tests/c07_generic.rs::monomorphization_produces_type_dependent_layout`。
14. 缺 `PartialOrd` bound 时，调用已写明该 bound 的函数报 E0277；拒绝发生在单态化之前。
    **依据**：`tests/c07_generic.rs::missing_partial_ord_bound_is_e0277`；
    OBSERVATIONS「c07_missing_bound」。
15. 同一套比较的泛型版与 trait 对象版行为等价。
    **依据**：`tests/c07_generic.rs::generic_and_dyn_comparison_are_behaviorally_equivalent`。
16. `Iterator::Item` 是关联类型；`PartialOrd` 以 `PartialEq` 为超 trait。
    **依据**：`core/src/iter/traits/iterator.rs:42`、`core/src/cmp.rs:250/1366`；
    `tests/c07_generic.rs::iterator_bound_filters_by_partial_ord_on_item`。

本节共 **16** 条论断，每条带一处依据。核对：

```bash
grep -c '\*\*依据\*\*' feynman/m2-types.md   # 期望 16
```

---

## 4. 常见误区

### C-05

- **误解**：结构体大小等于字段大小之和，调换字段不会改变大小。
  → **实际**：对齐空位会搬家，总大小可以变，有效载荷不变。
  → **证据**：`tests/c05_layout.rs::field_order_changes_padding_not_payload`。
- **误解**：`Option<T>` 永远比 `T` 宽一个判别式。
  → **实际**：名单内的 `T`（含 `&U`）保证同宽；`u8` 这种值域已满的类型才会增宽。
  → **证据**：`option_ref_has_the_same_width_as_the_reference` 与
  `option_u8_needs_an_extra_discriminant_byte`。

### C-06

- **误解**：两种分发返回值一样，所以代价一样。
  → **实际**：结果相等只说明实现选对了；IR 里一条是直接 `call`，一条是 load + 间接 `call`。
  → **证据**：`static_and_dynamic_dispatch_agree_on_results` +
  OBSERVATIONS「C-06：直接调用 vs vtable」。
- **误解**：`&dyn Trait` 和 `&T` 一样，都是一个指针。
  → **实际**：前者是胖指针，宽两个 `usize`。
  → **证据**：`dyn_trait_reference_is_two_usizes`。

### C-07

- **误解**：泛型函数编译后仍是一份，运行时再按类型分发。
  → **实际**：按类型复制发生在编译期；打印开关能数出三份 `max_of`。
  → **证据**：OBSERVATIONS「C-07：`max_of` 对三个整数类型各有一份单态化实例」。
- **误解**：缺 bound 是运行起来结果不对。
  → **实际**：编译失败，单态化尚未开始；错误码 E0277。
  → **证据**：`missing_partial_ord_bound_is_e0277`。

---

## 5. 验证性问题

1. 把 `#[repr(C)]` 结构体的两个字段对调，`size_of` 变了。变的是数据量还是空位？
   你怎么用 `offset_of` 证明？
   **回答**：空位。`LooseHeader` 有效载荷仍是 1+2+1，`length` 的偏移从"紧挨着 version"
   变成 2，中间空出 1 字节给 `u16` 对齐。
   **指向**：(a) `tests/c05_layout.rs::field_order_changes_padding_not_payload`

2. 若语言允许 `Option<&T>` 与 `&T` 同宽，却仍把空指针当成合法的 `&T`，
   会破坏哪条不变量？这和 C 里空指针解引用是同一件事吗？
   **回答**：破坏的是"`&T` 永不为空"，`None` 将无法与某个 `Some` 区分。
   和 C 的空指针解引用相关但不是同一件事：C 允许空指针存在、解引用才是 UB；
   Rust 把"空"从 `&T` 的合法值域里拿掉，编码进 `Option`。
   **指向**：(b) `source-refs.md` `core/src/option.rs:118-153` Representation

3. 同一个方法，静态分发在 IR 里是直接调用，动态分发经一张表。
   若你只比较返回值就断言"两种分发代价相同"，错在哪一步？
   **回答**：把"结果"当成了"路径"。返回值相等只说明选中了同一份 `impl`；
   动态路径每次多一次 vtable load + 间接 call。
   **指向**：(c) OBSERVATIONS「C-06：静态分发是直接调用，动态分发带 vtable 实参」

4. 一份泛型函数对三个整数类型各调用一次。实例数是 3 还是 1？
   如果优化器把三个实例都内联掉了，"单态化发生过"还成立吗？
   **回答**：实例数是 3。内联是单态化之后的另一步；打印开关看到的是单态化清单，
   不是最终 `.text` 里还剩几个符号。
   **指向**：(c) OBSERVATIONS「C-07：`max_of` 对三个整数类型各有一份单态化实例」

5. 方法若返回 `Self`，为什么通常不能做成 trait 对象？
   这限制的是调用约定，还是类型信息在运行期不够？
   **回答**：两者是一件事。调用方必须在编译期知道返回值的大小和 ABI；
   `Self` 在 `&dyn Trait` 上是运行期才确定的类型，vtable 装不下"按这个大小接返回值"。
   这就是对象安全。`Score` 只返回 `u32` / `&str`，所以能做成 `&dyn Score`。
   **指向**：(a) `tests/c06_trait.rs::dyn_trait_reference_is_two_usizes`
   （对象安全是它能被写成 `&dyn` 的前提）

6. 后续读 Aya 的 `Map<K, V>` 时，怎么判断这一次查找走的是单态化后的具体实现，
   还是一次间接调用？判断依据是签名上的哪几个记号？
   **回答**：签名里是 `<K, V>` / `M: Map` 这类类型参数 → 静态分发、按 `K`/`V` 单态化；
   出现 `&dyn …` 或 `Box<dyn …>` → 动态分发、经 vtable。
   Aya 用户态的 map API 几乎总是前者。
   **指向**：(a) `tests/c07_generic.rs::generic_and_dyn_comparison_are_behaviorally_equivalent`
   与 (c) OBSERVATIONS「C-06：直接调用 vs vtable」

---

## 检验结果

| # | 检验项 | 合格标准（可判定） | 状态 |
|---|-------|-----------------|------|
| 1 | 自述概念 | 第 1 节存在；Rust 术语（判别式 / niche / vtable / 单态化 / trait）均在首次出现时用 C 世界的话解释 | **pass** |
| 2 | 最小示例 | C-05 / C-06 / C-07 各一段，分别为 6 / 5 / 6 行（均 ≤15），并链接到 `examples/` | **pass** |
| 3 | 底层机制 | 第 3 节 16 条论断，每条带依据标记；`grep -c '^\s*\*\*依据\*\*'` = 16 | **pass** |
| 4 | 常见误区 | 每个 covered capability ≥1 条，三段式齐备（C-05×2、C-06×2、C-07×2） | **pass** |
| 5 | 回答问题 | 6 个问题（≥5），每题回答指向断言名 / 源码引用 / 观测块标题之一 | **pass** |

**五项是合取。** 本表全部 pass → m2 的 Capability 可以进入 `accepted`。
