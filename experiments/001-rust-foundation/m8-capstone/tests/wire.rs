//! C-21 在综合实验中的布局断言。真实 C 调用见 m6-ffi，本文件只量 `repr(C)` 头。

use core::mem::{align_of, offset_of, size_of};
use m8_capstone::wire::{HEADER_SIZE, WireHeader, layout};

/// CLAIM: `WireHeader` 的 `repr(C)` 布局为 size=6、align=2、`payload_len` 偏移 4。
#[test]
fn wire_header_layout() {
    let (sz, al, off) = layout();
    assert_eq!(sz, HEADER_SIZE);
    assert_eq!(sz, size_of::<WireHeader>());
    assert_eq!(al, align_of::<WireHeader>());
    assert_eq!(al, 2);
    assert_eq!(off, offset_of!(WireHeader, payload_len));
    assert_eq!(off, 4);
    assert_eq!(offset_of!(WireHeader, magic), 0);
    assert_eq!(offset_of!(WireHeader, version), 2);
    assert_eq!(offset_of!(WireHeader, kind), 3);
}
