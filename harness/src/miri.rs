//! UB 判定结果的结构化读取（FR-019 / experiment-contract §C5）。
//!
//! # 本模块最重要的一条设计
//!
//! [`MiriOutcome::reported_ub`] 在 Miri **未运行**时会 **panic**，而不是返回 `false`。
//!
//! 这是 FR-019 在**类型层面**的强制。FR-019 说："'程序未崩溃'或'输出符合预期'
//! MUST NOT 被作为不存在 UB 的证据。" 如果 `reported_ub()` 在没跑工具时静默返回
//! `false`，那么一个 `assert!(!out.reported_ub())` 会在 Miri 缺席时**照常通过**——
//! 验收就变成了自欺。让它 panic，意味着"没跑工具"这件事无法被悄悄忽略。
//!
//! 对应到 `ub_verdict` 字段：未运行工具时只能记 `n/a`，MUST NOT 记 `clean`（§C5.2）。

use std::path::PathBuf;
use std::process::Command;

/// 在 pinned nightly 下运行指定 example，捕获 Miri 判定结果。
///
/// 等价命令：`cargo +nightly miri run -p <当前 crate> --example <name>`
///
/// # 跳过
///
/// 下列任一情况返回 [`MiriOutcome::skipped`]：
///
/// - 环境变量 `RF_SKIP_MIRI=1`（供未安装 nightly 的环境仍能跑 `cargo test --workspace`）；
/// - 本进程自身就跑在 Miri 下（`cfg!(miri)`）—— Miri 不支持子进程；
/// - `cargo +nightly miri` 不可用。
///
/// 跳过时 `ub_verdict` MUST 记 `n/a`。
pub fn run_example(name: &str) -> MiriOutcome {
    if cfg!(miri) {
        return MiriOutcome::skipped_because("本进程已在 Miri 下运行，Miri 不支持子进程");
    }
    if std::env::var("RF_SKIP_MIRI").is_ok_and(|v| v == "1") {
        return MiriOutcome::skipped_because("RF_SKIP_MIRI=1");
    }
    if !miri_available() {
        return MiriOutcome::skipped_because("pinned nightly 上未找到 cargo-miri");
    }

    let manifest = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR 未设置：本函数只应在 cargo 驱动的测试中调用"),
    );

    let output = Command::new("cargo")
        .arg("+nightly")
        .arg("miri")
        .arg("run")
        .arg("--example")
        .arg(name)
        .current_dir(&manifest)
        // 让子进程继承干净的标志：MIRIFLAGS 由调用方（tools/run-miri.sh 或测试）显式设置，
        // 这里不擅自追加，否则 OBSERVATIONS 里记录的命令与实际执行的不一致。
        .output()
        .unwrap_or_else(|e| panic!("无法启动 cargo miri：{e}"));

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

    MiriOutcome {
        stderr,
        stdout,
        skipped: None,
    }
}

fn miri_available() -> bool {
    Command::new("cargo")
        .args(["+nightly", "miri", "--version"])
        .output()
        .is_ok_and(|o| o.status.success())
}

/// 一次 Miri 运行的结果。
pub struct MiriOutcome {
    stderr: String,
    stdout: String,
    /// `Some(reason)` 表示 Miri 未运行。
    skipped: Option<&'static str>,
}

impl MiriOutcome {
    fn skipped_because(reason: &'static str) -> Self {
        Self {
            stderr: String::new(),
            stdout: String::new(),
            skipped: Some(reason),
        }
    }

    /// 直接构造一个"已跳过"的结果。**仅供 `tests/harness_selfcheck.rs` 使用。**
    ///
    /// 自检需要验证"跳过时 `reported_ub()` 会 panic"这条契约。走正常路径会遇到两个问题：
    /// 装了 nightly 时 `run_example` 会真的去跑 Miri（慢且依赖环境），
    /// 而用 `RF_SKIP_MIRI` 环境变量制造跳过则要调用 `set_var` ——
    /// 它在 edition 2024 里是 `unsafe` 的，因为 cargo 并行跑测试时
    /// 另一个线程可能正在读环境变量，这是真实的数据竞争，不是形式上的。
    ///
    /// 直接构造状态既避开了竞争，也让自检测的正是它想测的那件事：
    /// **类型在"未运行"状态下的行为**，而不是"怎么进入该状态"。
    #[doc(hidden)]
    #[must_use]
    pub fn skipped_for_selfcheck(reason: &'static str) -> Self {
        Self::skipped_because(reason)
    }

    /// Miri 是否报告了 Undefined Behavior。
    ///
    /// # Panics
    ///
    /// **Miri 未运行时 panic**（见模块文档）。这是刻意的：
    /// 返回 `false` 会让"没跑工具"被静默当成"没有 UB"，而 FR-019 明确禁止这一点。
    ///
    /// 需要在可能缺少 nightly 的环境里跑测试时，先用 [`skipped`](Self::skipped) 判断。
    #[must_use]
    pub fn reported_ub(&self) -> bool {
        assert!(
            self.skipped.is_none(),
            "reported_ub() 在 Miri 未运行时被调用（原因：{}）。\n\
             \x20 FR-019：'程序未崩溃'或'工具没跑' MUST NOT 被当作不存在 UB 的证据。\n\
             \x20 此时 ub_verdict 只能记 `n/a`，MUST NOT 记 `clean`。\n\
             \x20 若确实需要在无 nightly 环境下跳过，请先检查 skipped()。",
            self.skipped.unwrap_or("unknown"),
        );
        self.stderr.contains("Undefined Behavior")
    }

    /// stderr 是否包含给定子串。
    ///
    /// 只用于匹配**稳定的类别文本**（experiment-contract §C5.3 的 W1–W11 白名单），
    /// MUST NOT 用于匹配 `alloc<N>` 编号、字节偏移或行号 —— 那些每次运行都可能变。
    ///
    /// # Panics
    ///
    /// 同 [`reported_ub`](Self::reported_ub)：Miri 未运行时 panic。
    #[must_use]
    pub fn stderr_contains(&self, needle: &str) -> bool {
        assert!(
            self.skipped.is_none(),
            "stderr_contains() 在 Miri 未运行时被调用（原因：{}）。\n\
             \x20 未运行时 stderr 为空，任何 contains 都会返回 false，\n\
             \x20 使 `assert!(!out.stderr_contains(..))` 形式的断言变成恒真。",
            self.skipped.unwrap_or("unknown"),
        );
        self.stderr.contains(needle)
    }

    /// 是否因缺少 nightly/miri 或身处 Miri 内而跳过。
    #[must_use]
    pub fn skipped(&self) -> bool {
        self.skipped.is_some()
    }

    /// 跳过原因；未跳过时为 `None`。用于在 OBSERVATIONS 中记录为何 `ub_verdict = n/a`。
    #[must_use]
    pub fn skip_reason(&self) -> Option<&'static str> {
        self.skipped
    }

    /// 完整 stderr。**NON-ASSERTION** —— 供抄录进 OBSERVATIONS，不参与相等比较。
    #[must_use]
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    /// 完整 stdout。**NON-ASSERTION**。
    #[must_use]
    pub fn stdout(&self) -> &str {
        &self.stdout
    }
}
