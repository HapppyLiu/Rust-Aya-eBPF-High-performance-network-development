# Quickstart: Rust Foundation 验证指南

**Feature**: 001-rust-foundation | **Plan**: [plan.md](./plan.md) | **Date**: 2026-09-04

本文件是**验证/运行指南**，不是实现说明。实现细节属于 `tasks.md` 与实施阶段。
这里回答的是："怎么证明这个 Feature 真的做成了？"

产物结构见 [contracts/experiment-contract.md](./contracts/experiment-contract.md)，
实体字段见 [data-model.md](./data-model.md)。

---

## 0. Prerequisites

工具链已在本机验证可用（R-01）。每次开始前确认版本未漂移：

```bash
rustc -Vv | grep -E 'release|commit-hash'
# 期望： release: 1.98.0   commit-hash: 88d9e12ae178fab0fb5cc050a94da85685d449ea

rustup run nightly rustc -V
# 期望： rustc 1.100.0-nightly (17fd5b8a3 2026-08-28)

cargo +nightly miri --version     # 期望： miri 0.1.0 (17fd5b8a37 2026-08-28)
rustup target list --installed    # 期望包含 x86_64-unknown-none
uname -r && uname -m              # 6.6.114.1-microsoft-standard-WSL2 / x86_64
```

> **不要执行 `rustup update`**。FR-020 要求 Feature 全程锁定工具链；升级会使既有验收记录失效。

一次性初始化（若尚未建立）：

```bash
tools/env-record.sh > /tmp/env-check.md   # 确认环境记录脚本可用
cargo fetch                                # m6-ffi 的 libc / cc
```

---

## 1. 一键验证（日常主循环）

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

**判定**：`cargo test --workspace` 全绿 **=** 全部稳定断言复现（SC-002）。
`OBSERVATIONS.md` 中的地址、耗时、线程交错差异**不参与**该判定（FR-003 / R-05）。

`no_std` 模块被排除在 workspace 之外，需单独构建：

```bash
cd experiments/m7-nostd && cargo build && cd -
```

---

## 2. 按模块验证

每个模块可独立验证（这是"模块可独立验证"的操作定义）：

| 模块 | Story | 能力 | 验证命令 |
|-----|-------|------|---------|
| m1-ownership | US1 (P1) | C-01…C-04 | `cargo test -p m1-ownership` |
| m2-types | US2 (P2) | C-05…C-07 | `cargo test -p m2-types` |
| m3-composition | US3 (P2) | C-08…C-11 | `cargo test -p m3-composition` |
| m4-concurrency | US4 (P2) | C-12…C-14 | `cargo test -p m4-concurrency` |
| m5-unsafe | US5 (P1) | C-15…C-20 | `cargo test -p m5-unsafe` + Miri（见 §4） |
| m6-ffi | US6 (P2) | C-21 | `cargo test -p m6-ffi` + ASan（见 §5） |
| m7-nostd | US7 (P1) | C-22…C-24 | `cd experiments/m7-nostd && cargo build`（见 §6） |
| m8-capstone | US8 (P3) | 综合 | `cargo test -p m8-capstone`（见 §7） |

单个能力的验证（验收的最小单位）：

```bash
cargo run  -p m5-unsafe --example c18_alignment    # 观察现象（NON-ASSERTION）
cargo test -p m5-unsafe --test    c18_alignment    # 稳定断言（验收）
```

---

## 3. 场景 A：编译器诊断预测（US1 / US4 的核心循环）

这是 US1 AS1 的验收形式——**先预测，再验证**。

```bash
# 1) 先读 compile_fail 源文件，在纸上写下预测的错误码与出错行
cat experiments/m1-ownership/compile_fail/c03_two_mut_borrows.rs

# 2) 再运行断言
cargo test -p m1-ownership --test c03_borrow
```

**预期**：测试通过表示实际错误码与文件头 `//! EXPECT:` 声明一致。
若你的预测与之不符，`rf_harness` 的失败信息会同时打印期望与实际错误码。

