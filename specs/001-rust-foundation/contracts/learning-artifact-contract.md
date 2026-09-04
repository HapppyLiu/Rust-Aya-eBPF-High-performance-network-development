# Contract: Learning / Feynman / Acceptance Artifacts

**Feature**: 001-rust-foundation

本契约定义三类文档产物的必需结构。目的是让 FR-002（模块六环节齐备）、FR-005（源码引用）、
FR-006（Feynman 五项检验）、FR-007（可验证验收标准）从"要求"变成"文件里能数出来的小节"。

**通用规则**：本契约中标注 REQUIRED 的小节缺失时，对应模块/能力 MUST NOT 被标记为完成。

---

## A. `learning/mN-<module>/concept.md` —— 概念材料

```markdown
# Module N: <名称>

**Story**: USn | **Capabilities**: C-xx … C-yy | **Prerequisite**: m(N-1)

## 这个模块回答什么问题        <!-- REQUIRED -->
（3–5 个具体问题，形如"为什么 `&mut` 不能同时存在两个？"，而非"介绍借用"）

## 概念 <!-- REQUIRED，每个 Capability 一节 -->
### C-xx <Capability 名>
- **一句话定义**
- **底层机制**：编译器 / 运行时实际做了什么（Constitution I，MUST 下沉到机制而非 API）
- **常见误解**：至少 1 条
- **对应实验**：`cNN_<slug>`（链接到 examples/tests）

## 与后续学习的关联            <!-- REQUIRED，FR-014 -->
每个 Capability 一行：与 Linux / eBPF / Aya 的关联点，
或显式写"仅为理解基础，不直接对应后续内容"。
```

**禁止**：把 `std` 文档或教程原文粘贴过来。二手资料 MAY 作入口，MUST NOT 作唯一依据
（Constitution II）。

---

## B. `learning/mN-<module>/source-refs.md` —— 源码引用（FR-005）

每个 Capability 至少一条。表格形式，字段对应 [data-model.md](../data-model.md) §6：

```markdown
| C-ID | 路径（相对 library/） | 符号 | 行 | kind | 这段源码回答了什么 |
|------|---------------------|------|----|------|------------------|
| C-12 | core/src/marker.rs | `unsafe auto trait Send` | 68 | library | Send 为何是 unsafe auto trait |
| C-03 | — Rust Reference §"Borrow checker" | — | — | reference-fallback | borrowck 属编译器内建，无库代码对应 |
```

**规则 B1**：路径根为 `$(rustc --print sysroot)/lib/rustlib/src/rust/library/`，
行号在 pinned 工具链下固定（R-01），因此可被记录并复核。

**规则 B2**：`kind = reference-fallback` 仅允许用于确实无库代码对应的能力
（已知：C-03 borrowck、C-04 elision、C-15 UB 定义）。其余能力使用 fallback MUST 说明理由。

**规则 B3**：MUST 有"这段源码回答了什么"一列。只记路径不记问题的引用等同于孤立笔记
（Constitution XIII）。

---

## C. `feynman/mN-<module>.md` —— Feynman 教学材料（FR-006 / Constitution IV）

按模块产出，共 8 份。**五个小节全部 REQUIRED，缺一即模块未完成**（不接受部分通过）。

```markdown
# Feynman: Module N — <名称>

**Capabilities covered**: C-xx … C-yy   <!-- MUST 等于该模块全部能力，SC-001 -->

## 1. 用自己的话解释            <!-- REQUIRED -->
面向"没学过 Rust 但懂 C 的同事"。禁止使用未解释的 Rust 术语。

## 2. 最小示例                  <!-- REQUIRED -->
每个 Capability 一段 ≤15 行代码，链接到对应的 examples/ 文件。

## 3. 底层机制                  <!-- REQUIRED -->
编译器/运行时实际做了什么。MUST 引用本模块的实验观测或源码引用作为依据，
而非"据说""一般认为"。

## 4. 常见误区                  <!-- REQUIRED -->
每个 Capability 至少 1 条，格式：
- **误解**：… → **实际**：… → **证据**：`cNN_<slug>` 的哪条断言/观测推翻了它

## 5. 验证性问题                <!-- REQUIRED -->
≥5 个问题 + 作者的回答。问题 MUST 是"能暴露理解缺口"的，
例如"如果去掉这个边界检查，Miri 会报告哪一类 UB？"
**每个问题的回答 MUST 指向一条实验断言名或一处源码引用**（见规则 C3）。

## 检验结果                     <!-- REQUIRED，五项各一行 -->
| # | 检验项 | 合格标准（可判定） | 状态 |
|---|-------|-----------------|------|
| 1 | 自述概念 | 第 1 节存在，且不含未解释的 Rust 术语 | pass / fail |
| 2 | 最小示例 | 每个 covered capability 各有一段 ≤15 行代码且链接到 examples/ | pass / fail |
| 3 | 底层机制 | 第 3 节每条论断都带断言名或源码符号 | pass / fail |
| 4 | 常见误区 | 每个 covered capability ≥1 条，且三段式齐备 | pass / fail |
| 5 | 回答问题 | ≥5 个问题，且每题的回答指向一条断言或一处源码引用 | pass / fail |
```

