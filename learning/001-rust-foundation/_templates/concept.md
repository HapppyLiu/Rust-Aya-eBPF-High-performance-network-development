# Module N: <名称>

<!--
模板：learning-artifact-contract §A。复制为 learning/mN-<module>/concept.md 后填写。

本文件属于 **Answer Track**。对应的 Learner Track 是 learner/mN-<module>/guide.md。
两者的关系：guide.md 提问，本文件回答。写本文件之前，guide.md MUST 已经提交（§H3.4）。

禁止：粘贴 std 文档或教程原文。二手资料 MAY 作入口，MUST NOT 作唯一依据（Constitution II）。
-->

**Story**: USn | **Capabilities**: C-xx … C-yy | **Prerequisite**: m(N-1)

## 这个模块回答什么问题        <!-- REQUIRED -->

3–5 个**具体**问题，形如"为什么 `&mut` 不能同时存在两个？"，
而不是"介绍借用"这类章节标题。

## 概念                        <!-- REQUIRED，每个 Capability 一节 -->

### C-xx <Capability 名>

- **一句话定义**：
- **底层机制**：编译器 / 运行时**实际做了什么**。
  MUST 下沉到机制而非 API（Constitution I）——
  "调用 `drop`" 是 API 描述，"编译器在作用域末尾插入 drop glue 调用" 才是机制。
- **常见误解**：至少 1 条。格式：误解 → 实际 → 证据（指向断言名或观测块）。
- **对应实验**：`cNN_<slug>`（链接到 `examples/` 与 `tests/`）

## 与后续学习的关联            <!-- REQUIRED，FR-014 -->

每个 Capability 一行：与 Linux / eBPF / Aya 的关联点，
或显式写"仅为理解基础，不直接对应后续内容"。

**不要硬凑关联**。写"仅为理解基础"是完全合格的答案；编造一个牵强的 eBPF 联系
反而会在后续 Feature 里变成误导。
