# Learner Track — Module 5 预测表

**Story**: US5 | **Capabilities**: C-15…C-20

## 规则

1. `我的预测` 与 `依据` 两列 MUST 在运行**任何**验证命令之前填写并**提交**。
2. 提交之后才允许跑 `验证命令`，然后回填 `实测` 与 `一致?`。
3. 事后回填的预测**无效**，该行判未完成。
4. `依据` 写不出来就填"暂无依据" —— 那同样是有效信息。
   写"感觉是"或"应该吧"则等于没填。

对照侧的 `我的预测` 填 **W 编号**（以及是否预期为干净）。
编号表在 `specs/001-rust-foundation/contracts/experiment-contract.md` 的 UB 类别节。
**不要**把表里的措辞抄进本文件。

> `一致? = ✗` **不是失败**。它精确定位了一处心智模型缺口。
> 反过来，12 项全中才值得怀疑 —— 检查一下是不是事后回填的。

## 预测表

| # | C-ID | 预测项 | 我的预测 | 依据（为什么） | 验证命令 | 实测 | 一致? |
|---|------|-------|---------|--------------|---------|------|-------|
| 1 | C-15 | `c15_unsafe`（安全侧）：工具判定是干净还是对照？若对照，W 编号是哪些？ | | | `cargo test -p m5-unsafe --test c15_unsafe`；`cargo +nightly miri run -p m5-unsafe --example c15_unsafe` | | |
| 2 | C-15 | `c15_unsafe_ub`（越界下标）：W 编号 | | | `cargo test -p m5-unsafe --test c15_unsafe_ub` | | |
| 3 | C-16 | `c16_raw_ptr`（安全侧）：干净还是对照？W 编号？ | | | `cargo test -p m5-unsafe --test c16_raw_ptr` | | |
| 4 | C-16 | `c16_raw_ptr_ub`（释放后再读）：W 编号 | | | `cargo test -p m5-unsafe --test c16_raw_ptr_ub` | | |
| 5 | C-17 | `c17_ptr_arith`（安全侧）：干净还是对照？W 编号？ | | | `cargo test -p m5-unsafe --test c17_ptr_arith` | | |
| 6 | C-17 | `c17_ptr_arith_ub`（越出分配的那种加法）：W 编号 | | | `cargo test -p m5-unsafe --test c17_ptr_arith_ub` | | |
| 7 | C-18 | `c18_alignment`（安全侧）：干净还是对照？W 编号？ | | | `cargo test -p m5-unsafe --test c18_alignment` | | |
| 8 | C-18 | `c18_alignment_ub`：普通 `cargo run` 会不会正常打印一个数？同一源码交给工具后的 W 编号？ | | | 先 `cargo run -p m5-unsafe --example c18_alignment_ub`，再 `cargo test -p m5-unsafe --test c18_alignment_ub` | | |
| 9 | C-19 | `c19_aliasing`（安全侧）：干净还是对照？W 编号？ | | | `cargo test -p m5-unsafe --test c19_aliasing` | | |
| 10 | C-19 | `c19_aliasing_ub`：默认别名模型的 W 编号；另一套模型的 W 编号（允许两者不同） | | | `cargo test -p m5-unsafe --test c19_aliasing_ub`；`tools/run-miri.sh m5-unsafe --tree-borrows` | | |
| 11 | C-20 | `c20_mem_safety`（安全侧）：干净还是对照？W 编号？ | | | `cargo test -p m5-unsafe --test c20_mem_safety` | | |
| 12 | C-20 | `c20_mem_safety_ub`（谎报长度 / 越界切片）：W 编号 | | | `cargo test -p m5-unsafe --test c20_mem_safety_ub` | | |

## 未命中复盘                   <!-- 仅当存在 `一致? = ✗` -->

每条不一致写三段。不要只写"记错了"——要定位到心智模型的哪一步出了偏差。

### #N（C-xx）

- **我原本以为**：
- **实际是**：
- **我的心智模型错在哪一步**：

## 打开 Answer Track 的记录       <!-- 仅当翻过答案 -->

| 时间 | 翻的是哪份 | 触发条件（§H4.1 的第几条） | 当时的卡点 |
|------|-----------|------------------------|-----------|
| | | | |
