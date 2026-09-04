# Contract: Experiment Artifact

**Feature**: 001-rust-foundation | **Consumers**: 实施者（`/speckit-implement`）、验收者、后续 Feature

本契约定义一个"实验"在仓库中的合法形态。**不满足本契约的产物 MUST NOT 被标记为完成。**
契约的核心目的是让 FR-003（稳定断言）与 FR-019（UB 判定）成为**可机械检查**的属性。

---

## C1. 文件布局

每个 Capability `C-NN` 在其模块 crate 内 MUST 拥有：

```text
experiments/mN-<module>/
├── examples/cNN_<slug>.rs      # 可观察实验：打印现象。此处输出一律为 NON-ASSERTION
├── tests/cNN_<slug>.rs         # 可断言实验：#[test] 稳定断言。这是验收单位
├── compile_fail/cNN_<case>.rs  # 可选：MUST NOT 编译成功的样本
└── OBSERVATIONS.md             # 模块级：抄录实际输出 + 环境记录
```

命名 MUST 与 plan.md `Capability Gate Matrix` 的 **Exp** 列一致，使 grep `c18_` 即可取出
C-18 的全部产物。

**Size budget**：单个 example ≤ 80 行。超出即说明实验不够"最小"，MUST 拆分。

---

## C2. 稳定断言（FR-003 的强制形式）

**规则 C2.1**：稳定断言 MUST 且只能出现在 `#[test]` 函数中。

**规则 C2.2**：`#[test]` 断言表达式中 MUST NOT 出现下列任一内容：

| 禁止内容 | 典型形式 |
|---------|---------|
| 指针地址值 | `{:p}`、`as usize` 后的地址比较、`ptr as u64` |
| 时间测量 | `Instant::now()`、`Duration` 比较 |
| 线程调度顺序 | 依赖线程完成顺序的结果比较 |
| 迭代顺序不稳定的容器 | `HashMap`/`HashSet` 的遍历顺序 |
| 进程/环境相关值 | PID、环境变量、临时路径 |
| 诊断措辞全文 | 完整 stderr 字符串比对（改用错误码，见 C4） |

> 允许的例外：**地址的关系性质**（如 `p.align_offset(8) == 0`、`a != b`、
> `(p as usize) % align_of::<T>() == 0`）是确定性的，可以断言；**地址的具体数值**不可以。

**规则 C2.3**：每个 `#[test]` 函数 MUST 带一行说明该断言验证了什么事实：

```rust
/// CLAIM: `Option<&T>` 借助 niche 优化与 `&T` 同宽，因此空指针不是合法的 `&T`。
#[test]
fn option_ref_has_no_discriminant_overhead() {
    assert_eq!(size_of::<Option<&u8>>(), size_of::<&u8>());
}
```

---

## C3. 非断言输出

**规则 C3.1**：`examples/` 的全部 `println!` 输出均视为 NON-ASSERTION。

**规则 C3.2**：输出抄录到 `OBSERVATIONS.md` 时 MUST 使用如下块，且 `解释` 字段非空 ——
Spec Edge Case 规定"仅记录现象者视为未完成"。

> **内容下限（T001-c / CHK017）**：`解释` 字段 MUST 同时回答下列**两问**，缺任一问即判该条
> NonAssertionOutput **未完成**：
>
> 1. **为什么会这样？** —— 指出产生该现象的机制（编译器决策、硬件行为、运行时结构），
>    MUST 落到机制层而非现象层。
> 2. **这不能证明什么？** —— 指出该观测的**证据边界**，即哪些结论**不能**由它推出。
>    典型形式："程序正常退出不能证明无 UB"、"x86_64 上未崩溃不能证明该访问合法"、
>    "IR 中出现直接调用不能证明所有调用点都被单态化"。
>
> 判定方式：逐条阅读 `解释` 字段，若无法指出它分别对应上述哪一问，即为仅复述现象，判未完成。
> 复述现象的典型形式（MUST 拒绝）："输出为 2，说明程序运行正常"、"打印顺序是 b 先于 a"。

