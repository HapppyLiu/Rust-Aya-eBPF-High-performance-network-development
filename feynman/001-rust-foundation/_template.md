# Feynman: Module N — <名称>

<!--
模板：learning-artifact-contract §C。复制为 feynman/mN-<module>.md 后填写。

五个小节全部 REQUIRED，**缺一即模块未完成**（不接受部分通过，FR-006）。
写之前先做 learner/mN-<module>/selfcheck.md —— 那是本文件的提问版。
-->

**Capabilities covered**: C-xx … C-yy   <!-- MUST 精确等于该模块全部能力，SC-001 -->

## 1. 用自己的话解释            <!-- REQUIRED -->

面向"**没学过 Rust 但懂 C** 的同事"。禁止使用未解释的 Rust 术语 ——
每出现一个新词，要么当场解释，要么换掉。

## 2. 最小示例                  <!-- REQUIRED -->

每个 Capability 一段 **≤15 行**代码，链接到对应的 `examples/` 文件。
超过 15 行说明还没找到这个概念的最小形态。

## 3. 底层机制                  <!-- REQUIRED -->

编译器 / 运行时实际做了什么。

**每条论断 MUST 引用本模块的实验断言名或源码引用作为依据**（规则 C2）。
"据说""一般认为""通常来说"是本节最常见的失败模式，审查时优先查这里。

## 4. 常见误区                  <!-- REQUIRED -->

每个 Capability 至少 1 条，三段式：

- **误解**：… → **实际**：… → **证据**：`cNN_<slug>` 的哪条断言 / 哪个观测块推翻了它

写**自己真的犯过**的误解。从别处抄来的"常见误区"没有诊断价值。

## 5. 验证性问题                <!-- REQUIRED -->

≥5 个问题 + 作者的回答。

问题 MUST 是"**能暴露理解缺口**"的，例如"如果去掉这个边界检查，Miri 会报告哪一类 UB？"
而不是"什么是所有权？"（后者只是第 1 节的重复）。

**每题的回答 MUST 指向下列之一**（规则 C3），否则该题不计入 5 个之内：

- (a) 本模块某个 `#[test]` 函数名，如 `tests/c03_borrow.rs::two_mut_borrows_rejected`
- (b) `source-refs.md` 中的一条源码引用（路径 + 符号）
- (c) `OBSERVATIONS.md` 中某个 NON-ASSERTION 记录块的标题

## 检验结果                     <!-- REQUIRED，五项各一行 -->

| # | 检验项 | 合格标准（可判定） | 状态 |
|---|-------|-----------------|------|
| 1 | 自述概念 | 第 1 节存在，且不含未解释的 Rust 术语 | pass / fail |
| 2 | 最小示例 | 每个 covered capability 各有一段 ≤15 行代码且链接到 `examples/` | pass / fail |
| 3 | 底层机制 | 第 3 节每条论断都带断言名或源码符号 | pass / fail |
| 4 | 常见误区 | 每个 covered capability ≥1 条，且三段式齐备 | pass / fail |
| 5 | 回答问题 | ≥5 个问题，且每题的回答指向一条断言 / 一处源码引用 / 一个观测块 | pass / fail |

**五项是合取。** 任一 fail →

- 本模块 `feynman_status = failed`；
- 该模块下**所有** Capability 停在 `experiment-passed`，MUST NOT 进入 `accepted`；
- 能力级 `criteria/cNN.md` 仍**如实**记为 pass（它记的是 AC 自身，不是 Capability 状态）；
- 下一个 Story MUST NOT 开始（FR-011）；
- 按 `tasks.md` §Remediation 追加补齐任务，触发原因记 FR-006。
