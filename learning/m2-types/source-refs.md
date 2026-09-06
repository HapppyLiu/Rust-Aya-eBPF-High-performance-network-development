# Module 2 源码引用

**Story**: US2 | **Capabilities**: C-05…C-07 | **依据**: FR-005 / SC-003 /
[learning-artifact-contract §B](../../specs/001-rust-foundation/contracts/learning-artifact-contract.md)

路径根：`$(rustc --print sysroot)/lib/rustlib/src/rust/library/`
本机为 `/root/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/`

行号在 pinned 工具链 **1.98.0**（`88d9e12ae 2026-08-18`）下固定，因此可被记录并复核（规则 B1）。

---

## 引用表

| C-ID | 路径（相对 `library/`） | 符号 | 行 | kind | 这段源码回答了什么 |
|------|----------------------|------|----|------|------------------|
| C-05 | `core/src/option.rs` | Representation / null pointer optimization | 118–153 | library | 哪些 `T` 保证 `Option<T>` 与 `T` 同宽、同对齐、同调用 ABI；`&U` 在名单上 |
| C-05 | `core/src/option.rs` | `pub enum Option<T>` | 598 | library | `Option` 本身仍是两变体 enum；优化发生在**布局**，不改源码形状 |
| C-05 | `core/src/mem/mod.rs` | `pub const fn size_of<T>` | 373 | library | 大小查询是 `const`：结果在编译期就确定 |
| C-05 | `core/src/mem/mod.rs` | `pub const fn align_of<T>` | 540 | library | 对齐同样是 `const`，由类型的 ABI 对齐要求决定 |
| C-05 | `core/src/mem/mod.rs` | `pub macro offset_of` | 1617 | library | 字段偏移是编译期内建，回答的是"从哪一字节开始"而不是"一共多宽" |
| C-06 | `core/src/fmt/mod.rs` | `pub trait Display` | 1187 | library | `{}` 解析到哪个 trait；要求实现的方法是 `fmt(&self, &mut Formatter) -> Result` |
| C-06 | `core/src/fmt/mod.rs` | `fn fmt` | 1212 | library | 输出槽是 `Formatter`，不是返回 `String` —— 格式化可以零分配 |
| C-06 | `core/src/ops/deref.rs` | `pub const trait Deref` | 139 | library | 自动解引用的语言项；关联类型 `Target` 决定方法查找落到谁身上 |
| C-06 | `core/src/ops/deref.rs` | `fn deref(&self) -> &Self::Target` | 150 | library | `Deref` 交出的是借用，不是所有权 |
| C-07 | `core/src/iter/traits/iterator.rs` | `pub const trait Iterator` | 42 | library | `Item` 是**关联类型**，一份实现只能有一种元素类型 |
| C-07 | `core/src/cmp.rs` | `pub const trait PartialEq` | 250 | library | `==` 的 bound；`PartialOrd` 以它为超 trait |
| C-07 | `core/src/cmp.rs` | `pub const trait PartialOrd` | 1366 | library | `>` / `>=` 需要的 bound；只实现 `PartialEq` 不能比较大小 |
| C-06 | — rustc 对象安全 / 胖指针 ABI | 直接调用 vs vtable 间接调用 | — | **reference-fallback** | 分发方式是代码生成决策，`library/` 下没有"vtable 怎么建"的库函数 |
| C-07 | — rustc 单态化（`-Z print-mono-items`） | 一份源码对应几份实例 | — | **reference-fallback** | 单态化是编译器后端，标准库只提供 bound，不提供"复制函数体"的 API |

### 复核命令

```bash
SRC="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
sed -n '118,153p' "$SRC/core/src/option.rs"          # C-05 NPO 保证与类型名单
sed -n '598p'     "$SRC/core/src/option.rs"          # C-05 Option 定义
sed -n '373p;540p;1617p' "$SRC/core/src/mem/mod.rs"  # C-05 size_of / align_of / offset_of
sed -n '1187p;1212p' "$SRC/core/src/fmt/mod.rs"      # C-06 Display + fmt
sed -n '139,150p' "$SRC/core/src/ops/deref.rs"       # C-06 Deref + Target
sed -n '42,46p'   "$SRC/core/src/iter/traits/iterator.rs"  # C-07 Iterator + Item
sed -n '250p;1366,1368p' "$SRC/core/src/cmp.rs"      # C-07 PartialEq / PartialOrd
```

---

## 读这些源码时最值得注意的三件事

### 1. `Option` 的 NPO 是**文档化保证**，不是实现巧合

`option.rs:118` 起用 "Rust guarantees to optimize" 列出名单。
`&U`（`U: Sized`）在表里，所以 `size_of::<Option<&u8>>() == size_of::<&u8>()`
可以写成稳定断言。`u8` 不在名单里 —— 它的 256 个位型都是合法值，没有空位给 `None`。

定义本身（`:598`）仍然是 `None` / `Some(T)` 两个变体。优化改的是**布局**，不是语法。

### 2. `Iterator::Item` 是关联类型，不是泛型参数

`type Item` 写在 trait 体内（iterator.rs:46）。
一份 `impl Iterator for Foo` 只能有一种元素类型。
若写成 `trait Iterator<Item>`，同一类型就能对多种 `Item` 各实现一次，
调用方每次都得写出 `Iterator<i32>` —— 那是另一种接口。
`count_gt` 的 bound 写在 `I::Item: PartialOrd`，约束的是元素，不是迭代器。

### 3. `PartialOrd` 以 `PartialEq` 为超 trait

`cmp.rs:1366`：`trait PartialOrd<Rhs = Self>: PartialEq<Rhs>`。
所以 `>` 能用时 `==` 一定能用；反过来不成立。
缺 bound 时，直接写 `a > b` 会先报 **E0369**（运算符找不到）；
调用一个**已经**写了 `T: PartialOrd` 的函数，才报 **E0277**（bound 不满足）。
两条路径说的是同一件事，错误码对准的环节不同。

---

## reference-fallback 理由说明

| C-ID | 为什么无库代码对应 |
|------|------------------|
| C-06 分发方式 | vtable 的布局与间接调用由 rustc 在代码生成时插入。`Display` / `Deref` 只定义**接口**；"这次调用走直接 call 还是 load-from-vtable" 不在 `library/`。改记本模块 LLVM IR 观察。 |
| C-07 单态化 | 编译器按具体类型复制函数体。标准库没有 `monomorphize()` 这种 API。观察手段是 nightly `-Z print-mono-items=yes`。 |

两条都不是"没找到"，而是"确实不在库里"。