```markdown
### C-18 / c18_alignment  [NON-ASSERTION]

命令：`cargo run -p m5-unsafe --example c18_alignment`

输出：
    base = 0x7ffd3a1c2b40   (地址每次运行不同)
    misaligned read = 7

解释：
  为什么会这样：x86_64 的 MOV 指令在硬件层面容忍未对齐地址，CPU 自动拆分为多次访问，
    因此这段源码在本机不会产生任何可见故障，照常打印 7。
  这不能证明什么：**不能**证明该访问没有 UB。UB 是 Rust 抽象机层面的判定，与本次运行
    是否崩溃无关。同一源码在 Miri 下判定为 UB（见 tests/c18_alignment_ub.rs 的
    expected-ub 断言）。也**不能**证明换一个优化等级或编译器版本后仍会打印 7。
架构相关性：本结果依赖 x86_64；在 aarch64 上可能触发 SIGBUS。不可跨架构推广。
```

**规则 C3.3**：MIR / LLVM IR 文本一律为 NON-ASSERTION。IR 中可断言的部分 MUST 先转化为
确定性量（`size_of`、分配计数、单态化实例数）再写进 `tests/`。

---

## C4. 编译失败实验

**规则 C4.1**：`compile_fail/cNN_<case>.rs` 的首行 MUST 用注释声明期望的错误码：

```rust
//! EXPECT: E0499
//! CLAIM: 同一作用域内不能对同一值取两次可变借用。
```

**规则 C4.2**：验证由 `tests/` 中的 `rf_harness::compile_fail::expect_errors` 执行，
断言对象是**错误码**而非措辞：

```rust
#[test]
fn two_mut_borrows_rejected() {
    rf_harness::compile_fail::expect_errors("compile_fail/c03_two_mut_borrows.rs", &["E0499"]);
}
```

**规则 C4.3**：完整 stderr 落盘到 `target/compile-fail/<case>.stderr` 并抄录进
`OBSERVATIONS.md`，标注 NON-ASSERTION。**MUST NOT** 对 stderr 全文做相等比较。

**规则 C4.4（多错误码，T001-e / CHK020）**：一个 `compile_fail/` 样本触发**多个**错误码时：

1. 断言语义固定为**子集关系**：`expected_codes ⊆ actual_codes`。
   `expect_errors` MUST NOT 被解释为"错误码集合相等"，因为 rustc 可能追加派生诊断
   （如 `E0499` 之后的 `E0502`），追加项不构成失败。
2. 首行 `//! EXPECT:` MUST **逐一列出全部实际出现的错误码**，而非只写"最想要的那一个"。
   顺序按 rustc 首次报出的顺序。这防止学习者用一个宽泛样本蒙对一个错误码。
3. `//! CLAIM:` MUST 说明**每个**列出的错误码各自对应哪一条规则；若某个错误码是前一个错误的
   派生结果，MUST 写明它派生自哪一条。
4. 若实际出现的错误码**多于**首行声明，判该样本 **fail**：说明样本不够最小，
   MUST 缩小样本或补全声明（二选一，并在 OBSERVATIONS 中记录选择理由）。

```rust
//! EXPECT: E0499, E0502
//! CLAIM: 第 5 行对 v 取第二个 &mut 触发 E0499（同一作用域两个可变借用）；
//!        第 7 行的 &v 在可变借用存活期内触发 E0502（可变与不可变借用冲突）。
```

对应的断言写法（子集语义，两个码都 MUST 列出）：

```rust
rf_harness::compile_fail::expect_errors(
    "compile_fail/c03_two_mut_borrows.rs",
    &["E0499", "E0502"],
);
```

---

## C5. UB 实验（FR-019）

**规则 C5.1**：每个 `unsafe_applicable` 的实验 MUST 声明 `ub_verdict` 的**预期值**，
并用对应工具实际取得。

| 实验意图 | 预期 `ub_verdict` | 通过条件 |
|---------|------------------|---------|
| 演示安全抽象正确 | `clean` | Miri 无 UB 报告 |
| 演示 UB（教学用） | `expected-ub` | Miri **报告** UB，且错误类别与预测一致 |

**规则 C5.2**：`ub_verdict` MUST NOT 在未运行 UB 工具的情况下被填为 `clean`。
未运行时只能填 `n/a`。"程序未崩溃"或"输出符合预期" MUST NOT 作为无 UB 的证据。

**规则 C5.1a（事前预测，T001-a / CHK027）**：`expected-ub` 类实验的 `ub_verdict` 预期值
MUST 附带一条**事前 UB 类别预测**，写在实验源文件首行的 `//! PREDICT-UB:` 中，
取值 MUST 来自 C5.3 的白名单，且 MUST 在运行 Miri **之前**提交到版本控制。

判定规则（三种结果，无第四种）：

