# learning/ —— Answer Track：概念与源码引用

**职责**：回答 `learner/` 提出的问题。每个模块两个文件。

> ⚠️ 本目录属于 **Answer Track**。如果你正在自测，先读 [`../learner/`](../learner/README.md)。

| 文件 | 契约 | 内容 |
|-----|------|------|
| `mN-<module>/concept.md` | [§A](../specs/001-rust-foundation/contracts/learning-artifact-contract.md) | 本模块回答什么问题 / 每个 Capability 的一句话定义、底层机制、常见误解、对应实验 / 与后续 Linux·eBPF·Aya 的关联（FR-014） |
| `mN-<module>/source-refs.md` | §B | 每个 Capability ≥1 条源码引用：路径、符号、**实际行号**、kind、"这段源码回答了什么"（FR-005） |

**硬性规则**

- `concept.md` 的"底层机制"MUST 下沉到编译器/运行时实际做了什么，不能停在 API 描述（Constitution I）。
- `source-refs.md` MUST 有"这段源码回答了什么"一列 —— 只记路径不记问题的引用等同于孤立笔记（§B3）。
- 路径根为 `$(rustc --print sysroot)/lib/rustlib/src/rust/library/`，行号在 pinned 工具链下固定，因此可被复核。
- `kind = reference-fallback` 只允许用于确实无库代码对应的能力（已知：C-03 borrowck、C-04 elision、C-15 UB 定义）。
- MUST NOT 粘贴 std 文档或教程原文（Constitution II）。

**归属**：本目录下每个 `### C-xx` 小节都 MUST 能在 `acceptance/capability-matrix.md` 中找到对应行，
否则按 §E3 判为孤立笔记（SC-011 要求数量为 0）。