**规则 C1**：五项为**合取**。任一 fail → 模块 `feynman_status = failed` → 该模块下所有
Capability MUST NOT 进入 `accepted`，且 MUST 生成补齐任务回写 `tasks.md`。

**规则 C1a（模块门禁与能力级 AC 的关系，T002-a / CHK050）**：两级验收**独立评估、串联放行**。

| 层级 | 评估对象 | 由谁判定 | 可否在对方失败时仍为 pass |
|-----|---------|---------|----------------------|
| 能力级 | `acceptance/criteria/cNN.md` | 命令退出码 + 可观测判据 | **可以**。模块 Feynman fail 时，能力级 AC 仍按自身判据独立判 `pass` |
| 模块级 | `feynman/mN-*.md` 五项检验 | 人工复核 | 能力级全 pass 也不自动放行 |

因此当 `feynman_status = failed` 时，该模块下 Capability 的状态迁移**停在 `experiment-passed`**：

```text
planned → in-progress → experiment-passed → ✗ 被模块门禁挡住 ✗ → accepted
                                    ↑
                        AC 判 pass 也只能到这里
```

具体规定：

1. `acceptance/criteria/cNN.md` 的 `Status` 字段 MUST 如实记为 `pass` —— 它记录的是**该条
   AC 自身**的判定结果，不是 Capability 的状态。把它改成 `fail` 来"表示模块没过"是错误的，
   会丢失"实验确实通过了、缺的是讲清楚"这一诊断信息。
2. `acceptance/capability-matrix.md` 的 `Status` 列 MUST 为 `experiment-passed`，
   MUST NOT 为 `accepted`。矩阵才是 Capability 状态的单一事实源。
3. 该模块 `status` 保持 `pending`，MUST NOT 计入进度，下一个 Story MUST NOT 开始（FR-011）。
4. 补齐任务按 §Remediation 写回 `tasks.md`，触发原因记 FR-006。

**理由**：Constitution IV 说"无法讲清楚等同于没有掌握"，所以模块门禁必须能挡住 `accepted`；
但把能力级 AC 一并判 fail 会掩盖"实验通过、表达未通过"的区别，使补齐任务无从定位。

**规则 C2**：第 3、4 节的论断 MUST 可追溯到本模块的实验或源码引用。无依据的断言是
Feynman 检验最常见的失败模式，审查时优先检查此处。

**规则 C3（第 5 节的合格标准，T002-c / CHK048）**：第 5 节 MUST 满足**两项**：

1. **数量**：≥5 个验证性问题。
2. **可追溯**：每个问题的回答 MUST 至少指向下列之一 ——
   (a) 本模块某个 `#[test]` 函数名（形如 `tests/c03_borrow.rs::two_mut_borrows_rejected`），
   (b) `source-refs.md` 中的一条源码引用（路径 + 符号），
   (c) `OBSERVATIONS.md` 中某个 NON-ASSERTION 记录块的标题。

只凭记忆或推理作答、无任何指向的问题**不计入 5 个之内**。

**理由**：验证性问题的作用是暴露理解缺口。若回答可以停留在"我认为"，它就退化成又一次自述，
与第 1 节重复。强制指向产物，使"我以为的机制"与"实验实际显示的机制"必须当场对齐。

---

## D. `acceptance/criteria/cNN.md` —— 验收标准（FR-007 / Constitution V）

每个 Capability 恰好一条，共 24 份。

