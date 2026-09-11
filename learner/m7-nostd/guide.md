# Learner Track — Module 7: no_std、运行时边界与分配器

**Story**: US7（P1，Feature 002 硬前置） | **Capabilities**: C-22…C-24 | **Prerequisite**: m6（已 accepted）

> 本文件属于 **Learner Track**，不含答案。
> Answer Track 在 `learning/m7-nostd/`、`feynman/m7-nostd.md` 与 `experiments/m7-nostd/`。
> 什么时候可以翻，见 §5。

## 0. 开始之前

- **前置模块**：m6（C-21），当前验收状态：查 `acceptance/capability-matrix.md`。
- **本模块假定你已经能**：
  - 说清跨语言边界上谁分配、谁释放，以及约定拆掉时的故障形态（C-21）；
  - 给 `unsafe` 块写出有效性 / 对齐 / 别名 / 来源 / 生命周期五要素（C-15）；
  - 把"本机没崩"和"语言合法"拆开（C-20）。
- **本模块结束时你应该能**：独立把一个 `no_std` 最小产物**构建成功**
  （本模块的"可运行"就是构建成功：裸机 target 没有可返回的操作系统，产物不能在本机直接执行）；
  对三步递进里固定清单的每一条错误，指出缺失的是七类边界中的哪一层，而不是笼统说"缺库"；
  说明堆分配能力由谁提供，以及这个前提在后续受限环境里是否成立。

**为什么排在 m6 之后**：要解释"缺的是哪一层运行时服务"，你得先知道 ABI 与
"谁提供一块内存"不是同一件事。分配器实现几乎必然碰到 `unsafe` 与裸指针。

**本模块的工作方式**：crate 被根 workspace 排除，必须进目录单独构建，target 由
`.cargo/config.toml` 钉死。默认构建应当成功。然后按 quickstart §6 把三步递进
**每一步单独构建**，让该步的错误完整暴露（编译器遇到某些错误会提前中止，只报首错）。
稳定断言在这里是**编译期判定、构建退出码、产物静态检查**，外加 host 侧对自实现分配器的检查。
**先填预测表，再跑。** 先跑再填等于没预测。

---

## 1. 本模块你要能回答的问题

先自己答一遍。答不上来是正常的 —— 记下来，那就是你这个模块的学习目标。

### C-22 no_std

1. 给 crate 加上 `#![no_std]`，编译器不再自动拉进来的是哪一件东西？
   是"语言里所有带 `std::` 前缀的名字"，还是"默认的那个 crate 及其依赖的运行时服务"？
2. 加上它之后，`Option`、`Result`、切片、迭代器为什么往往还在？它们住在哪一层？
3. 裸机 target 上，编译器还会向你要哪一类**语言项**（不是"某个日常 API"）？
   不提供时，缺的是库代码，还是运行时约定？
4. 产物不能在本机直接执行，本模块的 Independent Test 还算不算通过？
   "可运行"在这里被定义成什么？

### C-23 core / alloc / std

1. `Vec` 的实现写在哪一层？你在普通程序里写 `std::vec::Vec` 时，名字是从哪再导出的？
2. 同一段"把一组 `i32` 加起来"的逻辑，分别只使用 core、再加上 alloc、再试图加上 std，
   在本模块的裸机 target 上，哪一种能构建、哪一种不能？不能的那一种缺的是哪一类能力？
3. `String` 和 `File` 是不是同一层的东西？若不是，拆开的那条缝在哪？
4. 把失败笼统写成"不能用标准库"，为什么本模块的验收会判未通过？
   七类边界（core / alloc / std / allocator / panic / runtime / OS services）各自管什么？

### C-24 Panic and allocator fundamentals

1. `extern crate alloc` 之后，`Vec::new()` 就能用了吗？若不能，还缺谁？
   分配能力是 alloc crate 自带的，还是某份 `GlobalAlloc` 实现提供的？
2. 你自己用一块静态数组做 bump 分配器，让 `Vec` 在裸机产物里链上。
   这件事证明了"任何受限环境只要抄这份分配器就能堆分配"吗？
   后续 eBPF 那种环境里，这个前提成不成立？
3. `#[panic_handler]` 必须保证什么（返回？展开？永远不回来？）？
   有操作系统时 panic 的默认故事，和没有操作系统时，差在哪一步？
4. 本模块的分配器实现是 `unsafe` 的。五要素里哪几条对 bump 分配器特别容易漏？
   host 侧的检查工具和裸机产物分别能证明什么、不能证明什么？

---

## 2. 你要自己定位的源码

