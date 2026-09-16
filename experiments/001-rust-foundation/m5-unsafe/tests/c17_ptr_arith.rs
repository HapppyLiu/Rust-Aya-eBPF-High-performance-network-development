//! C-17 安全侧 —— 稳定断言。

mod common;

use m5_unsafe::c17::{
    add_matches_wrapping_one_step, offset_matches_add, one_step_byte_distance, read_at,
};
use std::mem::size_of;

/// CLAIM: 分配内 `add` 读到对应元素。
#[test]
fn add_within_allocation_reads_element() {
    assert_eq!(read_at(0), Some(10));
    assert_eq!(read_at(1), Some(20));
    assert_eq!(read_at(2), Some(30));
}

/// CLAIM: 越出逻辑下标的安全包装返回 `None`，不进入越界 `add`。
#[test]
fn index_past_end_is_none() {
    assert_eq!(read_at(3), None);
}

/// CLAIM: 分配内走一步，`add` 与 `wrapping_add` 得到同一指针。
#[test]
fn wrapping_add_agrees_inside_allocation() {
    assert!(add_matches_wrapping_one_step());
}

/// CLAIM: 分配内 `offset(1)` 与 `add(1)` 重合。
#[test]
fn offset_agrees_with_add() {
    assert!(offset_matches_add());
}

/// CLAIM: 走一步的字节差等于 `size_of::<u32>()`（地址关系，不是具体地址）。
#[test]
fn one_step_is_one_element_wide() {
    assert_eq!(one_step_byte_distance(), size_of::<u32>());
}

/// CLAIM: 安全侧 example 在 Miri 下 `ub_verdict = clean`。
/// PREDICT-UB: clean
#[test]
fn miri_reports_clean_for_ptr_arith() {
    common::expect_clean("c17_ptr_arith");
}