```markdown
# C-NN <Capability 名> — Acceptance Criterion

**Module**: mN | **Story**: USn | **UB tool**: miri / asan / compile-time / n/a

## 验证命令                     <!-- REQUIRED，MUST 可直接复制执行 -->
```
cargo test -p mN-<module> --test cNN_<slug>
cargo +nightly miri test -p mN-<module> --test cNN_<slug>   # 若适用
```

## 通过判据                     <!-- REQUIRED，MUST 可观测 -->
- [ ] 上述命令全部退出码为 0
- [ ] <能力特有的可观测判据，例如："能在不运行程序的前提下预测 E0499 的出现位置，
      预测与 rf_harness 打印的实际错误码一致">
- [ ] 源码引用已记录（路径 + 符号 + 行）
- [ ] （unsafe 适用时）每个 unsafe 块的 SAFETY 注释覆盖五要素

## 结果
**Status**: not-evaluated / pass / fail
**评估日期**： | **环境记录**：链接到 OBSERVATIONS.md
```

**规则 D1（禁用措辞）**：通过判据 MUST NOT 出现"看过""了解过""做过笔记""熟悉""基本掌握"。
合格谓词参照 Constitution V：能解释 / 能画图 / 能写代码 / 能运行实验 / 能分析输出 /
能定位源码 / 能完成测试。

**规则 D2**：至少一条判据 MUST 由**命令退出码**决定，即验收不能完全依赖自我评估。

**规则 D2a（无可执行产物目标的等效判据，T002-b / CHK003）**：少数学习目标本质上不产出可执行
产物 —— 已知恰好三类：§F 的 Send/Sync **推导依据**（结论可由编译器裁定，推导过程不能）、
§G 的 SC-004 / SC-005 **限时阅读评估**（素材是外部代码，不进本仓库构建）、以及 Feynman
第 1 / 4 节的**表达质量**。对这三类，D2 的退出码要求由下列**等效客观判据**替代，
MUST 同时满足全部四项：

| # | 要求 | 具体形式 |
|---|-----|---------|
| 1 | **事前**复核清单 | 判据逐条写死在产物文件中，MUST 在评估开始**之前**提交到版本控制。事后补写的清单无效 |
| 2 | 逐条勾选 | 每条独立勾选 `[x]` / `[ ]`，MUST NOT 只给一个总体结论 |
| 3 | 逐条证据指向 | 每条勾选 `[x]` 的旁边 MUST 写出证据位置（产物文件的行/小节、断言名、源码引用） |
| 4 | 复核人签名与日期 | 文件末尾 MUST 有 `复核人：<名> / 复核日期：<YYYY-MdD>`，日期 MUST 晚于清单的提交日期 |

**规则 D2b（通过线）**：清单**全部条目勾选**方为 pass。部分勾选 = fail，
MUST 按 §Remediation 登记补齐任务。这与 FR-006"五项检验不接受部分通过"取同一口径。

**规则 D2c（单人项目下的有效性前提）**：本项目为单人学习工程，复核人与学习者是同一人
（spec.md Assumptions 已声明）。此时清单的**事前冻结**（要求 1）与**逐条证据指向**
（要求 3）是有效性的唯一来源 —— 二者共同把"我觉得我懂了"改写成"这条判断的证据在第几行"。
若清单在评估后才写、或勾选处无证据指向，该评估 MUST 判 fail 并重做。

---

## E. `acceptance/capability-matrix.md` —— 追踪链单一事实源（FR-013 / SC-011）

```markdown
| C-ID | Capability | Module | Story | Task | Experiment | SourceRef | Criterion | UB Tool | Status |
|------|-----------|--------|-------|------|-----------|-----------|-----------|---------|--------|
| C-01 | Ownership | m1 | US1 | T029 | `c01_ownership` | core/src/ops/drop.rs `Drop` | criteria/c01.md | n/a | planned |
```

**规则 E1**：24 行，一行一个 Capability，行数与 ID 集合 MUST 与 spec.md Capability Coverage
完全一致（FR-001：无遗漏、无重复、无无归属项）。

**规则 E2**：`Status` 取值为 data-model.md §1 状态机的五个状态之一。

**规则 E2a（`Task` 列，T002-e / CHK051）**：`Task` 列为 **REQUIRED**，值是 `tasks.md` 中
产出该能力主实验的 T-ID（多个用逗号分隔，如 `T077, T078`）。

