# Contract: Experiment Artifact

**Feature**: 002-linux-foundation | **Consumers**: `/speckit-implement`、验收者、后续 Feature

不满足本契约的产物 MUST NOT 被标记为完成。

路径：`experiments/002-linux-foundation/`。

约束来源：spec FR-003 / R-15 —— 实验必须**小、独立、可运行、可观察**；模块必须能被独立验证。

---

## C1. 文件布局

```text
experiments/002-linux-foundation/mN-<module>/
├── Cargo.toml
├── src/lib.rs
├── examples/cNN_<slug>.rs     # 可观察；输出一律 NON-ASSERTION
├── tests/cNN_<slug>.rs        # 稳定断言；验收单位
├── scripts/cNN_<slug>.sh      # 可选：CLI / root 观测
└── OBSERVATIONS.md
```

命名 MUST 与 plan.md Capability Gate Matrix 的 Exp 列一致。
Size budget：单个 example ≤ 80 行。每个 test 文件一条主 `CLAIM`。

---

## C1.1 独立与清理

- 一次进程完成 setup → 触发**一个**机制 → 观察 → 断言 → cleanup。
- MUST NOT 依赖其它 `cNN` 仍在运行的进程、遗留 netns、遗留 mmap 文件。
- C-26 创建的 namespace 名字 MUST 带实验前缀（如 `lf-c26-a` / `lf-c26-b`），
  退出前 `ip netns del`；失败路径也 MUST 尽力删除。
- m7 capstone MUST 自己 setup/cleanup，MUST NOT 要求先手动跑 m1–m6 的 example。

---

## C1.2 可观察

每个实验 MUST 声明恰好一个主观察通道（与 data-model `observe_channel` 一致）。
没有通道、或只写“我看见了”，视为未完成。

---

## C2. 稳定断言

- MUST 只出现在 `#[test]` 或 `scripts/` 中以退出码判定的检查。
- `#[test]` MUST 带 `/// CLAIM: ...`。
- **禁止**：PID/TID 具体值、绝对地址、时间戳、perf 计数绝对值、ss Recv-Q 绝对值、
  网卡 packet 计数绝对值、完整 strace 文本逐字节比对。
- **允许**：集合包含关系、计数方向、存在性、loopback UP、本实验创建的 netns 名称出现后删除。

---

## C3. 非断言输出

`examples/` 的 println 与命令演示抄录到 OBSERVATIONS.md，解释字段 MUST 回答：
1. 为什么会这样？（机制）
2. 这不能证明什么？（证据边界）

仅复述现象视为未完成。

---

## C4. 工具探测与宿主

`lf_harness::tool_probe::have` 探测工具。

| 宿主 | 工具缺失时 |
|------|-----------|
| 基准 Ubuntu VM（默认，或 `LF_HOST_KIND=ubuntu-vm`） | 测试 **失败**。MUST NOT skip-as-pass。 |
| `LF_HOST_KIND=wsl2` / `docker` / `other` | OBSERVATIONS `[SKIP-UNAVAILABLE]` + 等价观察；矩阵备注 `off-baseline-fallback`；Status 最高 `experiment-passed` |

MUST NOT 使用 `#[ignore]` 把该能力移出 SC-002 分母。
MUST NOT 把 skip 记为“已掌握该工具”。

---

## C5. 权限

C-04 / C-06 / C-26：基准 VM 上 MUST 以 root 或契约所列 capability 执行。
普通用户失败 MUST NOT 记为 `accepted`。

脚本 shebang 旁注释 `# PRIV: root`。`lf-run.sh` 对这类脚本使用 `sudo`（若已是 root 则直接跑）。

偏离宿主无权限：记录 errno，可到 `experiment-passed`，MUST 在 VM 上补跑才能 `accepted`。

---

## C6. unsafe

若 Rust 实验使用 `unsafe`：每个 unsafe 块 MUST 有 `// SAFETY:`，覆盖有效性 / 对齐 /
别名 / provenance / 生命周期中的适用项。沿用 workspace clippy
`undocumented_unsafe_blocks = deny`。

---

## C7. 环境记录

每个模块 OBSERVATIONS 顶部一块：`rustc` `kernel` `kernel_series` `arch` `host_kind`
`tools` `command` `privilege_notes`。

可推广性一行：`可跨架构推广` 或 `仅适用于 x86_64，原因：…`。

---

## C8. 一键验证

```bash
./tools/lf-verify-module.sh m1-cli-observe   # 只验证这一模块
./tools/linux-tool-probe.sh > acceptance/002-linux-foundation/tool-probe.md
```

C 程序若存在，MUST 提供编译命令且不进入默认 `cargo test` 失败路径。

---

## C9. 禁止的产物

- `*.bpf.c`、Aya 程序、自定义 BPF 对象加载
- 把 bpftrace 写成多文件项目
- io_uring 示例或 AC
- TCP 拥塞控制实现或公式验收
- 物理网卡卸载 / 驱动开发实验
- 跨能力共享的“全局测试夹具”进程
