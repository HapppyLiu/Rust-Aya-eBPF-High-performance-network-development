//! C-21 双向调用 —— 两个方向各自一条通过判据（CHK042）。

use m6_ffi::c21::{
    PacketHdr, c_filled_hdr, c_to_rust_hdr_sum, c_to_rust_mul, rust_to_c_add, rust_to_c_hdr_sum,
};

/// CLAIM: Rust→C 整数调用：`m6_c_add(6, 7) == 13`。
#[test]
fn rust_to_c_add_returns_sum() {
    assert_eq!(rust_to_c_add(6, 7), 13);
    assert_eq!(rust_to_c_add(-2, 5), 3);
}

/// CLAIM: C→Rust 整数调用：C 包装调用 `m6_rust_mul(6, 7)` 得到 42。
/// 与上一条独立，任一条失败都不能用另一条顶替。
#[test]
fn c_to_rust_mul_returns_product() {
    assert_eq!(c_to_rust_mul(6, 7), 42);
    assert_eq!(c_to_rust_mul(0, 99), 0);
}

/// CLAIM: Rust→C 结构体指针：字段按 `repr(C)` 被 C 读到并求和。
#[test]
fn rust_to_c_struct_pointer_sum() {
    let hdr = PacketHdr {
        kind: 1,
        id: 2,
        len: 3,
    };
    assert_eq!(rust_to_c_hdr_sum(&hdr), 6);
}

/// CLAIM: C→Rust 结构体指针：C 栈上的 `PacketHdr` 被 Rust 导出函数按同名字段读到。
#[test]
fn c_to_rust_struct_pointer_sum() {
    assert_eq!(c_to_rust_hdr_sum(), 6);
}

/// CLAIM: C 写入的字段值 Rust 能按同名读回（布局一致的值级对照）。
#[test]
fn c_fill_is_readable_as_repr_c_fields() {
    let hdr = c_filled_hdr();
    assert_eq!(hdr.kind, 7);
    assert_eq!(hdr.id, 0x0102_0304);
    assert_eq!(hdr.len, 42);
}
