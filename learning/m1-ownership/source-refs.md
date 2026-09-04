# Module 1 源码引用

**Story**: US1 | **Capabilities**: C-01…C-04 | **依据**: FR-005 / SC-003 /
[learning-artifact-contract §B](../../specs/001-rust-foundation/contracts/learning-artifact-contract.md)

路径根：`$(rustc --print sysroot)/lib/rustlib/src/rust/library/`
本机为 `/root/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/`

行号在 pinned 工具链 **1.98.0**（`88d9e12ae 2026-08-18`）下固定，因此可被记录并复核（规则 B1）。
复核命令附在每条后面。

---

## 引用表

| C-ID | 路径（相对 `library/`） | 符号 | 行 | kind | 这段源码回答了什么 |
|------|----------------------|------|----|------|------------------|
| C-01 | `core/src/ops/drop.rs` | `pub const trait Drop` | 209 | library | 销毁行为由哪个 trait 定义；为什么它**不能**被手动调用 |
| C-01 | `core/src/ops/drop.rs` | `#[lang = "drop"]` | 206 | library | 它是**语言项**而非普通 trait —— 编译器直接认得它，据此生成 drop glue |
| C-01 | `core/src/marker.rs` | `pub trait Copy: Clone` | 454 | library | "赋值即复制"的标记；它与 `Drop` 为何互斥 |
| C-01 | `core/src/marker.rs` | `Copy` 文档"any type implementing `Drop` can't be `Copy`" | 419 | library | 互斥的**理由**原文（责任归属：可复制的值无法确定由哪一份负责释放） |
| C-02 | `core/src/mem/mod.rs` | `pub const fn replace<T>` | 953 | library | 如何在**不留下无效状态**的前提下把值换出来 |
| C-02 | `core/src/mem/mod.rs` | `pub const fn take<T: [const] Default>` | 886 | library | 为什么"取走"需要 `Default` 约束而 `replace` 不需要 |
| C-02 | `core/src/mem/mod.rs` | `pub const fn forget<T>` | 189 | library | 如何让值**不被销毁**；它的实现只有一行 `ManuallyDrop::new(t)` |
| C-02 | `core/src/mem/mod.rs` | `pub const fn swap<T>` | 822 | library | 两个位置互换，全程无无效状态 —— 与 `replace` 同源的手法 |
| C-03 | `core/src/cell.rs` | `pub struct RefCell<T: ?Sized>` | 849 | library | 把借用检查推迟到运行期的类型，其字段构成 |
| C-03 | `core/src/cell.rs` | `type BorrowCounter = isize` | 945 | library | **借用状态用一个有符号整数编码**：这是运行期借用检查的全部数据结构 |
| C-03 | `core/src/cell.rs` | `const UNUSED: BorrowCounter = 0` / `is_writing` / `is_reading` | 946 / 949 / 954 | library | 编码方案：0 = 无借用，正数 = 读借用计数，负数 = 写借用 |
| C-03 | `core/src/cell.rs` | `panic_already_mutably_borrowed` | 924 | library | 运行期违规的**后果是 panic**，而非编译失败 |
| C-03 | — Rust Reference §"Borrow checker"（`reference/destructors.html` 与 NLL RFC 2094） | — | — | **reference-fallback** | 编译期借用检查属编译器内建（`rustc_borrowck`），无库代码对应 |
| C-04 | `core/src/marker.rs` | `pub struct PhantomData<T: PointeeSized>` | 811 | library | 不占空间却参与类型检查的类型；定义只有**一行**，无字段 |
| C-04 | `core/src/marker.rs` | `PhantomData` 文档 `size_of::<PhantomData<T>>() == 0` | 805 | library | 零大小由标准库**文档化保证**，不是实现巧合 → 可作稳定断言 |
| C-04 | — Rust Reference §"Lifetime elision" | — | — | **reference-fallback** | 省略规则是编译器的推导算法，无库代码对应 |

### 复核命令

