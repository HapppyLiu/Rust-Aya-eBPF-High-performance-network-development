//! C-15 安全侧 —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c15.md`。

mod common;

use m5_unsafe::c15::get_in_bounds;

/// CLAIM: 已校验的下标走 `get_unchecked`，读到的值与直接索引相同。
#[test]
fn get_unchecked_in_bounds_matches_index() {
    let data = [10u8, 20, 30, 40];
    assert_eq!(get_in_bounds(&data, 2), Some(30));
    assert_eq!(data[2], 30);
}

/// CLAIM: 越界下标被安全路径拦下，返回 `None`，不进入 `unsafe`。
#[test]
fn out_of_bounds_is_none() {
    let data = [10u8, 20, 30];
    assert_eq!(get_in_bounds(&data, 3), None);
    assert_eq!(get_in_bounds(&data, 99), None);
}

/// CLAIM: 空切片上任何下标都是 `None`。
#[test]
fn empty_slice_is_none() {
    assert_eq!(get_in_bounds(&[], 0), None);
}

/// CLAIM: 安全侧 example 在 Miri 下 `ub_verdict = clean`。
/// PREDICT-UB: clean（见 examples/c15_unsafe.rs 首部）
#[test]
fn miri_reports_clean_for_in_bounds() {
    common::expect_clean("c15_unsafe");
}
