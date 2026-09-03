# Contract: `rf-harness` Public API

**Feature**: 001-rust-foundation | **Crate**: `harness/` (package `rf-harness`)

`rf-harness` 是本 Feature 唯一的共享设施 crate。它**不含任何学习内容**，只提供让实验契约可被机械
执行的三件工具。**零外部依赖**（仅用 `std`），以免把学习者的注意力从语言机制转移到库 API 上（R-08）。

设计约束：任何加入 `rf-harness` 的 API MUST 是"验证设施"，而非"被学习的对象"。
诸如自定义链表、缓冲区解析器一类**学习目标本身**的代码 MUST 放在各模块 crate 的 `src/` 中。

---

## Module `compile_fail` —— 编译失败的错误码断言（R-06）

用于 US1 / US4 的"预测编译器诊断"验收循环。断言对象是**错误码**（rustc 的稳定契约），
而非诊断措辞（不稳定）。

```rust
/// 编译 `path` 指向的源文件，断言编译**失败**且 stderr 中出现全部 `expected_codes`。
///
/// - 使用 pinned stable 工具链（由 rust-toolchain.toml 决定的 PATH 上的 rustc）
/// - 以 `--edition 2024 --emit=metadata` 编译，不产生可执行文件
/// - 完整 stderr 写入 `target/compile-fail/<stem>.stderr`（NON-ASSERTION 记录）
///
/// # Panics
/// - 源文件竟然编译成功
/// - 任一 expected code 未出现在 stderr 中
/// - rustc 无法启动
pub fn expect_errors(path: impl AsRef<Path>, expected_codes: &[&str]);

/// 同上，但返回结果而非 panic，供需要检视 stderr 的实验使用。
pub fn try_compile(path: impl AsRef<Path>) -> CompileOutcome;

pub struct CompileOutcome {
    pub success: bool,
    pub stderr: String,
}

impl CompileOutcome {
    /// stderr 中是否出现给定错误码（形如 "E0499"）。
    pub fn has_code(&self, code: &str) -> bool;
    /// 提取出现过的全部错误码，按首次出现顺序去重。
    pub fn codes(&self) -> Vec<String>;
}
```

**契约保证**：
- `expect_errors` 的失败信息 MUST 同时打印**期望错误码**与**实际出现的错误码**，
  使"预测 vs 实际"的差异一眼可见——这正是 US1 AS1 的验收形式。
- 路径相对于调用它的 crate 根（`CARGO_MANIFEST_DIR`），使 `compile_fail/cNN_x.rs` 可直接书写。

---

## Module `counting_alloc` —— 确定性分配计数（R-07）

用于 US3 AS1"预测实际发生的分配次数"，以及 C-24 的 allocator 教学。分配次数是**确定性**的，
因此可作稳定断言；这是本 Feature 用来替代计时 benchmark 的核心手段。

```rust
/// 包装 `std::alloc::System` 并统计分配活动的全局分配器。
///
/// 在实验 crate 中启用：
/// ```ignore
/// #[global_allocator]
/// static A: CountingAllocator = CountingAllocator::new();
/// ```
pub struct CountingAllocator { /* AtomicUsize 计数器 */ }

impl CountingAllocator {
    pub const fn new() -> Self;
}

unsafe impl GlobalAlloc for CountingAllocator { /* alloc / dealloc / realloc 计数后转发 System */ }

/// 某段代码执行期间的分配活动快照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocStats {
    pub allocs: usize,
    pub deallocs: usize,
    pub reallocs: usize,
    pub bytes_allocated: u64,
    /// 峰值净分配字节数（确定性，可断言）
    pub peak_bytes: u64,
}

/// 测量闭包执行期间的分配活动，返回闭包结果与统计。
///
/// # 并发约束
/// 计数器是进程全局的。使用本函数的测试 MUST 串行执行
/// （置于同一 `#[test]` 内，或用 crate 内互斥量串行化），否则统计不再确定。
pub fn measure<R>(f: impl FnOnce() -> R) -> (R, AllocStats);
```

**契约保证**：
- `AllocStats` 的每个字段都是确定性的 → 可直接写进 `#[test]` 断言。
- `bytes_allocated` 与 `peak_bytes` 反映**请求的**字节数，不含分配器内部开销，
  避免把 libc 实现细节引入断言。
- 使用 `CountingAllocator` 的 crate MUST 在 `OBSERVATIONS.md` 中说明它对该 crate 全局生效。

**教学价值**（C-24）：学习者必须亲手实现 `GlobalAlloc` 才能得到计数，从而具体回答
"`no_std` 下堆分配能力由谁提供"——这正是 US7 AS2 的问题。

---

## Module `miri` —— UB 判定结果的结构化读取（FR-019 / C5）

```rust
/// 在 pinned nightly 下运行指定 example，捕获 Miri 判定结果。
///
/// 命令等价于：`cargo +nightly miri run --example <name>`
/// 若环境变量 `RF_SKIP_MIRI=1`，返回 `MiriOutcome::skipped()`，
/// 使 `cargo test --workspace` 在未安装 nightly 的环境中仍可运行
/// （此时 ub_verdict 记为 `n/a`，MUST NOT 记为 `clean`）。
pub fn run_example(name: &str) -> MiriOutcome;

pub struct MiriOutcome { /* ... */ }

impl MiriOutcome {
    /// Miri 是否报告了 Undefined Behavior。
    pub fn reported_ub(&self) -> bool;
    /// stderr 是否包含给定子串（用于断言 UB **类别**，而非完整报告）。
    pub fn stderr_contains(&self, needle: &str) -> bool;
    /// 是否因缺少 nightly/miri 而跳过。跳过时 reported_ub() panic，防止误判为 clean。
    pub fn skipped(&self) -> bool;
}
```

**契约保证（关键）**：
- 当 Miri 未运行（`skipped`）时，`reported_ub()` MUST **panic** 而非返回 `false`。
  这是 FR-019 在类型层面的强制：不允许"没跑工具"被静默当作"没有 UB"。
- `stderr_contains` 只用于匹配稳定的类别文本（`"Undefined Behavior"`、
  `"memory access failed"`、`"attempting a read access"` 等），
  MUST NOT 用于匹配 `alloc<N>` 编号或行号。

---

## Module `env` —— 环境记录（FR-010）

```rust
/// 采集当前环境记录。字段定义见 data-model.md §9。
pub fn record() -> EnvironmentRecord;

pub struct EnvironmentRecord {
    pub rustc_stable: String,
    pub rustc_nightly: Option<String>,
    pub edition: &'static str,
    pub kernel: String,
    pub arch: String,
    pub target: String,
}

impl EnvironmentRecord {
    /// 渲染为 OBSERVATIONS.md 顶部的 Markdown 环境块。
    pub fn to_markdown(&self) -> String;
}
```

`tools/env-record.sh` 是它的 shell 对应物，供不便启动 cargo 的场合（如 `no_std` 构建）使用。
两者输出格式 MUST 一致。

---

## 非目标

`rf-harness` MUST NOT 提供：

- 任何属于学习目标的数据结构或算法（链表、环形缓冲、解析器）——那是模块 crate 的内容；
- 计时 / benchmark 设施——本 Feature 不产生需 benchmark 的性能主张（R-07）；
- 对 `unsafe` 的封装糖——学习者需要直接面对 `unsafe`，封装会掩盖 Constitution VI 要陈述的不变量。