理由：FR-013 要求的链条是 **Spec → Plan → Task → Learning Material → Experiment →
Source Code → Acceptance Criteria** 共**七段**。本表原有列覆盖了 Spec（`C-ID`/`Story`）、
Plan（`Module`/`UB Tool`）、Experiment、SourceRef、Criterion，唯独 **Task 段缺失**，
使追踪链在第三段断开。加入本列后，七段全部可在单一事实源中读出，SC-011 的
"孤立笔记数量为 0"才具备可核查性。

**规则 E3（孤立笔记的可执行枚举，T002-f / CHK052 / CHK053）**：任何 `learning/`、`feynman/`
或 `learner/` 下的内容若无法在本表中找到归属，即为孤立笔记，MUST 在下一次 Review 中并入
链条或删除。

"找不到归属"的判定 MUST 用下列**具体命令**执行，而非人工回忆：

```bash
# 步骤 1：枚举全部条目（文件级 + 小节级）
#   文件级：learning/ 与 feynman/ 与 learner/ 下的全部 .md（排除 _templates/ 与 README.md）
find learning feynman learner -name '*.md' \
  -not -path '*/_templates/*' -not -name 'README.md' | sort > /tmp/rf-entries.txt

#   小节级：concept.md 中每个 "### C-xx" 小节、feynman 第 2/4 节中每个能力条目
grep -rhoE '^### (C-[0-9]{2})' learning/*/concept.md | sort -u > /tmp/rf-sections.txt

# 步骤 2：枚举矩阵中登记的 C-ID
grep -oE '^\| (C-[0-9]{2})' acceptance/capability-matrix.md \
  | grep -oE 'C-[0-9]{2}' | sort -u > /tmp/rf-matrix.txt

# 步骤 3a：小节级比对 —— 出现在 learning/ 但不在矩阵中的能力 = 孤立笔记
comm -23 <(sed 's/^### //' /tmp/rf-sections.txt) /tmp/rf-matrix.txt

# 步骤 3b：反向比对 —— 在矩阵中但无对应小节的能力 = 链条缺环（同样 MUST 为空）
comm -13 <(sed 's/^### //' /tmp/rf-sections.txt) /tmp/rf-matrix.txt

# 步骤 4：文件级归属 —— 每个文件 MUST 属于某个已登记模块
#   learning/mN-*/ 与 feynman/mN-*.md 与 learner/mN-*/ 的 mN MUST 出现在矩阵 Module 列
cut -d/ -f2 /tmp/rf-entries.txt | grep -oE '^m[1-8]' | sort -u
grep -oE '\| m[1-8] \|' acceptance/capability-matrix.md | grep -oE 'm[1-8]' | sort -u
```

**通过判据**：步骤 3a 与 3b 的输出**均为空**，且步骤 4 的两个集合中，前者 MUST 是后者的子集。
三项全部满足 → `孤立笔记数量 = 0`（SC-011）。任一项非空 → 逐条处置（并入链条或删除）后重跑。

结果写入 `acceptance/traceability-audit.md`，MUST 抄录上述命令的实际输出而非只写结论。

---

## F. `acceptance/send-sync-quiz.md` —— 判定题集（SC-007 / R-10）

**规则 F1**：MUST 在 **US4 学习开始前**定稿并提交到版本控制。题目 MUST NOT 在验收时另行挑选。

**规则 F2**：≥10 个**自定义类型**（非 std 类型），每题给出完整类型定义，要求判定 `Send` 与 `Sync`
并写出推导依据。

**规则 F3**：参考答案与推导写入 `acceptance/send-sync-quiz.answers.md`；作答前 MUST NOT 打开。

**规则 F4**：客观校验由 `experiments/m4-concurrency/tests/c12_send_sync_quiz.rs` 承担 ——
正向用 `fn assert_send<T: Send>()` / `assert_sync<T: Sync>()`，负向用 `compile_fail/` 条目
断言 `E0277`。编译器是最终裁判。

**规则 F5（通过线，T002-d / CHK034）**：通过线为**合取**的两条，缺一不可：

1. **判定正确性**：一次性作答，`Send` 与 `Sync` 两项判定**全对**才算该题正确；
   全卷错题数 **≤ 1**。
2. **推导依据**：每题 MUST 写出推导依据而非结论。**无依据的题即使判定正确也计为错题**。

