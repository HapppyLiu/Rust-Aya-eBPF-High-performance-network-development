# Learner Track — Module 8: 综合实验（字节缓冲区报文解析器）

**Story**: US8（P3，Constitution XII 的 Build 环节） | **Capabilities**: C-01…C-24（全部） | **Prerequisite**: m1–m7 全部 accepted

> 本文件属于 **Learner Track**，不含答案，也不含设计方案。
> Answer Track 在 `learning/m8-capstone/`、`feynman/m8-capstone.md` 与 `experiments/m8-capstone/`。
> 什么时候可以翻，见 §5。

## 0. 开始之前

- **前置模块**：m1–m7。当前验收状态：查 `acceptance/capability-matrix.md`。
  三个硬前置（m1 / m5 / m7）必须已经满足，否则本模块不准开始。
- **本模块假定你已经能**：
  - 为借用视图写出生命周期，并说清它和拥有者的关系（C-01…C-04）；
  - 用 trait / 泛型 / 一处 trait 对象拆分解析步骤（C-05…C-07）；
  - 用 `Result` 传播失败、实现 `Iterator`、说清 `Box` 的所有权合同（C-08…C-11）；
  - 用编译器判定 `Send`/`Sync`，用原子计数而不是锁来记次数（C-12…C-14）；
  - 给 `unsafe` 块写出五要素，并把对外接口做成调用方逼不出 UB 的形状（C-15…C-20）；
  - 用 `repr(C)` 谈布局，而不把"声明了 repr(C)"当成跨语言调用（C-21）；
  - 指出 `no_std` + `alloc` 缺的是哪一类 OS services，以及因此必须换掉什么实现（C-22…C-24）。
- **本模块结束时你应该能**：独立完成一个面向字节缓冲区的最小解析场景，
  产物可 `cargo test -p m8-capstone` 且 Miri 判定为工具允许的那一档；
  对照 24 项能力清单，**每一项**都能指到综合实验里的一个文件和一个函数（不是"大概在解析器里"）。

**为什么排在最后**：单项能力通过不等于能组合使用。前面七个模块各自证明了一块积木；
本模块问的是：把它们放进同一个解析器之后，所有权、失败路径、裸指针和不存在的操作系统
会不会互相拆台。

**本模块的工作方式**：先自己设计分层（本文件只给约束，不给方案），再填预测表，再写代码。
库本身是 `no_std` + `alloc`，测试 crate 才有 `std`。动手之前先填 [`predictions.md`](./predictions.md)。

---

## 1. 本模块你要能回答的问题

先自己答一遍。答不上来是正常的 —— 记下来。**不要在本文件里找设计方案，这里没有。**

### 场景本身

1. 一个"面向字节缓冲区的最小解析器"最少要处理哪几件事
   （切一段字节、认出报文头、失败时返回什么、多帧怎么走）？
   哪些事**不必**放进这个最小场景？
2. 为什么这个场景能同时压上前七个模块的能力，而不是再做 24 个互不相干的小函数？
3. "单项能力通过 ≠ 能组合使用"——组合时最容易在哪几条缝上裂开
   （借用活不过所有者、错误类型对不上、unsafe 的检查被另一层绕开、缺 OS 却去起线程）？

### 分层设计（自己画，不要对照答案）

4. 你打算把解析器拆成几层？每一层对应哪些 C-ID？画一张表，先空着实现，只写职责。
5. 零拷贝视图的生命周期参数写在类型上还是只写在方法上？两种选择分别会把谁卡住？
6. `trait Parse`（或你起的同名接口）的关联类型如果带上输入切片的寿命，
   还能不能做成 trait 对象？若不能，你准备在哪一处单独使用 `dyn`？
7. 库是 `no_std` 的。并发计数还能否 `thread::spawn`？若不能，你用什么换实现？
   这件事体现的是 C-13 还是 C-22，还是两者的交界？
8. 报文头写成 `#[repr(C)]` 之后，还要不要在本 crate 里做真实的 C 调用？
   若不做，C-21 如何仍然能在综合实验里被**定位**，而不是被默认为"做过 FFI 了"？
9. 对外解析入口是安全的。边界检查放在 `unsafe` 块里面还是外面？
   调用方传入过短切片时，你保证的是返回失败，还是"程序没崩"？

### 验收形态

10. 24 项能力的定位粒度是"文件"就够了，还是必须到函数？漏一项会怎样？
11. 稳定断言允许写分配次数和 `size_of`，不允许写什么？Miri 没跑时，
    你能不能把 UB 判定写成"干净"？
12. `cargo build -p m8-capstone --no-default-features` 成功，证明的是
    "这个解析器在裸机上能跑"，还是别的什么？缺的那类 OS services 你列得出几条？

---

## 2. 你要自己定位的源码