| Miri 实际报告 | 与事前预测的关系 | 判定 | 后续动作 |
|--------------|----------------|------|---------|
| 报告 UB | 类别**命中**预测 | **pass**，`ub_verdict = expected-ub` | 正常推进 |
| 报告 UB | 类别**未命中**预测 | **fail**，`ub_verdict = unexpected-ub` | 见下方 |
| 未报告 UB | — | **fail**，`ub_verdict = clean`（与 `expected-ub` 意图矛盾） | 实验构造无效，MUST 重写实验 |

"类别未命中"时 MUST 执行下列全部三步，MUST NOT 就地把预测改成实际值了事：

1. 在 `OBSERVATIONS.md` 中**保留原预测**并抄录实际类别，标注 `PREDICTION-MISS`；
2. 书面回答"我原本以为会触发哪条规则、实际触发的是哪条、我的心智模型错在哪一步"；
3. 按 tasks.md §Remediation 追加一条补齐任务，然后才允许改写 `//! PREDICT-UB:`。

**理由**：预测未命中意味着心智模型有缺口，而这正是本 Feature 要暴露的东西。允许事后改预测
等于把验收变成"抄写 Miri 输出"，FR-019 的立法意图会被完全架空。

**规则 C5.3**：`expected-ub` 类实验的稳定断言是 **Miri 错误类别文本的子串**（稳定），
而非完整报告（含分配编号 `alloc159`、行号，均可变）。

**允许的子串白名单（T001-b / CHK026）**：`stderr_contains` 的实参 MUST 取自下表。
表外字符串 MUST NOT 用作稳定断言 —— 这条规则的目的是禁止"跑完 Miri 再从输出里任选一段
文本回填断言"，那样断言恒真、验收恒过。

| # | 允许的子串 | 对应 UB 类别 | 典型触发 |
|---|-----------|-------------|---------|
| W1 | `Undefined Behavior` | 通用类别前缀（**每条 expected-ub 断言 MUST 包含它**） | 全部 |
| W2 | `memory access failed` | 越界 / 已释放内存访问 | C-15、C-17、C-20 |
| W3 | `attempting a read access` | 读越界（与 W2 联合出现） | C-16、C-20 |
| W4 | `attempting a write access` | 写越界（与 W2 联合出现） | C-17、C-20 |
| W5 | `not sufficiently aligned` | 对齐违规 | C-18 |
| W6 | `has been freed` | use-after-free | C-16 |
| W7 | `out-of-bounds pointer arithmetic` | 指针运算越出分配 | C-17 |
| W8 | `does not exist in the borrow stack` | Stacked Borrows 别名违规 | C-19 |
| W9 | `/tag-mismatch|protected tag/`（Tree Borrows 二选一） | Tree Borrows 别名违规 | C-19 |
| W10 | `uninitialized memory` | 未初始化读取 | C-20 |
| W11 | `Data race detected` | 数据竞争 | C-13、C-14 |

**规则 C5.3a**：每条 `expected-ub` 断言 MUST 由 **W1 + 至少一条 W2–W11** 组成。
只断言 W1 不合格 —— `Undefined Behavior` 对所有 UB 都成立，单独使用等于没有预测类别。

**规则 C5.3b**：白名单的扩充是**契约修订**，MUST 先改本表并说明该类别对应哪条 UB 规则，
然后才允许在实验中使用；MUST NOT 在实验源码中就地引入表外子串。

**规则 C5.3c**：`//! PREDICT-UB:` 声明的类别 = 该实验将要断言的 W 编号集合。
C5.1a 的"命中/未命中"即以此集合与实际 stderr 的匹配结果判定。

```rust
/// CLAIM: 未对齐的 8 字节写入越出了 u64 分配边界，Miri 判定为 UB。
/// PREDICT-UB: W1 + W2（事前预测，见 examples/c18_alignment_ub.rs 首行）
#[test]
fn misaligned_write_is_ub_under_miri() {
    let out = rf_harness::miri::run_example("c18_alignment_ub");
    assert!(out.reported_ub());
    assert!(out.stderr_contains("Undefined Behavior")); // W1，必需
    assert!(out.stderr_contains("memory access failed")); // W2，类别
    // 非断言：alloc 编号、字节偏移、行号
}
```

**规则 C5.4**：FFI 实验（C-21）因 Miri 无法执行真实 C 调用，改用 ASan 判定；
`ub_verdict` 的取得命令 MUST 记录在环境记录中。

---

## C6. Safety Invariant（Constitution VI / FR-008）

