//! 编译失败实验的**错误码**断言器（R-06 / experiment-contract §C4）。
//!
//! # 为什么断言错误码而不是诊断措辞
//!
//! US1 AS1 的验收形式是"在编译前预测错误类型与出错位置，预测与编译器实际诊断一致"。
//! 这里需要的粒度恰好是**错误码**：
//!
//! - 错误码（`E0499`）是 rustc 的**稳定契约**，跨版本不变 → 可作稳定断言（FR-003）；
//! - 诊断措辞、路径、行号会随版本漂移 → 只能作 NON-ASSERTION（§C4.3）。
//!
//! 因此完整 stderr 被落盘保存供人阅读，但**只有错误码进入断言**。
//! 这也是不用 `trybuild` 的原因：它比对完整 stderr，等价于 Clarification 明确拒绝的
//! "逐字节相同"。

use std::path::{Path, PathBuf};
use std::process::Command;

/// 编译 `path` 指向的源文件，断言编译**失败**且 stderr 中出现全部 `expected_codes`。
///
/// # 断言语义：子集（experiment-contract §C4.4）
///
/// 判定条件是 `expected_codes ⊆ 实际出现的错误码`，**不是**集合相等。
/// rustc 可能追加派生诊断（`E0499` 之后跟一个 `E0502`），追加项不构成失败。
/// 但样本首行的 `//! EXPECT:` MUST 逐一列出全部实际错误码 —— 那是给人看的完整声明，
/// 这里的断言只保证"你预测的那些确实出现了"。
///
/// # 路径
///
/// 相对于调用方 crate 根（`CARGO_MANIFEST_DIR`），因此可以直接写
/// `"compile_fail/c03_two_mut_borrows.rs"`。
///
/// # Panics
///
/// - 源文件竟然编译**成功**（预测"会报错"本身就错了）；
/// - 任一 expected code 未出现在 stderr 中；
/// - rustc 无法启动。
///
/// 失败信息 MUST 同时打印**期望**与**实际**错误码 —— 这正是 US1 AS1 要看的那个对照。
///
/// # Miri 下的行为
///
/// Miri 不支持 `Command::spawn`（进程隔离）。本函数在 Miri 下**跳过**并打印说明。
/// 这是安全的：编译失败是**编译期**事实，与 Miri 负责的运行期 UB 判定正交，
/// 且它在普通 `cargo test` 下始终被执行。
pub fn expect_errors(path: impl AsRef<Path>, expected_codes: &[&str]) {
    let path = path.as_ref();

    if cfg!(miri) {
        eprintln!(
            "[rf-harness] SKIP compile_fail::expect_errors({}) —— Miri 不支持子进程。\n\
             \x20            该断言由普通 `cargo test` 覆盖；此处跳过不影响 UB 判定。",
            path.display()
        );
        return;
    }

    assert!(
        !expected_codes.is_empty(),
        "expect_errors 的 expected_codes 不能为空：\n\
         \x20 断言「编译失败」而不断言「因何失败」，会让学习者用错误的理由通过验收\n\
         \x20 （research.md R-06 拒绝 `#[doc = compile_fail]` doctest 正是这个理由）。"
    );

    let outcome = try_compile(path);
    let actual = outcome.codes();

    assert!(
        !outcome.success,
        "compile_fail 样本竟然编译**成功**了：{}\n\
         \x20 期望错误码：{:?}\n\
         \x20 实际：无错误\n\
         \x20 → 该样本没有触发预期的编译器规则，MUST 修正样本或修正预测。",
        path.display(),
        expected_codes,
    );

    let missing: Vec<&str> = expected_codes
        .iter()
        .copied()
        .filter(|c| !outcome.has_code(c))
        .collect();

    assert!(
        missing.is_empty(),
        "compile_fail 错误码不匹配：{}\n\
         \x20 期望（预测）：{:?}\n\
         \x20 实际（rustc）：{:?}\n\
         \x20 缺失：{:?}\n\
         \x20 完整 stderr 已落盘：{}\n\
         \x20 → 预测与实际不一致本身就是有价值的结果：先想清楚为什么，\n\
         \x20   再决定是改样本还是改预测（MUST NOT 直接把预测抄成实际值）。",
        path.display(),
        expected_codes,
        actual,
        missing,
        stderr_dump_path(path).display(),
    );
}