给的是**搜索范围**和**要找什么**，行号要你自己填。
综合实验是**对照**这些位置，不是再发现一套新的库代码。

```bash
SRC="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
rg -n 'pub struct Iter' "$SRC/core/src/slice/"
rg -n 'pub struct Vec' "$SRC/alloc/src/vec/mod.rs"
rg -n 'read_unaligned' "$SRC/core/src/ptr/mod.rs"
rg -n 'from_raw_parts' "$SRC/core/src/slice/raw.rs"
rg -n 'pub struct AtomicUsize' "$SRC/core/src/sync/atomic.rs"
rg -n 'unsafe auto trait Send' "$SRC/core/src/marker.rs"
rg -n 'pub struct Box' "$SRC/alloc/src/boxed.rs"
```

| C-ID | 去哪找 | 找什么 | 找到后回答 |
|------|-------|-------|-----------|
| C-01…C-04 | `core/src/ops/`、`core/src/mem/`、`core/src/marker.rs` | `Drop` / `replace` / `PhantomData` | 你的视图类型为什么不需要自己写 `Drop`？寿命参数钉住的是哪一块内存？ |
| C-05…C-07 | `core/src/option.rs`、`core/src/iter/traits/` | enum 布局与 `Iterator` 的关联类型 | 报文种类用 enum 还是 `u8` 魔法数？关联类型带寿命时 `dyn` 还在不在？ |
| C-08…C-11 | `core/src/result.rs`、`core/src/iter/`、`alloc/src/boxed.rs`、`alloc/src/vec/mod.rs` | `FromResidual`、`Iterator::next`、`Box<T>` | `?` 在解析函数里转换的是哪两个错误类型？`Box<dyn …>` 的所有权在谁手里？ |
| C-12…C-14 | `core/src/marker.rs`、`core/src/sync/atomic.rs` | `Send`/`Sync`、`AtomicUsize` 的 `Ordering` | 只含原子计数的结构体为什么能自动满足这两个 trait？你有没有自己写 `unsafe impl`？ |
| C-15…C-20 | `core/src/ptr/`、`core/src/slice/raw.rs`、`core/src/mem/` | `add` / `read_unaligned` / `from_raw_parts` / `align_of` | 报文缓冲的起始地址保证按 `u16` 对齐吗？不保证时该走哪条读取路径？ |
| C-21 | `core/src/ffi/`、`core/src/mem/` | `c_int` 一类别名、`size_of`/`offset_of` | 只断言布局、不做 C 调用，和 m6 的双向调用差在哪一条义务？ |
| C-22…C-24 | `core/src/lib.rs`、`alloc/src/lib.rs`、`alloc/src/alloc.rs` | `no_std` / `needs_allocator` / `GlobalAlloc` | 本 crate 是库不是裸机二进制。panic 处理由谁提供？测试里的计数分配器算不算本库的一部分？ |

---

## 3. 你要自己做的实验

**动手写代码或跑命令之前，先把预测填进** [`predictions.md`](./predictions.md) **并提交。**

本模块没有 24 个分散的 example 名。验收命令是整包测试 + 一次 Miri + 一次不启用 std 的构建。
你要自己设计分层文件（任务里点名了几个 `src/*.rs`，那是**文件职责约束**，不是设计方案）。

| C-ID | 实验 | 观察什么 | 断言什么 | 动手前先做 |
|------|-----|---------|---------|-----------|
| C-01…C-04 | `src/buffer.rs` | 拥有缓冲 vs 借用视图；移动拥有者之后视图还能不能用 | 解析结果的载荷是否仍是原切片的子区间（零拷贝）；寿命关系 | 填预测表 #1 #2 |
| C-05…C-07 | `src/parse.rs` | 一种报文种类表示；泛型组合器；**一处** trait 对象 | 合法帧解析成功；未知种类走错误路径 | 填 #3 |
| C-08…C-11 | `src/error.rs`、`src/iter.rs` | `?` 传播；多帧迭代；`Box<dyn …>` | 截断 / 坏魔数等错误路径；迭代在空缓冲上结束；装箱解析器能跑通 | 填 #4 #5 |
| C-12…C-14 | `src/stats.rs` | 原子计数；跨线程（若你的设计允许）或你换成的实现 | `Send`/`Sync` 静态断言；计数终值确定（不要断言线程交错顺序） | 填 #6 |
| C-15…C-20 | `src/raw.rs` | 先检查再读；对外只有安全函数 | 过短切片返回错误而不是读穿；Miri 对安全路径的判定档 | 填 #7 #8 |
| C-21 | `src/wire.rs` | `repr(C)` 头的 `size_of` / `offset_of` | 布局数字（你自己先预测再量）；**不要**把"有 repr(C)"写成"已经做过 FFI" | 填 #9 |
| C-22…C-24 | crate 组织 | `--no-default-features` 能否构建；哪些 API 因缺 OS 而换了 | 构建退出码；分配次数（解析借用视图时是否堆分配） | 填 #10 #11 |
| 全部 | 定位表 | 24 行是否都能填到**文件 + 函数** | 无空行；函数名能在源码里搜到 | 填 `selfcheck.md` 空白表 |