**通过判据**：预测与编译器诊断一致，**且**能解释该规则为何存在（而非复述错误信息）——
后半句由 Feynman 材料第 4 节承担。

完整诊断文本（措辞可变，NON-ASSERTION）：

```bash
cat target/compile-fail/c03_two_mut_borrows.stderr
```

---

## 4. 场景 B：UB 判定（US5 的核心循环，FR-019）

**这是本 Feature 最重要的验证场景。** 关键在于对照——先看普通运行，再看 Miri。

```bash
# 步骤 1：普通运行。注意它会正常退出并打印"合理"的结果
cargo run -p m5-unsafe --example c18_alignment_ub

# 步骤 2：同一份源码交给 Miri
cargo +nightly miri run -p m5-unsafe --example c18_alignment_ub
```

**预期对照**（本机已实证，见 research.md R-02）：

```text
普通运行:  2                                    ← 程序正常，输出"符合预期"
Miri:      error: Undefined Behavior: memory access failed: attempting to
           access 8 bytes, but got alloc159+0x1 which is only 7 bytes from
           the end of the allocation
```

**判定规则**：
- 该实验意图是演示 UB，因此 `ub_verdict = expected-ub` 才算**通过**（Miri 必须报告 UB）。
- 步骤 1 的正常输出 **MUST NOT** 被当作"无 UB"的证据。
- 稳定断言只匹配错误**类别**文本（`"Undefined Behavior"`、`"memory access failed"`）；
  `alloc159`、字节偏移、行号是 NON-ASSERTION。

全模块 UB 扫描与别名模型对照：

```bash
cargo +nightly miri test -p m5-unsafe                              # 默认 Stacked Borrows
MIRIFLAGS="-Zmiri-tree-borrows" cargo +nightly miri test -p m5-unsafe   # Tree Borrows 对照（C-19）
MIRIFLAGS="-Zmiri-many-seeds"   cargo +nightly miri test -p m4-concurrency  # 并发交错探索（C-13/C-14）
```

> 若在无 nightly 的机器上运行，设 `RF_SKIP_MIRI=1`。此时 `ub_verdict` 记为 `n/a`——
> **不是** `clean`。`MiriOutcome::reported_ub()` 在跳过时会 panic，正是为了阻止这种误判。

---

## 5. 场景 C：FFI 双向调用（US6 / SC-008）

Miri 无法执行真实 C 调用，因此 C-21 的 UB 判定改用 ASan（R-02）。

```bash
# 布局一致性与双向调用的稳定断言
cargo test -p m6-ffi

# UB 判定
tools/run-asan.sh m6-ffi
# 等价于：RUSTFLAGS="-Zsanitizer=address" cargo +nightly test -p m6-ffi \
#           -Zbuild-std --target x86_64-unknown-linux-gnu
```

**通过判据**（SC-008）：
- Rust 调 C 与 C 调 Rust 双向均成功；
- 两侧结构体的 `size_of` / `offset_of` 断言一致（靠 `#[repr(C)]` 保证，不是靠巧合）；
- ASan 无报告；
- 学习者能说明**哪一侧负责释放**跨边界分配的内存，以及约定不一致会产生何种故障。

---

## 6. 场景 D：`no_std` 最小产物（US7 / SC-006）

这是"逐条解释编译错误归属"的验收场景，过程比结果重要。

```bash
cd experiments/m7-nostd
cargo build                       # target 由 .cargo/config.toml 固定为 x86_64-unknown-none
```

**递进式验收**（US7 AS1–AS3）：

```bash
# 1) 先临时移除 #[panic_handler]，编译，记录错误 → 归属：OS services / 语言 item
# 2) 再临时引入一个 std 类型，编译，记录错误 → 归属：std
# 3) 再尝试使用 Vec，编译，记录错误 → 归属：alloc（需 extern crate alloc + GlobalAlloc）
# 每恢复一步都重新构建，逐条记录到 OBSERVATIONS.md
```