/// 同 [`expect_errors`]，但返回结果而非 panic，供需要检视 stderr 的实验使用。
pub fn try_compile(path: impl AsRef<Path>) -> CompileOutcome {
    let rel = path.as_ref();
    let abs = crate_root().join(rel);

    assert!(
        abs.exists(),
        "compile_fail 样本不存在：{}\n\
         \x20 （相对路径解析自 CARGO_MANIFEST_DIR = {}）",
        abs.display(),
        crate_root().display(),
    );

    let out_dir = dump_dir().join("out");
    std::fs::create_dir_all(&out_dir).expect("无法创建 compile-fail 输出目录");

    // `--emit=metadata` 只做类型检查与借用检查，不生成代码 —— 我们要的诊断在这一步就已产生。
    // `--crate-type lib` 使样本无需 `fn main`，让样本能保持最小。
    let output = Command::new("rustc")
        .arg(&abs)
        .args(["--edition", "2024"])
        .args(["--crate-type", "lib"])
        .arg("--emit=metadata")
        .arg("--out-dir")
        .arg(&out_dir)
        // 关闭颜色，否则 ANSI 转义会混进落盘的 stderr，影响人工阅读与 grep。
        .args(["--color", "never"])
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "无法启动 rustc：{e}\n\
                 \x20 预期使用 rust-toolchain.toml 锁定的 pinned stable（R-01）。"
            )
        });

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    write_stderr_dump(rel, &stderr);

    CompileOutcome {
        success: output.status.success(),
        stderr,
    }
}

/// 一次 `compile_fail` 编译的结果。
pub struct CompileOutcome {
    /// 编译是否成功。对 `compile_fail/` 样本而言，`true` 意味着实验失败。
    pub success: bool,
    /// 完整 stderr。**NON-ASSERTION** —— MUST NOT 对它做相等比较（§C4.3）。
    pub stderr: String,
}

impl CompileOutcome {
    /// stderr 中是否出现给定错误码（形如 `"E0499"`）。
    pub fn has_code(&self, code: &str) -> bool {
        self.codes().iter().any(|c| c == code)
    }

    /// 提取出现过的全部错误码，按**首次出现顺序**去重。
    ///
    /// 只认 `error[EXXXX]` 形式。`warning[...]` 与正文中偶然出现的 `E0499` 字样不计入，
    /// 避免把"帮助信息里提到的错误码"误当成"实际触发的错误码"。
    pub fn codes(&self) -> Vec<String> {
        const HEAD: &str = "error[";
        let mut found: Vec<String> = Vec::new();
        for (at, _) in self.stderr.match_indices(HEAD) {
            let rest = &self.stderr[at + HEAD.len()..];
            let Some(end) = rest.find(']') else { continue };
            let code = &rest[..end];
            let is_code = code.len() >= 2
                && code.starts_with('E')
                && code[1..].chars().all(|c| c.is_ascii_digit());
            if is_code && !found.iter().any(|c| c == code) {
                found.push(code.to_owned());
            }
        }
        found
    }
}

fn crate_root() -> PathBuf {
    PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR 未设置：本函数只应在 cargo 驱动的测试中调用"),
    )
}

/// 定位 workspace 的 `target/compile-fail/`。
///
/// 测试二进制位于 `<target>/debug/deps/<name>`，所以从 `current_exe` 向上找第一个名为
/// `target` 的祖先即可。用"搜索"而非"固定上溯 N 级"，是因为层级会随构建配置变化
/// （`--target <triple>` 会多一层，自定义 profile 也会）。
fn dump_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("CARGO_TARGET_DIR") {
        return PathBuf::from(dir).join("compile-fail");
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(target) = exe
            .ancestors()
            .find(|p| p.file_name().is_some_and(|n| n == "target"))
    {
        return target.join("compile-fail");
    }
    crate_root().join("target").join("compile-fail")
}

fn stderr_dump_path(rel: &Path) -> PathBuf {
    let stem = rel
        .file_stem()
        .map_or_else(|| "unknown".into(), |s| s.to_string_lossy().into_owned());
    dump_dir().join(format!("{stem}.stderr"))
}

/// 把完整 stderr 落盘为 NON-ASSERTION 记录（§C4.3）。
///
/// 顶部写入产生该诊断的 rustc 版本：诊断措辞与工具链版本绑定，
/// 抄进 OBSERVATIONS 时必须能说清它出自哪个编译器（FR-018）。
fn write_stderr_dump(rel: &Path, stderr: &str) {
    let path = stderr_dump_path(rel);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_else(|| "unknown".into());

    let body = format!(
        "// NON-ASSERTION —— 完整诊断文本，MUST NOT 用于相等比较（experiment-contract §C4.3）\n\
         // sample : {}\n\
         // rustc  : {}\n\
         \n{stderr}",
        rel.display(),
        version,
    );
    let _ = std::fs::write(&path, body);
}
