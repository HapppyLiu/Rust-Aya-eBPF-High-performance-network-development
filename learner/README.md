# Learner Track（学习者视图）

本目录是**双轨学习产物**的学习者侧。它**不含答案**。

契约：[learning-artifact-contract.md §H](../specs/001-rust-foundation/contracts/learning-artifact-contract.md)

## 两条轨道

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

| 轨道 | 在哪 | 放什么 | 何时读 |
|-----|------|-------|-------|
| **Learner Track** | `learner/mN-*/` | 引导问题、要自己定位的源码范围、要自己写的实验、预测表、提示阶梯、自检清单 | **先读** |
| **Answer Track** | `learning/mN-*/`、`feynman/mN-*.md`、`experiments/mN-*/` 源码 | 概念解释、底层机制、源码路径与行号、实验实现、Feynman 材料 | **卡住后**再读 |
| **汇合点** | `acceptance/criteria/cNN.md`、`acceptance/capability-matrix.md` | 客观验收：命令 + 退出码 + 可观测判据 | 任何时候 |

> `learner/`（学习者）**提问**，`learning/`（学习材料）**回答**。
> 两个目录一字之差，职责相反。

## 每个模块的三个文件

| 文件 | 作用 | 关键规则 |
|-----|------|---------|
| `guide.md` | 学习框架：你要能回答什么、要自己找哪些源码、要自己做哪些实验、卡住时的提示阶梯 | 提示逐级收窄但**始终不给答案** |
| `predictions.md` | 先预测后验证 | **预测列必须先填写并提交，然后才允许运行验证命令** |
| `selfcheck.md` | Feynman 五项的提问版 | 不看材料作答，答完再对照 `feynman/mN-*.md` |

## 怎么用

1. 读 `guide.md` §1，逐条尝试回答。答不上来是正常的 —— 记下来。
2. 按 §2 自己去 Rust 源码里找东西，记录路径、符号、行号。
3. 按 §3 自己写实验。**动手之前**先把预测填进 `predictions.md` 并提交。
4. 跑验证命令，回填实测值。
5. 卡住了 → 走 §4 提示阶梯，一级一级来。
6. 提示用尽仍卡住 → 按下面的条件打开 Answer Track。
7. 最后做 `selfcheck.md`。

## 什么时候可以打开 Answer Track

满足**任一**条件即可，不要更早（§H4.1）：

1. `guide.md` §1 的问题已逐条尝试作答，且 `predictions.md` 对应行的预测已填写**并提交**；
2. 提示阶梯已用到最后一级仍无进展；
3. 实验已实际跑过，实测与预测不一致，需要机制解释 ——
   此时**优先只读该能力对应的 `concept.md` 小节**，而不是整份文件。

打开之后，MUST 在 `predictions.md` 的"未命中复盘"里记下**打开原因**与**卡点**。
不记录等于扔掉了这次学习里最有价值的那条信息。

### 打开答案不影响验收

验收由 `acceptance/criteria/cNN.md` 的客观判据决定（命令退出码 + 可观测判据），
**不因为"看了答案"而降级**（§H4.3）。本 Feature 验收的是**最终能力**，不是**自学纯度**。

预测未命中同理 —— `一致? = ✗` 不是失败，它精确定位了一处心智模型缺口，
是本 Feature 最有价值的产出。反过来，未命中率为 0 才应该怀疑预测是不是事后回填的。

## 写 Learner Track 时不能写进来的东西

本项目是单人工程：同一个人既写答案又用问题。所以"不泄漏"没法靠信息不对称，
只能靠**顺序纪律** —— Learner Track 三件套 MUST 在对应模块的实验与 concept 材料
投入使用**之前**先行提交，版本控制的提交顺序是唯一证据（§H3.4）。

下面六类内容 MUST NOT 出现在 `learner/` 下的任何文件里（§H3.1）：

| # | 禁止 | 例子 |
|---|-----|------|
| L1 | 编译器错误码 | `E0499`、`E0597`、`E0277` |
| L2 | 具体数值答案 | `size_of::<&dyn Trait>() == 16`、`分配次数 = 2`、`drop 顺序为 b,a` |
| L3 | Miri UB 类别文本 | `Undefined Behavior`、`memory access failed` 等白名单子串 |
| L4 | 源码行号 | `core/src/marker.rs:68` |
| L5 | 机制性结论 | "因为 niche 优化把 `None` 编码进空指针表示" |
| L6 | Answer Track 原文 | 从 `concept.md` / `feynman/` 复制的段落 |

可以出现的：Capability 名与 C-ID、**目录/文件范围**（`core/src/ops/` 可以，带行号不可以）、
要运行的**命令**（命令不是答案，输出才是）、术语名本身（"niche 优化"这个词可以出现在问题里，
它的含义与后果不可以）、待填空的表格骨架。

收窄提示但又不能写结论时，把它改写成**动作指令**：

| ✗ 泄漏 | ✓ 合规 |
|-------|-------|
| "这里会报 E0499" | "先写下你预测的错误码，再编译比对" |
| "`&dyn Trait` 是 16 字节，因为含 vtable 指针" | "用 `size_of` 量一下 `&dyn Trait` 和 `&T`，差值来自什么？" |
| "Miri 会报 not sufficiently aligned" | "跑 Miri 之前，先在 predictions.md 写下你认为会触发哪一类 UB" |
| "`Drop` 在 `core/src/ops/drop.rs` 第 16 行" | "在 `core/src/ops/` 下找到这个 trait，记录其行号" |