**统一口径说明**：spec.md SC-007 的表述是"正确率不低于 90%（即最多错 1 道）"，
本规则的表述是"错题数 ≤ 1"。二者在题量为 10 时等价；题量 >10 时**以本规则（错题数 ≤ 1）为准**，
因为它是更严格的一方，且 SC-007 自身用括号把 90% 注释为"即最多错 1 道"，
说明立法意图是**绝对错题数**而非百分比。

**冲突时的优先级**：本契约 §F5 > spec.md SC-007 的百分比表述。
若将来二者出现实质分歧（而非上述表述差异），MUST 修订 spec.md 使其与本规则一致，
MUST NOT 在验收时临场选择宽松的一方。

**规则 F6（"一次性"的定义）**：`acceptance/send-sync-quiz.result.md` 的首次提交即为作答结果。
提交后再修改答案不改变判定。打开 `send-sync-quiz.answers.md` 的时点 MUST 晚于 result 的
首次提交时点，以版本控制记录为证。

---

## G. `acceptance/unfamiliar-code-reading.md` —— 限时阅读评估（SC-004 / SC-005）

```markdown
## SC-004: 中等复杂度代码阅读
- 素材：200–400 行、此前未读过的 Rust systems code（来源与选取日期）
- 限时：60 分钟
- 产出：所有权流转 / 抽象分发方式 / 错误传播路径 三份说明
- 复核：关键判断逐条核对，无误则 pass

## SC-005: unsafe 代码 Safety Invariant
- 素材：包含 unsafe 的陌生代码（来源与选取日期）
- 限时：30 分钟
- 产出：Safety Invariant 清单 + ≥1 处"调用方违反即 UB"的位置
- 复核：以 Miri 或人工推导验证所指位置确实构成 UB 风险
```

**规则 G1**：素材 MUST 在评估前选定并记录来源，MUST NOT 使用本 Feature 自己产出的代码
（自己写的代码不构成"陌生代码"）。

**规则 G2**：本节两项评估适用 §D2a 的等效客观判据（无可执行产物），
复核清单 MUST 在评估开始前冻结。

---

## H. Dual-Track Learning Artifact（双轨学习产物）

**适用范围**：本 Feature 的**每一个**产出学习内容的 Task。基础设施类 Task
（工具链、harness、脚本）不产出学习内容，不适用本节。

### H1. 两条轨道的定义与物理位置

```text
                     Spec
                       │
                       ▼
                     Task
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
       Learner Track        Answer Track
       学习者视图             答案视图
             │                   │
             ▼                   ▼
       学习框架 / 问题          完整答案
       不暴露答案              原理 / 源码 / 实验
             │                   │
             └─────────┬─────────┘
                       ▼
                 Acceptance Test
```

| 轨道 | 目录 | 内容 | 何时读 |
|-----|------|------|-------|
| **Learner Track** | `learner/mN-<module>/` | 学习框架、引导问题、预测表、自检清单、提示阶梯 | **先读**，用于自测 |
| **Answer Track** | `learning/mN-<module>/`、`feynman/mN-*.md`、`experiments/mN-*/`（源码） | 完整答案：概念解释、底层机制、源码行号、实验实现、Feynman 材料 | **卡住之后**再读 |
| **汇合点** | `acceptance/criteria/cNN.md`、`acceptance/capability-matrix.md` | 两轨共用的客观验收 | 任何时候 |

**规则 H1.1**：Answer Track 就是本契约 §A–§G 已定义的全部产物，**不新增、不改写**。
双轨机制只是在其**之前**加一层学习者视图，不改变任何既有验收判定。

**规则 H1.2**：`learner/` 与 `learning/` 是两个不同目录，一字之差但职责相反 ——
`learner/`（学习者）**提问**，`learning/`（学习材料）**回答**。

### H2. Learner Track 文件 schema

每个模块 `learner/mN-<module>/` 下 MUST 有三个文件：

#### H2.1 `guide.md` —— 学习框架