给的是**搜索范围**和**要找什么**，行号要你自己填。

```bash
SRC="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
rg -n 'no_std|no_core' "$SRC/core/src/lib.rs" "$SRC/std/src/lib.rs" "$SRC/alloc/src/lib.rs"
rg -n 'panic_impl|panic_handler' "$SRC/core/src/panicking.rs"
rg -n 'GlobalAlloc' "$SRC/core/src/alloc/global.rs"
rg -n '__rust_alloc|handle_alloc_error' "$SRC/alloc/src/alloc.rs"
```

| C-ID | 去哪找 | 找什么 | 找到后回答 |
|------|-------|-------|-----------|
| C-22 | `core/src/lib.rs` | crate 顶部的属性：它声明自己是哪一层？有没有 `no_std` 这个词，若没有，用的是更底层的哪一个？文档里点名要消费方提供哪些符号？ | 这些符号是"库 API"还是"语言项 / 运行时约定"？ |
| C-22 | `std/src/lib.rs` | 同一类属性在 std 自己身上出现了没有？附近的注释怎么解释"我们自己是谁"？ | 若 std 自己也写了某种 `no_*` 属性，这对" `no_std` = 不能碰标准库"这句话意味着什么？ |
| C-23 | `alloc/src/lib.rs` | crate 文档怎么描述自己和 `std`、和 `#![no_std]` crate 的关系？有没有 `needs_allocator` 一类的属性？ | alloc 提供集合类型，还是提供那块实际的堆？ |
| C-23 | `std/src/lib.rs` | `extern crate alloc` 以及随后对 `vec` / `string` / `boxed` 的再导出 | `std::vec::Vec` 的实现落在哪一层？ |
| C-24 | `core/src/panicking.rs` | 文档说 core 能不能定义 panic 处理；真正漏斗进哪一个语言项；`#[panic_handler]` 和那个语言项的关系 | 不提供处理函数时，缺的是 core 里的某段代码，还是消费方必须供给的入口？ |
| C-24 | `core/src/alloc/global.rs` | `GlobalAlloc` 这个 trait 的 `alloc` / `dealloc` 签名与 Safety 段 | 返回空指针和触发处理函数，各自表示什么？size 为 0 合法吗？ |
| C-24 | `alloc/src/alloc.rs` | `__rust_alloc` 这类符号从哪来（有 `#[global_allocator]` 时 / 没有时）；`handle_alloc_error` 在链接了 std 与纯 `no_std` 时默认分别做什么 | 没有全局分配器时，失败出现在编译、链接，还是第一次分配？ |

---

## 3. 你要自己做的实验

**动手写代码或跑命令之前，先把预测填进** [`predictions.md`](./predictions.md) **并提交。**

| C-ID | 实验 | 观察什么 | 断言什么 | 动手前先做 |
|------|-----|---------|---------|-----------|
| C-22 | `c22_nostd` 默认构建 | `cd experiments/m7-nostd && cargo build` 的退出码；构建脚本是否拒绝非裸机 target | **构建是否成功**（成功与否是断言对象；不要断言诊断全文） | 填预测表 #0 |
| C-22 | 产物静态检查 | `nm` / `readelf -h` 里有没有 C 启动例程、有没有展开人格符号、入口叫什么、ELF 头的机器与类别 | 脚本退出码；你事先预测的"有 / 无"是否被脚本同意 | 填 #7 |
| C-22 / C-24 | 三步递进 · 步骤 1 | 两份**单独**构建：拿掉 panic 处理；以及不走 abort 策略 | 每一条计入分母的错误：**归属哪一层**（七类里选，不要写笼统的"缺库"） | 填 #1 #2 |
| C-23 | `c23_core_alloc_std` | 同一段求和：core 版、alloc 版在默认构建里是否都链上；std 版在裸机 target 上的构建结果 | 哪一版成功、哪一版失败；失败归属哪一层 | 填 #3 #4，并对照默认构建 |
| C-23 / C-24 | 三步递进 · 步骤 2 / 3 | 引入 std 专属类型；让 crate 去找 std；用 `Vec` 但不引入 alloc crate；引入 alloc 但不提供全局分配器。每步单独构建 | 清单上剩下四条的归属层 | 填 #3 #4 #5 #6 |
| C-24 | `c24_panic_alloc` | 默认构建里 `Vec` 能否链接；分配器是 bump 静态数组 | 构建成功本身；"分配能力由谁提供"你写在观察记录里的那句 | 填 #8 |
| C-24 | host 侧分配器 | 不在裸机上跑，而在 host 上直接调用 bump 的 `alloc`/`dealloc` | 检查脚本 / `cargo +nightly miri test` 的退出码；把结论写成工具允许的哪一档（不要把"没跑"写成"干净"） | 填 #9 |

