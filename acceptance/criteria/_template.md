# C-NN <Capability 名> — Acceptance Criterion

<!--
模板：learning-artifact-contract §D。复制本文件为 cNN.md 后填写，删除本注释。

§D1 禁用措辞（出现即该条判据作废）：
    看过 / 了解过 / 做过笔记 / 熟悉 / 基本掌握 / 有印象 / 大致清楚
合格谓词（Constitution V）：
    能解释 / 能画图 / 能写代码 / 能运行实验 / 能分析输出 / 能定位源码 / 能完成测试

§D2：**至少一条判据 MUST 由命令退出码决定** —— 验收不能完全依赖自我评估。
-->

**Module**: mN | **Story**: USn | **UB tool**: miri / asan / compile-time / n/a

## 验证命令                     <!-- REQUIRED，MUST 可直接复制执行 -->

```bash
cargo test -p mN-<module> --test cNN_<slug>
# 若适用：
tools/run-miri.sh mN-<module>
```

## 通过判据                     <!-- REQUIRED，MUST 可观测 -->

- [ ] 上述命令全部退出码为 0 ← **这一条由退出码决定（§D2）**
- [ ] <能力特有的可观测判据。写成"能做什么并且结果可核对"的形式，例如：
      "能在**不运行程序**的前提下预测 compile_fail 样本的错误码，
       预测与 `rf_harness` 打印的实际错误码一致">
- [ ] 源码引用已记录（路径 + 符号 + **实际行号** + "这段源码回答了什么"）
- [ ] （`UB Tool ≠ n/a` 时）`ub_verdict` 实际值与事前 `PREDICT-UB` 一致
- [ ] （unsafe 适用时）每个 unsafe 块的 SAFETY 注释**实质**覆盖五要素：
      有效性 / 对齐 / 别名 / provenance / 生命周期，不适用项写明原因

## 结果

**Status**: not-evaluated / pass / fail

> 本字段记的是**这条 AC 自身**的判定，不是 Capability 的状态。
> Capability 状态的唯一事实源是 `acceptance/capability-matrix.md`（§C1a）。
> 模块 Feynman fail 时，这里仍可如实记 `pass`，但矩阵中该能力停在 `experiment-passed`。

**评估日期**：YYYY-MM-DD
**环境记录**：[../../experiments/mN-<module>/OBSERVATIONS.md](../../experiments/mN-<module>/OBSERVATIONS.md)
