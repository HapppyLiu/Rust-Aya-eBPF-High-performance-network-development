//! 环境记录采集（FR-010 / data-model.md §9）。
//!
//! 输出格式 MUST 与 `tools/env-record.sh` **逐字一致** —— 两者是同一份环境记录的
//! Rust 与 shell 实现。改一个就要同步改另一个。
//!
//! 之所以两份都要：`no_std` 裸机构建（US7）不便启动 cargo，那里只能用 shell 版；
//! 而实验测试内部生成环境块时用 Rust 版更自然。格式不一致会让 8 份
//! `OBSERVATIONS.md` 的环境块无法互相比对，FR-018 的跨环境判断就失去基准。

use std::process::Command;

/// 采集当前环境记录。字段定义见 data-model.md §9。
#[must_use]
pub fn record() -> EnvironmentRecord {
    EnvironmentRecord {
        rustc_stable: rustc_version(&mut Command::new("rustc"))
            .unwrap_or_else(|| "UNAVAILABLE".into()),
        // nightly 是分析工具链，缺失时记 n/a 而非报错 —— 它不参与稳定断言（R-01）。
        rustc_nightly: rustc_version(Command::new("rustup").args(["run", "nightly", "rustc"])),
        edition: "2024",
        kernel: uname("-r"),
        arch: uname("-m"),
        target: std::env::var("TARGET").unwrap_or_else(|_| HOST_TARGET.into()),
        command: None,
    }
}

const HOST_TARGET: &str = "x86_64-unknown-linux-gnu";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentRecord {
    pub rustc_stable: String,
    pub rustc_nightly: Option<String>,
    pub edition: &'static str,
    pub kernel: String,
    pub arch: String,
    pub target: String,
    /// 实际执行的完整命令。由调用方通过 [`with_command`](Self::with_command) 填入。
    pub command: Option<String>,
}

impl EnvironmentRecord {
    /// 记录产生本次观测的命令。
    #[must_use]
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// 渲染为 `OBSERVATIONS.md` 顶部的 Markdown 环境块。
    ///
    /// 格式与 `tools/env-record.sh` 的输出逐字一致。
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let nightly = self.rustc_nightly.as_deref().unwrap_or("n/a");
        let command = self
            .command
            .as_deref()
            .unwrap_or("（按各记录块内的命令为准）");
        format!(
            "## 环境记录\n\
             \n\
             | 字段 | 值 |\n\
             |------|-----|\n\
             | `rustc_stable` | {} |\n\
             | `rustc_nightly` | {nightly} |\n\
             | `edition` | {} |\n\
             | `kernel` | {} |\n\
             | `arch` | {} |\n\
             | `target` | {} |\n\
             | `command` | {command} |\n",
            self.rustc_stable, self.edition, self.kernel, self.arch, self.target,
        )
    }
}

/// 把 `rustc -Vv` 拼成 data-model §9 的示例格式：`1.98.0 (88d9e12ae 2026-08-18)`。
fn rustc_version(cmd: &mut Command) -> Option<String> {
    let out = cmd.arg("-Vv").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let field = |key: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(key))
            .map(str::trim)
            .unwrap_or_default()
            .to_owned()
    };
    let release = field("release:");
    let hash = field("commit-hash:");
    let date = field("commit-date:");
    if release.is_empty() {
        return None;
    }
    // 短 hash 取 9 位，与 `rustc -V` 自身的显示一致。
    let short: String = hash.chars().take(9).collect();
    Some(format!("{release} ({short} {date})"))
}

fn uname(flag: &str) -> String {
    Command::new("uname")
        .arg(flag)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_else(|| "UNAVAILABLE".into())
}