每个 `unsafe` 块旁边要有 Safety 注释，五件事写全；哪一条不适用，写原因。
**范文不在本文件里。**

---

## 4. 提示阶梯

逐级往下看，每次只看一级。**每一级都不给答案。**

### C-22 no_std

- **Hint 1（方向）**：问的不是"标准库三个字还能不能出现"，而是"默认 crate 以及它依赖的操作系统服务还在不在"。
- **Hint 2（位置）**：对照 `core/src/lib.rs` 与 `std/src/lib.rs` 的 crate 属性。
  给自己的 crate 加的那个属性，和 std **自己**身上的属性，列成两行再比较。
- **Hint 3（拆解）**：把"语言原语（Option、切片）"和"文件 / 线程 / 环境变量"拆成两句。
  前者走哪一层就能用？后者缺的是 API 名字，还是实现这些 API 所需的内核？

### C-23 三层 crate

- **Hint 1（方向）**：`std::vec::Vec` 这个路径不等于"实现写在 std 里"。去找再导出。
- **Hint 2（位置）**：`alloc/src/lib.rs` 顶部文档 + `std/src/lib.rs` 里对 `vec` 的 `pub use`。
  然后看 `String`（堆上的字符串）和 `File`（打开一个路径）还能不能放进同一层。
- **Hint 3（拆解）**：同一段求和，core 版只碰切片；alloc 版把数字 `push` 进 `Vec`；std 版去用一个需要操作系统的类型。
  三份构建的失败，不会是同一个原因。

### C-24 panic 与分配器

- **Hint 1（方向）**：alloc crate 是"会说话的集合"，不是"一块堆"。说话要找人，那个人是 `GlobalAlloc`。
- **Hint 2（位置）**：`core/src/panicking.rs` 看漏斗；`alloc/src/alloc.rs` 看 `__rust_alloc` 在有无
  `#[global_allocator]` 时分别由谁生成。默认产物里用一块静态数组实现 bump，让 `Vec` 链上。
- **Hint 3（拆解）**：把"链接成功"和"这块分配器可以搬到任意受限环境"写成两句。
  后一句需要哪类运行时服务（线程、页分配、系统调用）是 bump 静态数组没有、而那些环境也没有的？

### 三步递进（SC-006 清单）

- **Hint 1（方向）**：每一步单独构建。不要在同一次调用里叠三种破坏，否则分母会被"只报首错"打乱。
- **Hint 2（位置）**：六条构造写在 `predictions.md` #1–#6，命令在表里。归属从七类里选一层，
  并准备一句机制，而不是三个字母碰运气。
- **Hint 3（拆解）**：若你想写"不能用标准库"，先停下来，改写成"缺的是语言项 / 堆实现 / 操作系统服务中的哪一个"。
  笼统归因即使层次标签碰巧写对，本模块也判未通过。

### 怎么自己写 Safety 五要素（分配器）

- **Hint 1（方向）**：bump 从一块静态缓冲里切区间。有效性是"切出来的范围还在这块缓冲里"；
  别名是"两块已分配区间不许重叠"。
- **Hint 2（位置）**：对着 `alloc` 里那一次指针加法，逐项问：基址从哪来、对齐怎么凑、
  并发两个分配怎么避免切到同一段、指针还记不记得这块静态数组、释放是真还是空操作。
- **Hint 3（拆解）**：`dealloc` 若什么都不做，生命周期那一条怎么写才不算撒谎？
  写完用 `cd experiments/m7-nostd && cargo clippy -- -D warnings` 做机械兜底。

---

## 5. 打开 Answer Track 的条件

满足**任一**即可，不要更早：

1. §1 的问题已逐条尝试作答，且 `predictions.md` 的预测列已填写**并提交**；
2. 提示阶梯用到 Hint 3 仍无进展；
3. 实验已实际跑过，实测与预测不一致，需要机制解释 ——
   此时**优先只读该能力对应的** `concept.md` **小节**，不要整份翻。

打开之后，去 `predictions.md` 的"打开 Answer Track 的记录"里写下原因与卡点。

**打开答案不影响验收。** 验收看的是 `acceptance/criteria/c22.md`、`c23.md`、`c24.md`
的客观判据（命令退出码 + 可观测判据），不看你是不是自己想出来的。

---

## 6. 自检

上面全部做完之后，去 [`selfcheck.md`](./selfcheck.md)。
自检答完再对照 `feynman/m7-nostd.md`。