每个 `unsafe` 块旁边要有 Safety 注释，五件事写全；哪一条不适用，写原因。
**范文不在本文件里。**

---

## 4. 提示阶梯

逐级往下看，每次只看一级。**每一级都不给设计方案。**

### 场景怎么切

- **Hint 1（方向）**：先写"输入是一段字节、输出是头 + 载荷视图、失败是枚举"。
  还没到指针。
- **Hint 2（位置）**：对照任务里点名的文件名：`buffer` / `parse` / `error` / `iter` /
  `stats` / `raw` / `wire`。每个文件只承担一类问题。
- **Hint 3（拆解）**：问自己三句：谁拥有字节？谁只是看着？谁被允许做指针运算？
  三句的主语不应该是同一个类型。

### 所有权与寿命

- **Hint 1（方向）**：零拷贝意味着载荷里**没有**新的 `Vec`。
- **Hint 2（位置）**：视图类型上的寿命参数，应该和 `split_at` / `get` 返回的子切片同源。
- **Hint 3（拆解）**：拥有者被 `move` 走之后，原先的视图还在不在？用编译器回答，不要用运行。

### trait / dyn / 泛型

- **Hint 1（方向）**：关联类型一旦提到输入的寿命，`dyn Trait` 往往就做不成。
- **Hint 2（位置）**：把"解析报文头"和"解析完整帧"拆成两个接口，看哪一个还能是对象。
- **Hint 3（拆解）**：泛型组合器接收一个 `FnOnce`，和 `dyn` 那一处，不要放进同一个函数签名里硬凑。

### 缺 OS 的并发

- **Hint 1（方向）**：库代码里不要出现 `std::thread`。
- **Hint 2（位置）**：`core::sync::atomic` 在 `no_std` 里还在。计数放进原子里。
- **Hint 3（拆解）**：真正起线程的代码可以放在 `tests/`（那是另一个 crate，默认有 std）。
  问自己：这算"换实现"还是"把演示挪到测试"？两种你都要在观察记录里写清楚。

### unsafe 与对外安全

- **Hint 1（方向）**：检查在 `unsafe` 之外做完，块内只做"已经合法的那一步"。
- **Hint 2（位置）**：报文缓冲**不**保证按 `u16` 对齐。对照 m5 里"未对齐该走哪条读"。
- **Hint 3（拆解）**：`from_raw_parts` 拼载荷切片之前，长度和起点必须已经是切片的子区间。
  五要素里"对齐"这一条，对 `u8` 切片和对 `u16` 切片答案不同。

### FFI 定位而不冒充

- **Hint 1（方向）**：综合实验允许"产物**或配套说明**"。说明必须写清**为什么**这里不做 C 调用。
- **Hint 2（位置）**：布局断言放在 `wire` 层；真实 syscall / 双向调用已经在 m6。
- **Hint 3（拆解）**：`size_of` 一致不等于"跨语言调用安全"。还缺哪几条义务（谁分配、谁释放、调用约定）？

### no_std 库 vs 裸机二进制

- **Hint 1（方向）**：本 crate 是**库**。最终二进制的 panic 处理在测试运行器或宿主程序里。
- **Hint 2（位置）**：对照 m7：那里是 `no_main` + 自己的处理函数 + 自己的分配器。这里呢？
- **Hint 3（拆解）**：`extern crate alloc` 之后，测试里的全局计数分配器是谁注册的？
  库本身有没有 `#[global_allocator]`？两问分开答。

---

## 5. 打开 Answer Track 的条件

满足**任一**即可，不要更早：

1. §1 的问题已逐条尝试作答，且 `predictions.md` 的预测列已填写**并提交**；
2. 提示阶梯用到 Hint 3 仍无进展；
3. 实验已实际跑过，实测与预测不一致，需要机制解释 ——
   此时**优先只读** `concept.md` 里对应分层的小节，不要整份翻。

打开之后，去 `predictions.md` 的"打开 Answer Track 的记录"里写下原因与卡点。

**打开答案不影响验收。** 验收看的是 `cargo test -p m8-capstone` 退出码、
Miri 判定档、以及 `acceptance/capability-location-map.md` 的 24 行无空缺。

---

## 6. 自检

上面全部做完之后，去 [`selfcheck.md`](./selfcheck.md)。
自检答完再对照 `feynman/m8-capstone.md`。
