//! C-19 安全侧 —— 稳定断言。

mod common;

use m5_unsafe::c19::write_twice;

/// CLAIM: `UnsafeCell` 上顺序写两次，读回最后一次写入。
#[test]
fn unsafecell_sequential_writes() {
    assert_eq!(write_twice(0, 1, 2), 2);
    assert_eq!(write_twice(9, 9, 9), 9);
}

/// CLAIM: 安全侧 example 在 Miri 下 `ub_verdict = clean`。
/// PREDICT-UB: clean
#[test]
fn miri_reports_clean_for_unsafecell() {
    common::expect_clean("c19_aliasing");
}
