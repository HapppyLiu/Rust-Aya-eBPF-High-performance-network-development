//! C-20 UB 对照 —— 稳定断言。
//!
//! PREDICT-UB: W1 + W10（见 examples/c20_mem_safety_ub.rs 首部）

mod common;

/// CLAIM: 对外安全、对内 `set_len` 暴露未初始化内存的调用序列，Miri 判定 UB。
/// 类别命中 W1 + W10。调用方只是按 `Vec<u8>` 去读 —— 撒谎的是封装。
#[test]
fn lying_set_len_is_ub_under_miri() {
    common::expect_ub(
        "c20_mem_safety_ub",
        &["Undefined Behavior", "memory is uninitialized"],
    );
}
