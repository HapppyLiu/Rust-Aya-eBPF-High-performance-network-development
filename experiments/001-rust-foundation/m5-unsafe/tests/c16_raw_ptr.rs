//! C-16 安全侧 —— 稳定断言。

mod common;

use m5_unsafe::c16::{addrs_of, write_then_read};

/// CLAIM: 裸指针写再读，读回的是刚写入的值。
#[test]
fn write_then_read_roundtrips() {
    assert_eq!(write_then_read(1, 9), 9);
    assert_eq!(write_then_read(-4, 0), 0);
}

/// CLAIM: 两次 `addr_of!` 指向同一位置（地址的关系，不是具体数值）。
#[test]
fn two_addr_of_are_the_same_place() {
    let x = 4;
    let (p, q) = addrs_of(&x);
    assert_eq!(p, q);
}

/// CLAIM: 安全侧 example 在 Miri 下 `ub_verdict = clean`。
/// PREDICT-UB: clean
#[test]
fn miri_reports_clean_for_raw_ptr() {
    common::expect_clean("c16_raw_ptr");
}