**通过判据**（SC-006）：对构建过程中出现的**每一条**错误，都能说明缺失能力属于
`core` / `alloc` / `OS services` 中的哪一层，正确率 100%。
"不能用标准库"这种笼统归因 MUST 判为未通过（FR-009）。

产物静态检查：

```bash
nm      target/x86_64-unknown-none/debug/m7-nostd | head
readelf -h target/x86_64-unknown-none/debug/m7-nostd
```

---

## 7. 场景 E：综合实验终验收（US8 / SC-009）

```bash
cargo test -p m8-capstone
cargo +nightly miri test -p m8-capstone
```

**通过判据**：
- [ ] 24 项能力**逐项**能在综合实验产物或配套说明中定位到具体体现位置，无遗漏（SC-009）；
- [ ] 每个 unsafe 块都有覆盖五要素的 `// SAFETY:` 注释（SC-010，比例 100%）；
- [ ] Miri 报告 clean（综合实验的安全抽象必须真的安全，`ub_verdict = clean`）；
- [ ] 在记录的环境中重跑，全部稳定断言复现（FR-015）。

---

## 8. Feature 完成判定（Definition of Done）

按 Success Criteria 逐条核对。**所有条目通过才可启动 Feature 002。**

| SC | 判定方式 |
|----|---------|
| SC-001 | 8 份 `feynman/mN-*.md` 五项检验全 pass，且覆盖能力集合等于模块能力集合 |
| SC-002 | `cargo test --workspace` 全绿 + m7 构建成功；24 项能力各 ≥1 实验 |
| SC-003 | `acceptance/capability-matrix.md` 中 24 行 SourceRef 列非空 |
| SC-004 | `acceptance/unfamiliar-code-reading.md` 中 60 分钟评估记录为 pass |
| SC-005 | 同上，30 分钟 unsafe Safety Invariant 评估为 pass |
| SC-006 | 场景 D 的错误归属说明正确率 100% |
| SC-007 | `send-sync-quiz` 一次性作答错 ≤1 题，且每题有推导依据 |
| SC-008 | 场景 C 全部通过 |
| SC-009 | 场景 E 的 24 项定位无遗漏 |
| SC-010 | 全仓 unsafe 块的 SAFETY 覆盖率 100%（`cargo clippy` 的 `undocumented_unsafe_blocks` 零告警 + 人工核查五要素） |
| SC-011 | `capability-matrix.md` 外无孤立笔记，数量为 0 |

**Feature 002 硬前置**（FR-012）：`m1`、`m5`、`m7` 三个模块 `status = complete`。
未通过即阻塞，不接受"边学边补"。

---

## 9. 常见问题

**Q：Miri 很慢，能不能只在最后跑一次？**
不能。UB 判定是 `expected-ub` / `clean` 的**通过条件**，不是可选检查。可以用
`cargo +nightly miri test -p <单个模块>` 缩小范围，但不能省略。首次运行需要构建 Miri sysroot
（本机实测约 40 秒），之后有缓存。

**Q：`cargo test --workspace` 为什么不包含 m7？**
`#![no_main]` + 自定义 `#[panic_handler]` 的 crate 无法用 host target 构建，纳入 workspace 会让
一键验证永远失败。它被 `exclude`，单独构建（R-03）。

**Q：实验结果和 `OBSERVATIONS.md` 里记的不一样，算失败吗？**
看差异落在哪一侧。稳定断言（`tests/`）不一致 → 失败，能力状态回退为 `regressed`。
非断言输出（地址、耗时、线程交错）不同 → 正常，不影响判定（FR-003）。

**Q：某个实验在 aarch64 上表现不同怎么办？**
按 Spec Edge Case：**能解释差异来源**者视为掌握，仅记录现象者视为未完成。
在 `OBSERVATIONS.md` 的"架构相关性"一行写明是否可跨架构推广（FR-018）。
