//! C-16 UB 对照 —— 稳定断言。
//!
//! PREDICT-UB: W1 + W6（见 examples/c16_raw_ptr_ub.rs 首部）

mod common;

/// CLAIM: 释放后再 `ptr::read` 被 Miri 判定为 UB，类别命中 W1 + W6。
#[test]
fn use_after_free_is_ub_under_miri() {
    common::expect_ub("c16_raw_ptr_ub", &["Undefined Behavior", "has been freed"]);
}
