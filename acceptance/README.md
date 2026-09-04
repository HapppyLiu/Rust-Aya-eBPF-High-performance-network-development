# acceptance/ —— 验收（双轨汇合点）

**职责**：Constitution V 的落点。验收标准对两条轨道中立 —— 它既不是问题也不是答案，
而是"怎么判定学会了"。Learner Track 与 Answer Track 都汇合到这里。

契约：[learning-artifact-contract.md §D–§G](../specs/001-rust-foundation/contracts/learning-artifact-contract.md)

## 目录内容

| 路径 | 作用 |
|-----|------|
| `capability-matrix.md` | **追踪链单一事实源**。24 行，串起 C-ID → Story → Task → 实验 → 源码 → 判据 → 状态（FR-013） |
| `criteria/cNN.md` | 24 条 Acceptance Criteria，每能力一条 |
| `environment-baseline.md` | 环境基线，后续所有 OBSERVATIONS 环境块的基准（FR-010） |
| `send-sync-quiz.md` | US4 判定题集，**MUST 在 US4 开始前冻结**（SC-007 / R-10） |
| `send-sync-quiz.answers.md` | 参考答案。**作答完成前 MUST NOT 打开** |
| `unfamiliar-code-reading.md` | SC-004 / SC-005 限时阅读评估 |
| `safety-invariant-audit.md` | unsafe 块 SAFETY 五要素覆盖审计（SC-010） |
| `traceability-audit.md` | 孤立笔记枚举结果（SC-011） |
| `dual-track-audit.md` | 双轨完整性与不泄漏核查（§H3） |
| `definition-of-done.md` | SC-001…SC-011 的逐条判定 |

## 写 Acceptance Criterion 的规则

**禁用措辞**（§D1）：「看过」「了解过」「做过笔记」「熟悉」「基本掌握」。
合格谓词参照 Constitution V：能解释 / 能画图 / 能写代码 / 能运行实验 / 能分析输出 /
能定位源码 / 能解释性能差异 / 能完成测试。

**至少一条判据 MUST 由命令退出码决定**（§D2）—— 验收不能完全依赖自我评估。

**例外**：三类目标本质上不产出可执行产物（Send/Sync 的**推导依据**、SC-004/SC-005 的
**限时阅读**、Feynman 第 1/4 节的**表达质量**）。它们改用等效客观判据（§D2a），
四项同时满足才算 pass：

1. 复核清单**事前**冻结（评估开始前提交到版本控制，事后补写无效）；
2. 逐条勾选，不接受一个总体结论；
3. 每条 `[x]` 旁边写出**证据位置**（行号/小节/断言名/源码引用）；
4. 文件末尾有复核人签名与日期，日期晚于清单提交日期。

单人项目里复核人就是学习者本人，所以**事前冻结**与**逐条证据指向**是有效性的唯一来源 ——
它俩把"我觉得我懂了"改写成"这条判断的证据在第几行"。

## 状态的单一事实源

Capability 的状态**只**看 `capability-matrix.md`。`criteria/cNN.md` 里的 `Status`
记的是那条 AC 自身的判定，两者含义不同，不要互相覆盖（§C1a）。

```text
planned → in-progress → experiment-passed → accepted
               ▲                              │
               └────────── regressed ◀────────┘
```

只有 `accepted` 计入进度。
