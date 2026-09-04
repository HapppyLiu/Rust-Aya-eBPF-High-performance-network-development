# Module N —— OBSERVATIONS

<!--
模板：experiment-contract §C3 / §C7。复制为 experiments/mN-<module>/OBSERVATIONS.md 后填写。

本文件里的一切都是 **NON-ASSERTION**：它的差异不参与一致性判定（§C8.1）。
稳定断言在 tests/，两者物理隔离（R-05）。
-->

<!-- 顶部环境块：由 `tools/env-record.sh` 生成，MUST 与 acceptance/environment-baseline.md 一致 -->
## 环境记录

| 字段 | 值 |
|------|-----|
| `rustc_stable` | |
| `rustc_nightly` | |
| `edition` | 2024 |
| `kernel` | |
| `arch` | |
| `target` | |
| `command` | |

> 基线：[../../acceptance/environment-baseline.md](../../acceptance/environment-baseline.md)

---

## 记录块

<!--
每个 example 一块。格式固定，三个字段都是 REQUIRED。

`解释` 的**内容下限**（§C3.2）—— MUST 同时回答两问，缺任一问即该条未完成：
  1. **为什么会这样？** 落到机制（编译器决策 / 硬件行为 / 运行时结构），不是现象复述。
  2. **这不能证明什么？** 指出证据边界，哪些结论**不能**由它推出。

仅复述现象的写法 MUST 拒绝，例如"输出为 2，说明程序运行正常"。

`架构相关性` 对**全部**记录块强制（§C7.2），取值三选一且每种都要写理由：
  可跨架构推广 / 仅适用于 <arch> / 未知，需实测
校验：`grep -c '架构相关性：' OBSERVATIONS.md` MUST 等于记录块数量。
-->

### C-NN / cNN_<slug>  [NON-ASSERTION]

命令：`cargo run -p mN-<module> --example cNN_<slug>`

输出：

```text
（原样抄录）
```

解释：
  为什么会这样：
  这不能证明什么：

架构相关性：<可跨架构推广 / 仅适用于 x86_64 / 未知，需实测>。<理由>

---

## 编译器诊断抄录（compile_fail 样本）

<!-- §C4.3：完整 stderr 落盘在 target/compile-fail/<case>.stderr，此处抄录。
     MUST NOT 对 stderr 全文做相等比较 —— 断言对象是错误码。 -->

### cNN_<case>  [NON-ASSERTION]

期望错误码（样本首行 `//! EXPECT:` 声明）：
实际错误码（`rf_harness` 提取）：

```text
（stderr 抄录）
```

解释：
  为什么会这样：
  这不能证明什么：

架构相关性：

---

## UB 判定记录

<!-- §C5。未运行工具时 `ub_verdict` 只能记 `n/a`，MUST NOT 记 `clean`（FR-019）。 -->

| 实验 | 事前预测（`PREDICT-UB`） | 工具与命令 | 实际类别 | `ub_verdict` | 命中? |
|------|----------------------|-----------|---------|-------------|-------|
| | | | | | |

<!-- 未命中（PREDICTION-MISS）时 MUST 执行三步（§C5.1a），MUST NOT 就地改预测：
       1. 保留原预测并抄录实际类别，标注 PREDICTION-MISS
       2. 书面回答：我原以为会触发哪条规则 / 实际触发的是哪条 / 心智模型错在哪一步
       3. 按 tasks.md §Remediation 追加补齐任务，然后才允许改写 PREDICT-UB -->

---

## IR 观察

<!-- §C3.3：MIR / LLVM IR 文本一律 NON-ASSERTION。
     IR 中可断言的部分 MUST 先转化为确定性量（size_of / 分配计数 / 单态化实例数）再写进 tests/。 -->

### <观察目标>  [NON-ASSERTION]

命令：`tools/emit-mir.sh mN-<module> --example cNN_<slug>`

摘录：

```text
（只取相关函数体，不要贴整个文件）
```

解释：
  为什么会这样：
  这不能证明什么：

架构相关性：
