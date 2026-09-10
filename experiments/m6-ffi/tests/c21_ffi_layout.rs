//! C-21 布局一致性 —— 双侧量过才算一致（CHK043）。

use m6_ffi::c21::{c_layout, rust_layout};

/// CLAIM: `#[repr(C)] PacketHdr` 的 size/align/三个 offset 与 C 侧导出值逐项相等。
/// 本断言核对的是两侧编译器排出的图，不是"源码里写了 repr(C)"这一事实本身。
#[test]
fn repr_c_layout_matches_c_compiler() {
    let rust = rust_layout();
    let c = c_layout();
    assert_eq!(rust.size, c.size, "size_of");
    assert_eq!(rust.align, c.align, "align_of");
    assert_eq!(rust.off_kind, c.off_kind, "offset_of kind");
    assert_eq!(rust.off_id, c.off_id, "offset_of id");
    assert_eq!(rust.off_len, c.off_len, "offset_of len");
}

/// CLAIM: `kind` 是第一个字段，偏移为 0；`id` 因 C 对齐规则排在 `kind` 之后（含填充）。
/// 不断言具体填充字节数以外的"碰巧"关系：只断言双侧同一套数字。
#[test]
fn kind_is_at_offset_zero_on_both_sides() {
    let rust = rust_layout();
    let c = c_layout();
    assert_eq!(rust.off_kind, 0);
    assert_eq!(c.off_kind, 0);
    assert_eq!(rust, c);
}