**规则 C6.1**：每个 `unsafe` 块 MUST 紧邻一条 `// SAFETY:` 注释。

**规则 C6.2**：`// SAFETY:` MUST 逐项覆盖下列**所有适用**要素，不适用项显式写"不适用及原因"：

1. **有效性（validity）**——指针指向的内存已分配且未释放，长度足够；
2. **对齐（alignment）**——满足 `align_of::<T>()`；
3. **别名（aliasing）**——此期间不存在冲突的其他引用；
4. **provenance**——指针来自哪个分配，未越出该分配的可访问范围；
5. **生命周期（lifetime）**——被引用数据的存活期覆盖使用期。

**规则 C6.3**：`// SAFETY: Rust 要求 unsafe` 或同类空洞说明 MUST 被拒绝。
无法陈述不变量的 unsafe 代码 MUST 被重写或删除。

**规则 C6.4**：clippy 配置启用 `undocumented_unsafe_blocks` 与 `multiple_unsafe_ops_per_block`
作为机械兜底，人工审查负责判断内容是否**实质**覆盖 C6.2 五要素。

---

## C7. 环境记录（FR-010 / FR-018）

**规则 C7.1**：每个模块的 `OBSERVATIONS.md` 顶部 MUST 有由 `tools/env-record.sh` 生成的环境块，
字段见 [data-model.md](../data-model.md) §9。

**规则 C7.2（T001-d / CHK019）**：**可推广性判定对全部实验强制**，无一例外。
每个 NonAssertionOutput 记录块 MUST 有且仅有一行 `架构相关性：`。

这里的强制项是**"作出判定"这个动作本身**，而不是"结果必须架构相关"。是否敏感由实验作者
判断并给出理由 —— 把判定权交给作者、把"必须表态"设为硬性要求，才能避免"没想过架构问题"
与"想过并认定无关"这两种情况在产物上长得一模一样。

取值 MUST 为下列三者之一，且**每一种都 MUST 附理由**：

| 取值 | 含义 | 理由 MUST 说明 |
|-----|------|--------------|
| `可跨架构推广` | 结果由语言语义或类型系统决定 | 依据哪条语义规则得出与硬件无关的结论 |
| `仅适用于 <arch>` | 结果依赖本机硬件行为 | 依赖哪个硬件特性，换架构后预期如何变化 |
| `未知，需实测` | 作者无法判断 | 需要什么实验才能判定；MUST 同时按 §Remediation 登记补齐任务 |

示例（三类各一）：

```text
架构相关性：可跨架构推广。size_of/align_of 由 Rust 类型布局规则与 target 的数据模型决定，
            本断言只依赖 `Option<&T>` 的 niche 优化这一语言层保证，与指令集无关。
架构相关性：仅适用于 x86_64。依赖 x86_64 对未对齐访问的硬件容忍；aarch64 上同一代码
            可能触发 SIGBUS，因此"程序正常退出"这一现象不可跨架构推广。
架构相关性：未知，需实测。放宽为 Relaxed 后本机未观察到重排，但 x86_64 是强序模型，
            无法据此判断 aarch64 行为。已按 §Remediation 登记 aarch64 对照实验。
```

**判定方式**：`grep -c '架构相关性：' OBSERVATIONS.md` 的结果 MUST 等于该文件中
NON-ASSERTION 记录块的数量。缺行即该模块 OBSERVATIONS 未完成。

---

## C8. 命令契约

每个实验 MUST 可通过下列命令之一独立执行（"独立可运行"的定义）：

| 用途 | 命令 |
|-----|------|
| 观察单个实验 | `cargo run -p mN-<module> --example cNN_<slug>` |
| 验收单个能力 | `cargo test -p mN-<module> --test cNN_<slug>` |
| 验收单个模块 | `cargo test -p mN-<module>` |
| 全量一致性判定 | `cargo test --workspace` |
| UB 判定（纯 Rust） | `cargo +nightly miri test -p mN-<module>` |
| UB 判定（并发） | `MIRIFLAGS="-Zmiri-many-seeds" cargo +nightly miri test -p m4-concurrency` |
| UB 判定（FFI） | `tools/run-asan.sh m6-ffi` |
| no_std 构建 | `cd experiments/m7-nostd && cargo build` |

**规则 C8.1**：`cargo test --workspace` 全绿 = 全部稳定断言复现（SC-002 的判定方式）。
`OBSERVATIONS.md` 的差异不参与该判定。
