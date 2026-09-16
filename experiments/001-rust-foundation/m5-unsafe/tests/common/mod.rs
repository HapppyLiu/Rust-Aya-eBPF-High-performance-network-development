//! 共享的 Miri 子进程断言。本目录不是独立测试目标。
//!
//! 每个 integration test 只走安全侧或对照侧之一，故本模块里总有一半助手看起来未使用。
#![allow(dead_code)]
//!
//! `cargo test`：有 nightly miri 时真的拉起子进程。
//! `cargo +nightly miri test`：本进程已在 Miri 下，子进程不可用，这些断言跳过；
//! 安全侧的函数体仍由 Miri 直接解释执行。

use rf_harness::miri::{self, MiriOutcome};

pub fn run(name: &str) -> Option<MiriOutcome> {
    run_with_flags(name, None)
}

pub fn run_with_flags(name: &str, flags: Option<&str>) -> Option<MiriOutcome> {
    let out = miri::run_example_with_miriflags(name, flags);
    if out.skipped() {
        eprintln!(
            "SKIP miri example={name} flags={flags:?} reason={:?}",
            out.skip_reason()
        );
        return None;
    }
    Some(out)
}

pub fn expect_ub(name: &str, needles: &[&str]) {
    let Some(out) = run(name) else {
        return;
    };
    assert!(
        out.reported_ub(),
        "expected UB for {name}, stderr:\n{}",
        out.stderr()
    );
    for n in needles {
        assert!(
            out.stderr_contains(n),
            "missing category `{n}` for {name}\nstderr:\n{}",
            out.stderr()
        );
    }
}

pub fn expect_clean(name: &str) {
    let Some(out) = run(name) else {
        return;
    };
    assert!(
        !out.reported_ub(),
        "expected clean for {name}, stderr:\n{}",
        out.stderr()
    );
}