```bash
SRC="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
sed -n '206,209p'  "$SRC/core/src/ops/drop.rs"    # C-01 Drop + lang item
sed -n '419p;454p' "$SRC/core/src/marker.rs"      # C-01 Copy 与互斥理由
sed -n '189p;822p;886p;953p' "$SRC/core/src/mem/mod.rs"  # C-02 forget/swap/take/replace
sed -n '849,851p;945,956p' "$SRC/core/src/cell.rs"       # C-03 RefCell + BorrowCounter
sed -n '805p;811p' "$SRC/core/src/marker.rs"      # C-04 PhantomData
```

---

## 读这些源码时最值得注意的三件事

### 1. `Drop` 是**语言项**，不是普通 trait

`#[lang = "drop"]`（drop.rs:206）意味着编译器对它有内建认知。
普通 trait 是"库定义、编译器不特别对待"；语言项是"编译器在代码生成时直接查它"。

这解释了两件事：为什么 `Drop::drop` 不能手动调用（`E0040`），
以及为什么"销毁代码"在源码里找不到 —— 它由编译器在作用域末尾**生成**（drop glue），
不是任何一行 Rust 写出来的。想看它，只能看 MIR。

### 2. 契约里写的 `BorrowFlag`，在 1.98.0 里实际叫 `BorrowCounter`

plan.md 的 Capability Gate Matrix 写的是 `RefCell`/`BorrowFlag`，
而 1.98.0 的实际符号是 **`BorrowCounter`**（cell.rs:945）。

按规则 B1，源码引用记的是**实际**符号名，不是计划里的预期名。这条差异本身是有价值的信息：
它说明 plan 阶段的源码定位是"范围"而非"承诺"，实施时必须真的打开文件去看。

更值得注意的是它的类型：**`isize`**。整个运行期借用检查的状态就是**一个有符号整数**：

- `0`（`UNUSED`）= 没有借用
- 正数 = 当前有几个不可变借用（每次 `borrow()` 加一）
- 负数 = 当前有可变借用（`borrow_mut()` 减一）

编译期借用检查器要做全函数的控制流分析，运行期版本只需要一个 `isize` 的加减和符号判断。
这个对比直接量化了"把检查从编译期挪到运行期"省下了什么、又付出了什么。

### 3. `PhantomData` 的定义只有一行，且**没有字段**

```rust
pub struct PhantomData<T: PointeeSized>;
```

它不是"存了个假的 T"，而是一个**单元结构体**（unit struct）——
运行期什么都没有。类型参数 `T` 只出现在类型签名里，供类型检查器使用。

`size_of::<PhantomData<T>>() == 0` 写在文档（marker.rs:805）里，
是**标准库承诺**而非实现细节，所以可以拿来做稳定断言。

（`PointeeSized` 是 1.98.0 的 sizedness 层次里最宽松的那一档，
比 `?Sized` 还宽，使 `PhantomData` 能用于任何类型包括 extern type。）

---

## reference-fallback 理由说明

本模块用了两条 fallback，均属 §B2 已知的三项之内（C-03 borrowck、C-04 elision）：

| C-ID | 为什么无库代码对应 |
|------|------------------|
| C-03 | 编译期借用检查由 `rustc_borrowck` 实现，是**编译器**的一部分，不在 `library/` 下。`core/src/cell.rs` 提供的是它的**运行期对照物**，二者是同一条规则的两种执行时机，不能互相替代。因此 C-03 同时有一条 library 引用（运行期版本）和一条 fallback（编译期版本）。 |
| C-04 | 生命周期省略是编译器的**推导算法**，规则写在 Rust Reference 里，没有任何库代码承载它。能找到的只有 `PhantomData` 这类与生命周期**协作**的类型，而非规则本身。 |

两条 fallback 都不是"没找到"，而是"确实不存在"。区别在于前者是能力不足，后者是事实 ——
按 FR-005，后者改记语言参考位置即为已满足。
