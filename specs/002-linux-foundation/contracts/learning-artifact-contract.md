# Contract: Learning / Feynman / Acceptance Artifacts

**Feature**: 002-linux-foundation

本契约定义文档产物的必需结构。路径从仓库根起算，本 Feature 对应 `*/002-linux-foundation/`。
标注 REQUIRED 的小节缺失时，对应模块/能力 MUST NOT 被标记为完成。

双轨精神与 001 `learning-artifact-contract.md` §H 相同；下列为 Linux Foundation 的强制文件。

模块独立验证：`./tools/lf-verify-module.sh mN-<module>` MUST 能在不依赖其它模块
运行时状态的前提下，检查本契约在该模块下的文件齐备性。

---

## A. `learning/<feature>/mN-<module>/concept.md` —— Teaching Material

```markdown
# Module N: <名称>

**Story**: USn | **Capabilities**: C-xx … C-yy | **Learn-after**: m(N-1) | **Verify**: independent

## 这个模块回答什么问题        <!-- REQUIRED -->

## 概念 <!-- REQUIRED，每个 Capability 一节 -->
### C-xx <Capability 名>
- **一句话定义**
- **底层机制**：内核 / 硬件实际做了什么（Constitution I）
- **常见误解**：至少 1 条
- **对应实验**：`cNN_<slug>`（独立、可单独运行）
- **观察通道**：proc / strace / perf / gdb / bpftrace / ip-ss / epoll …

## 与后续学习的关联            <!-- REQUIRED，FR-014 -->
每个 Capability 一行：与 eBPF / Aya / 高性能网络的关联。禁止空白。

## 深度上限                    <!-- REQUIRED，FR-024 -->
本模块明确不覆盖什么（例如：本模块不讲 io_uring / 拥塞控制 / 物理网卡驱动）。
```

**禁止**：把 man page 或教程原文粘贴过来作为唯一依据。

---

## B. `learning/<feature>/mN-<module>/source-refs.md`

| C-ID | 路径（相对 linux.git） | 符号 | 行 | kind | KERNEL_SERIES | 这段源码回答了什么 |

`KERNEL_SERIES` MUST 等于验收 VM `uname -r` 的主.次（`vX.Y`）。
kind 为 `kernel-tree` 或 `tree-url`。禁止只记文件不记符号。

---

## C. Feynman

提问版：`learner/<feature>/mN-<module>/selfcheck.md`（**不含答案**）。
参考回答：`feynman/<feature>/mN-<module>.md`（五项 REQUIRED，与 001 相同）。

面向“懂一点操作系统、没读过这棵内核树的同事”。禁止未解释的内核黑话。
五项合取。模块 Feynman fail 时能力级 AC 仍可 pass，矩阵 Status 停在 `experiment-passed`。

---

## D. `acceptance/<feature>/criteria/cNN.md`

与 001 §D 相同结构。验证命令 MUST 可直接复制。至少一条由退出码决定。
禁用措辞：看过 / 了解过 / 做过笔记 / 熟悉 / 基本掌握。

root 类（C-04、C-06、C-26）MUST 写明以 root 执行。
C-26 MUST 断言两个 namespace 的创建、隔离与删除。

偏离宿主的等价观察 MAY 作为 `experiment-passed` 的临时证据，MUST NOT 单独把 Status
写成 `accepted`。

---

## E. `acceptance/<feature>/capability-matrix.md`

27 行。列：

`C-ID | Capability | Module | Story | Task | Experiment | SourceRef | Criterion | Observe | Privilege | Status | Downstream`

孤立笔记枚举：FEATURE=`002-linux-foundation`，模块匹配 `m[1-7]`。

---

## F. Quiz

`learner/.../quiz.md`：≥5 题，无答案、无选项泄露。
`learning/.../quiz-answers.md`：每题答案 + 推导 + 指向的断言或源码符号。
通过线：错 ≤1 且无依据的正确题计错。

---

## G. Dual-Track（FR-008）

| 轨道 | 目录 | 内容 |
|------|------|------|
| Learner | `learner/002-linux-foundation/mN-*/` | `guide.md` `predictions.md` `selfcheck.md` `quiz.md` |
| Answer | `learning/` `feynman/` `experiments/` 源码 | 概念、行号、答案、实现 |
| 汇合 | `acceptance/` | AC 与矩阵 |

### Learner 禁止泄漏（L1–L7）

| # | 禁止 |
|---|------|
| L1 | 稳定断言的预期值 |
| L2 | 内核源码行号 |
| L3 | 机制性结论 |
| L4 | Quiz 答案或 Feynman 第 1/3 节原文 |
| L5 | eBPF 程序写法、Aya API |
| L6 | 具体 PID/地址样例冒充通用答案 |
| L7 | perf/bpftrace 瞬时值冒充稳定答案 |

可以写：C-ID、搜索范围（文件名可以）、要跑的命令、术语名、待填表。

打开 Answer Track 的条件与 001 相同（先预测并提交 / 提示用尽 / 实测与预测不一致）。

---

## H. 模块独立验证清单

`lf-verify-module.sh mN-<module>` MUST 检查：

1. 上表 Learner 四文件存在且（抽查）不含 L1–L7 明显泄漏模式；
2. `concept.md`、`source-refs.md`、`quiz-answers.md`、`feynman/mN-*.md` 存在；
3. 该模块每个 C-ID 有 `examples/cNN_*.rs` 或 `scripts/cNN_*.sh`，以及对应 `tests/` 或脚本退出码检查；
4. `cargo test -p <crate>` 在本模块退出码 0（基准 VM 上；偏离宿主遵循 experiment-contract skip 规则）。

该检查失败时模块 `status` MUST NOT 为 `complete`。
