//! C-18 UB 对照 —— 稳定断言。本 Feature 的核心教学对照。
//!
//! 普通运行打印 `2`（NON-ASSERTION）。同一源码在 Miri 下判定 UB。
//! PREDICT-UB: W1 + W2（见 examples/c18_alignment_ub.rs 首部）

mod common;

/// CLAIM: 未对齐的 `u64` 读写被 Miri 判定为 UB。
/// 断言只匹配类别文本；`alloc` 编号 / 偏移 / 行号为 NON-ASSERTION。
#[test]
fn misaligned_u64_is_ub_under_miri() {
    common::expect_ub(
        "c18_alignment_ub",
        &["Undefined Behavior", "memory access failed"],
    );
}