```markdown
# Learner Track — Module N: <名称>

**Story**: USn | **Capabilities**: C-xx … C-yy | **Prerequisite**: m(N-1)

> 本文件属于 **Learner Track**，不含答案。
> Answer Track 位于 `learning/mN-<module>/`、`feynman/mN-*.md` 与 `experiments/mN-*/`。
> 建议按 §打开 Answer Track 的条件 决定何时翻阅。

## 0. 开始之前                  <!-- REQUIRED -->
前置模块与其验收状态；本模块预期你已经能做什么。

## 1. 本模块你要能回答的问题      <!-- REQUIRED -->
每个 Capability 3–5 个具体问题。MUST 是开放问题，MUST NOT 自带答案或选项。
（例："同一作用域里两个 `&mut` 为什么不行？编译器凭什么在不运行程序的情况下知道？"）

## 2. 你要自己定位的源码          <!-- REQUIRED -->
每个 Capability 给出**搜索范围与要找的东西**，MUST NOT 给出行号或结论。
（例：C-01 → 在 `core/src/ops/` 下找到定义"值离开作用域时会发生什么"的那个 trait，
  记录它的路径、符号与行号。问题：它为什么不能被手动调用？）

## 3. 你要自己做的实验            <!-- REQUIRED -->
每个 Capability 给出**要观察什么 + 要写出什么断言**，MUST NOT 给出预期值。
（例：C-03 → 写一个同一作用域两次 `&mut` 的样本，**先预测错误码再编译**，
  把预测填进 predictions.md 的对应行。）

## 4. 提示阶梯                   <!-- REQUIRED -->
每个 Capability 至少两级提示，逐级收窄但**仍不给答案**：
- **Hint 1**（方向）：该往哪个方向想 / 该看哪一类机制
- **Hint 2**（位置）：该去哪个文件、用哪个命令观察
- **Hint 3**（可选，最后一级）：把问题拆成两个更小的问题

## 5. 打开 Answer Track 的条件     <!-- REQUIRED -->
见 §H4。

## 6. 自检                       <!-- REQUIRED -->
指向 `selfcheck.md`。
```

#### H2.2 `predictions.md` —— 先预测后验证

```markdown
# Learner Track — Module N 预测表

**规则**：`我的预测` 与 `依据` 两列 MUST 在运行任何命令**之前**填写并提交。
提交后才允许运行 `验证命令`，然后回填 `实测` 与 `一致?`。

| # | C-ID | 预测项 | 我的预测 | 依据（为什么） | 验证命令 | 实测 | 一致? |
|---|------|-------|---------|--------------|---------|------|-------|
| 1 | C-03 | 两次 `&mut` 的错误码 | | | `cargo test -p m1-ownership --test c03_borrow` | | |

## 未命中复盘                   <!-- REQUIRED，仅当存在 `一致? = ✗` -->
每条不一致 MUST 写：我原本以为的机制 / 实际的机制 / 我的心智模型错在哪一步。
```

**规则 H2.2a**：预测表的价值完全来自**先填后跑**。事后回填的预测列 MUST 视为无效，
该行判未完成。这与 experiment-contract §C5.1a 的 `PREDICT-UB` 事前预测规则同源。

**规则 H2.2b**：`一致? = ✗` **不是失败**，而是本 Feature 最有价值的产出 ——
它精确定位了一处心智模型缺口。未命中率为 0 反而应当怀疑预测是否事后回填。

#### H2.3 `selfcheck.md` —— Feynman 五项的提问版

```markdown
# Learner Track — Module N 自检

不看任何材料，逐项作答。答完之后再对照 `feynman/mN-*.md`。

- [ ] 1. 自述概念：向"懂 C 但没学过 Rust 的同事"解释本模块每个 Capability，不用未解释的术语
- [ ] 2. 最小示例：为每个 Capability 默写一段 ≤15 行代码
- [ ] 3. 底层机制：说出编译器/运行时实际做了什么，并指出你的依据来自哪个实验或哪处源码
- [ ] 4. 常见误区：为每个 Capability 说出至少 1 条你自己曾经的误解，以及推翻它的证据
- [ ] 5. 验证性问题：回答下列问题（≥5 个，此处只列问题不给答案）

## 卡在第几项？
记录卡住的项与具体卡点 —— 这是下一轮补齐任务的输入。
```

### H3. 答案不泄漏规则（本节的核心约束）

**规则 H3.1**：`learner/` 下的文件 MUST NOT 出现下列任何一类内容：

