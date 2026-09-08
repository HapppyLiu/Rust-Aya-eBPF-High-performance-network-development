//! C-19 UB 对照 + 双别名模型。
//!
//! PREDICT-UB: W1 + W8（Stacked Borrows）；W1 + W9（Tree Borrows）
//! 两轮各写一条断言，MUST NOT 断言两轮结果相同（plan.md C-19 规则 3）。

mod common;

/// CLAIM: 重叠可变引用在默认 Stacked Borrows 下是 UB，类别命中 W1 + W8。
#[test]
fn overlapping_mut_is_ub_under_stacked_borrows() {
    common::expect_ub(
        "c19_aliasing_ub",
        &["Undefined Behavior", "does not exist in the borrow stack"],
    );
}

/// CLAIM: 同一源码在 Tree Borrows 下也是 UB，类别命中 W1 + W9 白名单中的任一子串。
#[test]
fn overlapping_mut_is_ub_under_tree_borrows() {
    let Some(out) = common::run_with_flags("c19_aliasing_ub", Some("-Zmiri-tree-borrows")) else {
        return;
    };
    assert!(
        out.reported_ub(),
        "expected UB under Tree Borrows, stderr:\n{}",
        out.stderr()
    );
    assert!(
        out.stderr_contains("Undefined Behavior"),
        "missing W1\n{}",
        out.stderr()
    );
    let w9 = out.stderr_contains("tag-mismatch")
        || out.stderr_contains("protected tag")
        || out.stderr_contains("foreign write");
    assert!(
        w9,
        "missing W9 (tag-mismatch | protected tag | foreign write)\n{}",
        out.stderr()
    );
}
