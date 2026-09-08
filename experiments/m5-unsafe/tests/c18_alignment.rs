//! C-18 安全侧 —— 稳定断言。

mod common;

use m5_unsafe::c18::{aligned_addr_mod, read_aligned, read_unaligned_u16, u64_align};

/// CLAIM: `u64` 的对齐要求至少为 1，且是 2 的幂。
#[test]
fn u64_align_is_power_of_two() {
    let a = u64_align();
    assert!(a >= 1);
    assert_eq!(a.count_ones(), 1);
}

/// CLAIM: 独立 `u64` 局部变量的地址满足 `(p as usize) % align_of::<u64>() == 0`。
#[test]
fn local_u64_is_aligned() {
    assert_eq!(aligned_addr_mod(), 0);
}

/// CLAIM: 对齐满足时普通 `read` 读回原值。
#[test]
fn aligned_read_roundtrips() {
    assert_eq!(read_aligned(7), 7);
    assert_eq!(read_aligned(0), 0);
}

/// CLAIM: `read_unaligned` 能从偏移 1 处读出按**本机字节序**解释的两字节载荷。
#[test]
fn read_unaligned_recovers_payload() {
    assert_eq!(read_unaligned_u16(), u16::from_ne_bytes([0x34, 0x12]));
}

/// CLAIM: 安全侧 example 在 Miri 下 `ub_verdict = clean`。
/// PREDICT-UB: clean
#[test]
fn miri_reports_clean_for_alignment() {
    common::expect_clean("c18_alignment");
}
