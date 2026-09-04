# experiments/ —— 可执行实验：一模块一 crate

**职责**：把每项能力变成可重复执行、可机械判定的产物。

契约：[experiment-contract.md](../specs/001-rust-foundation/contracts/experiment-contract.md)

## 文件布局

```text
experiments/mN-<module>/
├── examples/cNN_<slug>.rs      # 可观察：打印现象。此处输出一律 NON-ASSERTION
├── tests/cNN_<slug>.rs         # 可断言：#[test] 稳定断言。这是验收单位
├── compile_fail/cNN_<case>.rs  # 可选：MUST NOT 编译成功的样本，首行声明期望错误码
└── OBSERVATIONS.md             # 模块级：环境记录 + 实际输出抄录 + 解释
```

## 核心规则：断言与观测**物理隔离**（R-05 / FR-003）

| | 稳定断言 | 非断言输出 |
|---|---------|-----------|
| 位置 | 只能在 `tests/` 的 `#[test]` 中 | 只能在 `examples/` 的 `println!` 与 `OBSERVATIONS.md` |
| 内容 | 确定性事实：`size_of`、分配次数、错误码、UB 类别子串 | 地址、耗时、线程交错、IR 文本、诊断全文 |
| 是否参与一致性判定 | **是**（`cargo test --workspace` 全绿 = SC-002 达成） | 否 |

`#[test]` 断言中 MUST NOT 出现：指针地址值、`{:p}`、时间测量、线程调度顺序、
`HashMap`/`HashSet` 遍历顺序、PID/环境变量、诊断措辞全文。
允许的例外是**地址的关系性质**（`(p as usize) % align_of::<T>() == 0`），不是地址的具体数值。

每个 `#[test]` MUST 带一行 `/// CLAIM:` 说明它验证了什么事实。

## UB 判定（FR-019）

- "程序未崩溃" MUST NOT 作为无 UB 的证据。未跑工具时 `ub_verdict` 只能记 `n/a`，不能记 `clean`。
- `expected-ub` 实验 MUST 在源文件首行用 `//! PREDICT-UB:` 做**事前**类别预测，
  取值来自 §C5.3 的 W1–W11 白名单，且 MUST 在跑 Miri **之前**提交。
- 预测未命中 = **fail**，MUST 保留原预测、书面复盘、按 §Remediation 登记补齐任务，
  然后才允许改写预测。就地把预测改成实际值等于把验收变成抄写工具输出。

## Size budget

单个 example ≤ 80 行。超出即说明实验不够"最小"，MUST 拆分。

## 命令契约（§C8）

| 用途 | 命令 |
|-----|------|
| 观察单个实验 | `cargo run -p mN-<module> --example cNN_<slug>` |
| 验收单个能力 | `cargo test -p mN-<module> --test cNN_<slug>` |
| 验收单个模块 | `cargo test -p mN-<module>` |
| 全量一致性判定 | `cargo test --workspace` |
| UB 判定 | `tools/run-miri.sh mN-<module>` |
| no_std 构建 | `cd experiments/m7-nostd && cargo build` |

> `experiments/m7-nostd` 被根 workspace `exclude`（R-03），**不在** `cargo test --workspace`
> 覆盖范围内。SC-002 的判定口径因此是三项合取，见 spec.md SC-002。
