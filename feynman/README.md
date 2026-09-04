# feynman/ —— Answer Track：Feynman 教学材料

**职责**：Constitution IV 的落点。按**模块**产出，共 8 份（`m1-ownership.md` … `m8-capstone.md`）。

> ⚠️ 本目录属于 **Answer Track**。自测请先用 [`../learner/mN-*/selfcheck.md`](../learner/README.md)。

契约：[learning-artifact-contract.md §C](../specs/001-rust-foundation/contracts/learning-artifact-contract.md)

## 五个 REQUIRED 小节

| # | 小节 | 合格标准（可判定） |
|---|-----|-----------------|
| 1 | 用自己的话解释 | 面向"懂 C 但没学过 Rust 的同事"，不含未解释的 Rust 术语 |
| 2 | 最小示例 | 每个 covered capability 各一段 ≤15 行代码，链接到 `examples/` |
| 3 | 底层机制 | 每条论断都带**断言名或源码符号**，不能是"据说""一般认为" |
| 4 | 常见误区 | 每个 capability ≥1 条，格式为 **误解 → 实际 → 证据**（指向具体断言/观测） |
| 5 | 验证性问题 | ≥5 个能暴露理解缺口的问题，**每题的回答 MUST 指向一条断言、一处源码引用或一个观测块** |

## 五项是**合取**

任一项 fail → 该模块 `feynman_status = failed` → 后果：

- 该模块下**所有** Capability MUST NOT 进入 `accepted`，状态停在 `experiment-passed`；
- 能力级 `acceptance/criteria/cNN.md` 仍**如实**记为 `pass`（它记的是 AC 自身，不是 Capability 状态）；
- 模块 `status` 保持 `pending`，下一个 Story MUST NOT 开始（FR-011）；
- 补齐任务按 tasks.md §Remediation 写回，触发原因记 FR-006。

不接受部分通过。`Capabilities covered` MUST 精确等于该模块的全部能力（SC-001）。

## 最常见的失败模式

第 3、4 节出现**无依据的断言**。审查时优先检查这两节：每一条都应该能指回本模块的某条
`#[test]`、某处 `source-refs.md` 条目，或 `OBSERVATIONS.md` 里的某个记录块。
