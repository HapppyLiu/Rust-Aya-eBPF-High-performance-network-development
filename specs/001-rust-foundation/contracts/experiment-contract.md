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
Spec Edge Case 规定"仅记录现象者视为未完成"：

```markdown
### C-18 / c18_alignment  [NON-ASSERTION]

命令：`cargo run -p m5-unsafe --example c18_alignment`

输出：
    base = 0x7ffd3a1c2b40   (地址每次运行不同)
    misaligned read = 7

解释：x86_64 硬件容忍未对齐访问，因此程序正常输出；这**不构成**无 UB 的证据。
      同一源码在 Miri 下判定为 UB（见 tests/c18_alignment.rs 的 expected-ub 断言）。
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

**规则 C5.3**：`expected-ub` 类实验的稳定断言是 **Miri 错误类别文本的子串**（稳定），
而非完整报告（含分配编号 `alloc159`、行号，均可变）：

```rust
/// CLAIM: 未对齐的 8 字节写入越出了 u64 分配边界，Miri 判定为 UB。
#[test]
fn misaligned_write_is_ub_under_miri() {
    let out = rf_harness::miri::run_example("c18_alignment_ub");
    assert!(out.reported_ub());
    assert!(out.stderr_contains("Undefined Behavior"));
    assert!(out.stderr_contains("memory access failed"));
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

**规则 C7.2**：当实验结果依赖架构或编译器版本时，MUST 附一行**可推广性判定**：
`可跨架构推广` / `仅适用于 x86_64，原因：…`。

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
