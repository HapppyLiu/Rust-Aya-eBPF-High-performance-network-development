//! C-17 UB 对照 —— 稳定断言。
//!
//! PREDICT-UB: W1 + W7（见 examples/c17_ptr_arith_ub.rs 首部）

mod common;

/// CLAIM: 越出分配的 `add` 被 Miri 判定为 UB，类别命中 W1 + W7。
#[test]
fn add_past_allocation_is_ub_under_miri() {
    common::expect_ub(
        "c17_ptr_arith_ub",
        &["Undefined Behavior", "in-bounds pointer arithmetic failed"],
    );
}
