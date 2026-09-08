//! C-20 安全侧 —— 稳定断言。

mod common;

use m5_unsafe::c20::{as_u16_slice, parse_u16_be_checked, parse_u16_be_unchecked};

/// CLAIM: 边界内大端 `u16` 解析读到预期值。
#[test]
fn checked_parse_in_bounds() {
    let buf = [0x12u8, 0x34, 0x56, 0x78];
    assert_eq!(parse_u16_be_checked(&buf, 0), Some(0x1234));
    assert_eq!(parse_u16_be_checked(&buf, 2), Some(0x5678));
}

/// CLAIM: 越界返回 `None`，调用方无法逼出 UB。
#[test]
fn checked_parse_out_of_bounds_is_none() {
    let buf = [0x12u8, 0x34];
    assert_eq!(parse_u16_be_checked(&buf, 1), None);
    assert_eq!(parse_u16_be_checked(&buf, 2), None);
}

/// CLAIM: 先检查再走 unchecked，结果与 checked 相同。
#[test]
fn unchecked_after_check_matches() {
    let buf = [0x08u8, 0x00, 0x45, 0x00];
    let off = 2;
    let checked = parse_u16_be_checked(&buf, off).unwrap();
    // SAFETY:
    // - 有效性：上一行对同一 `off` 返回 `Some`，故 `off + 2 <= buf.len()`。
    // - 对齐：按字节读，`u8` 对齐为 1。
    // - 别名：只读。
    // - provenance：来自 `buf`。
    // - 生命周期：`buf` 活过本测试。
    let unchecked = unsafe { parse_u16_be_unchecked(&buf, off) };
    assert_eq!(checked, unchecked);
}

/// CLAIM: 偶数长度且对齐时 `from_raw_parts` 包装给出元素个数 = 字节数 / 2。
#[test]
fn as_u16_slice_len() {
    let packed = m5_unsafe::c20::AlignedBytes::demo();
    let s = as_u16_slice(&packed.0).expect("aligned even slice");
    assert_eq!(s.len(), 2);
}

/// CLAIM: 奇数长度被拒绝。
#[test]
fn odd_len_rejected() {
    assert!(as_u16_slice(&[1, 2, 3]).is_none());
}

/// CLAIM: 安全侧 example 在 Miri 下 `ub_verdict = clean`。
/// PREDICT-UB: clean
#[test]
fn miri_reports_clean_for_mem_safety() {
    common::expect_clean("c20_mem_safety");
}

/// CLAIM: 边界检查对照 example 同样 `clean`（T090 的检查侧）。
#[test]
fn miri_reports_clean_for_bounds_check() {
    common::expect_clean("c20_bounds_check");
}
