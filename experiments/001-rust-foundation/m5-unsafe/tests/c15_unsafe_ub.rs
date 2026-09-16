//! C-15 UB 对照 —— 稳定断言。
//!
//! PREDICT-UB: W1 + W2（见 examples/c15_unsafe_ub.rs 首部）

mod common;

/// CLAIM: 越界 `get_unchecked` 被 Miri 判定为 UB，类别命中 W1 + W2。
#[test]
fn get_unchecked_oob_is_ub_under_miri() {
    common::expect_ub(
        "c15_unsafe_ub",
        &["Undefined Behavior", "`assume` called with `false`"],
    );
}