| # | 禁止内容 | 典型形式 | 为什么 |
|---|---------|---------|-------|
| L1 | 编译器错误码 | `E0499`、`E0597`、`E0277` | 错误码正是 US1/US4 要预测的答案 |
| L2 | 具体数值答案 | `size_of::<&dyn Trait>() == 16`、`分配次数 = 2`、`drop 顺序为 b,a` | 数值就是实验结论 |
| L3 | Miri UB 类别文本 | `Undefined Behavior`、`memory access failed`、W1–W11 的任何子串 | UB 类别是 `PREDICT-UB` 要预测的答案 |
| L4 | 源码行号 | `core/src/marker.rs:68` | 定位源码本身是 §B 要练的动作 |
| L5 | 机制性结论 | "因为 niche 优化把 `None` 编码进空指针表示" | 这是 concept.md §底层机制 的内容 |
| L6 | Answer Track 原文 | 从 `concept.md` / `feynman/` 复制的段落 | 直接等于泄漏 |

**规则 H3.2（允许出现的）**：`learner/` **可以**出现下列内容，它们是提问所必需的：

- Capability 名称与 C-ID、模块与 Story 编号；
- **文件路径**与**目录范围**（`core/src/ops/` 可以，`core/src/ops/drop.rs:16` 不可以）；
- 要运行的**命令**（命令不是答案，输出才是）；
- 术语名本身（"niche 优化"这个词可以出现在问题里，它的**含义与后果**不可以）；
- 待填空的表格骨架。

**规则 H3.3（边界情况）**：当一个提示不写出 L1–L5 就无法收窄时，MUST 改写为**动作指令**
而非**结论陈述**：

| ✗ 泄漏写法 | ✓ 合规写法 |
|-----------|-----------|
| "这里会报 E0499" | "先写下你预测的错误码，再编译比对" |
| "`&dyn Trait` 是 16 字节，因为含 vtable 指针" | "用 `size_of` 量一下 `&dyn Trait` 和 `&T`，差值来自什么？" |
| "Miri 会报 not sufficiently aligned" | "运行 Miri 之前，先在 predictions.md 写下你认为会触发哪一类 UB" |
| "`Drop` 定义在 `core/src/ops/drop.rs` 第 16 行" | "在 `core/src/ops/` 下找到这个 trait，记录其行号" |

**规则 H3.4（本项目为单人工程时的执行方式）**：本项目由同一人既写 Answer Track 又用
Learner Track，因此"不泄漏"不可能靠信息不对称保证，只能靠**顺序纪律**：
Learner Track 的三个文件 MUST 在对应模块的实验与 concept 材料**投入使用之前**先行提交，
且 `predictions.md` 的预测列 MUST 先于验证命令提交（H2.2a）。
版本控制的提交顺序是该纪律的唯一证据，与 §F1 的题集冻结、§D2a 的清单冻结取同一机制。

### H4. 打开 Answer Track 的条件

**规则 H4.1**：下列任一条件满足即可打开，MUST NOT 更早：

1. `guide.md` §1 的问题已逐条尝试作答，且 `predictions.md` 对应行的预测列已填写并提交；
2. 提示阶梯已用到最后一级仍无进展；
3. 实验已实际运行，实测结果与预测不一致，需要机制解释（此时**优先**只读该能力对应的
   `concept.md` 小节，而非整份文件）。

**规则 H4.2**：打开后 MUST 在 `predictions.md` 的"未命中复盘"中记录**打开原因**与**卡点**。
无记录的打开等于放弃了本次学习最有价值的信息。

**规则 H4.3**：打开 Answer Track **不影响**任何验收判定。验收由
`acceptance/criteria/cNN.md` 的客观判据决定，不因"看了答案"而降级 ——
本 Feature 验收的是**最终能力**，不是**自学纯度**。

### H5. 与既有门禁的关系

**规则 H5.1**：Learner Track 是**非阻塞**产物。它缺失不影响 Capability 的状态迁移，
但模块 MUST NOT 标记 `complete` —— 因为 §H 是本契约的 REQUIRED 小节。

**规则 H5.2**：`learner/` 下的全部文件 MUST 可在 `acceptance/capability-matrix.md` 中
找到归属（模块级即可），否则按 §E3 判为孤立笔记。§E3 的枚举命令已包含 `learner/`。

**规则 H5.3**：Learner Track MUST NOT 引入新的验收标准。它的全部产出
（预测表、自检清单）都是**学习过程记录**，不是验收依据。
