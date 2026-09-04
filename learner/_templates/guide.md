# Learner Track — Module N: <名称>

<!--
模板：learning-artifact-contract §H2.1。复制为 learner/mN-<module>/guide.md 后填写。

⚠️ 本文件属于 Learner Track。写它的时候，下列六类内容 MUST NOT 出现（§H3.1）：
   L1 编译器错误码（E0499…）        L4 源码行号（xxx.rs:68）
   L2 具体数值答案（size == 16）     L5 机制性结论（"因为 niche 优化把…"）
   L3 Miri UB 类别文本               L6 从 concept.md / feynman/ 复制的段落

   可以写：C-ID 与能力名、**目录/文件范围**、要跑的**命令**、术语名本身、待填空的表格。

   收窄提示但不能写结论时，改写成**动作指令**：
     ✗ "这里会报 E0499"        ✓ "先写下你预测的错误码，再编译比对"
     ✗ "`Drop` 在 drop.rs:16"  ✓ "在 core/src/ops/ 下找到这个 trait，记录其行号"
-->

**Story**: USn | **Capabilities**: C-xx … C-yy | **Prerequisite**: m(N-1)

> 本文件属于 **Learner Track**，不含答案。
> Answer Track 在 `learning/mN-<module>/`、`feynman/mN-<module>.md` 与 `experiments/mN-<module>/`。
> 什么时候可以翻，见下面 §5。

## 0. 开始之前                  <!-- REQUIRED -->

- **前置模块**：m(N-1)，当前验收状态：<查 `acceptance/capability-matrix.md`>
- **本模块假定你已经能**：<列出依赖的前置能力，一句话一条>
- **本模块结束时你应该能**：<对应 Independent Test 的表述>

## 1. 本模块你要能回答的问题      <!-- REQUIRED -->

每个 Capability 3–5 个**开放**问题。不要自带答案，也不要写成选择题。

### C-xx <Capability 名>

1.
2.
3.

## 2. 你要自己定位的源码          <!-- REQUIRED -->

给**搜索范围**和**要找什么**，不给行号、不给结论。

定位工具：

```bash
SRC="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
grep -rn '<你要找的符号>' "$SRC/core/src/<范围>/"
```

| C-ID | 去哪找 | 找什么 | 找到后回答 |
|------|-------|-------|-----------|
| C-xx | `core/src/<dir>/` | <描述那个东西的作用，不给名字或给了名字不给位置> | <一个只有读了源码才答得上的问题> |

## 3. 你要自己做的实验            <!-- REQUIRED -->

给"**要观察什么** + **要写出什么断言**"，不给预期值。

| C-ID | 实验 | 观察什么 | 断言什么 | 动手前先做 |
|------|-----|---------|---------|-----------|
| C-xx | `cNN_<slug>` | <现象> | <断言的**对象**，不是它的值> | 填 `predictions.md` 第 N 行并提交 |

## 4. 提示阶梯                   <!-- REQUIRED -->

每个 Capability 至少两级，逐级收窄，**每一级都不给答案**。

### C-xx

- **Hint 1（方向）**：<该往哪类机制上想>
- **Hint 2（位置）**：<该看哪个文件、跑哪个命令去观察>
- **Hint 3（拆解，可选）**：<把问题拆成两个更小的问题>

## 5. 打开 Answer Track 的条件     <!-- REQUIRED -->

满足**任一**即可，不要更早（§H4.1）：

1. §1 的问题已逐条尝试作答，且 `predictions.md` 对应行的预测已填写**并提交**；
2. 提示阶梯已用到最后一级仍无进展；
3. 实验已实际跑过，实测与预测不一致，需要机制解释 ——
   此时**优先只读该能力对应的 `concept.md` 小节**，不要整份翻。

打开后 MUST 在 `predictions.md` 的"未命中复盘"里记下**打开原因**与**卡点**。

打开答案**不影响验收**（§H4.3）。验收由 `acceptance/criteria/cNN.md` 的客观判据决定。

## 6. 自检                       <!-- REQUIRED -->

做完上面全部内容后，去 [`selfcheck.md`](./selfcheck.md)。
