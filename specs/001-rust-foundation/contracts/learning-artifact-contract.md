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

## 检验结果
| 检验项 | 状态 |
|-------|------|
| 1 自述概念 / 2 最小示例 / 3 底层机制 / 4 常见误区 / 5 回答问题 | pass / fail |
```

**规则 C1**：五项为**合取**。任一 fail → 模块 `feynman_status = failed` → 该模块下所有
Capability MUST NOT 进入 `accepted`，且 MUST 生成补齐任务回写 `tasks.md`。

**规则 C2**：第 3、4 节的论断 MUST 可追溯到本模块的实验或源码引用。无依据的断言是
Feynman 检验最常见的失败模式，审查时优先检查此处。

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

---

## E. `acceptance/capability-matrix.md` —— 追踪链单一事实源（FR-013 / SC-011）

```markdown
| C-ID | Capability | Module | Story | Experiment | SourceRef | Criterion | UB Tool | Status |
|------|-----------|--------|-------|-----------|-----------|-----------|---------|--------|
| C-01 | Ownership | m1 | US1 | `c01_ownership` | core/src/ops/drop.rs `Drop` | criteria/c01.md | n/a | planned |
```

**规则 E1**：24 行，一行一个 Capability，行数与 ID 集合 MUST 与 spec.md Capability Coverage
完全一致（FR-001：无遗漏、无重复、无无归属项）。

**规则 E2**：`Status` 取值为 data-model.md §1 状态机的五个状态之一。

**规则 E3**：任何 `learning/` 或 `feynman/` 下的内容若无法在本表中找到归属，即为孤立笔记，
MUST 在下一次 Review 中并入链条或删除。

---

## F. `acceptance/send-sync-quiz.md` —— 判定题集（SC-007 / R-10）

**规则 F1**：MUST 在 **US4 学习开始前**定稿并提交到版本控制。题目 MUST NOT 在验收时另行挑选。

**规则 F2**：≥10 个**自定义类型**（非 std 类型），每题给出完整类型定义，要求判定 `Send` 与 `Sync`
并写出推导依据。

**规则 F3**：参考答案与推导写入 `acceptance/send-sync-quiz.answers.md`；作答前 MUST NOT 打开。

**规则 F4**：客观校验由 `experiments/m4-concurrency/tests/c12_send_sync_quiz.rs` 承担 ——
正向用 `fn assert_send<T: Send>()` / `assert_sync<T: Sync>()`，负向用 `compile_fail/` 条目
断言 `E0277`。编译器是最终裁判。

**规则 F5**：通过线为一次性作答错 ≤1 题，且每题 MUST 给出推导依据而非结论。

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
