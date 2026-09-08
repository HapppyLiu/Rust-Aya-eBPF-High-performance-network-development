//! C-13 Concurrency —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c13.md`。
//! 本文件 MUST NOT 断言线程完成顺序（experiment-contract §C2.2）。

use m4_concurrency::c13::{scoped_mutex_sum, scoped_push_ids};

/// CLAIM: 4 个 scoped 线程经 Mutex 各加 25，总和为 100；与谁先结束无关。
#[test]
fn scoped_mutex_sum_is_order_independent() {
    assert_eq!(scoped_mutex_sum(4, 25), 100);
}

/// CLAIM: 推入 0..4 之后，集合是 {0,1,2,3}，元素和为 6。
/// 不断言 Vec 的排列。
#[test]
fn scoped_id_set_is_complete() {
    let mut ids = scoped_push_ids(4);
    ids.sort_unstable();
    assert_eq!(ids, vec![0, 1, 2, 3]);
}

/// CLAIM: 安全 Rust 中两个线程对同一局部变量 `+=` 被借用规则拒绝（E0499）。
#[test]
fn data_race_in_safe_rust_is_e0499() {
    rf_harness::compile_fail::expect_errors("compile_fail/c13_data_race.rs", &["E0499"]);
}
