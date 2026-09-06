//! C-05 Struct / Enum —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c05.md`。

use m2_types::c05::{ConnState, LooseHeader, PacketHeader, WireKind};
use std::mem::{align_of, offset_of, size_of};

/// CLAIM: `#[repr(C)]` 包头的大小、对齐与 `length` 偏移由字段顺序和对齐规则决定，不是猜测。
#[test]
fn repr_c_header_has_deterministic_layout() {
    assert_eq!(size_of::<PacketHeader>(), 8);
    assert_eq!(align_of::<PacketHeader>(), 4);
    assert_eq!(offset_of!(PacketHeader, version), 0);
    assert_eq!(offset_of!(PacketHeader, kind), 1);
    assert_eq!(offset_of!(PacketHeader, length), 4);
}

/// CLAIM: 同样的有效字段换顺序会插入填充，`size_of` 变的是空位而不是数据量。
#[test]
fn field_order_changes_padding_not_payload() {
    assert_eq!(offset_of!(LooseHeader, version), 0);
    assert_eq!(
        offset_of!(LooseHeader, length),
        2,
        "u16 必须落在 2 对齐上，前面空 1 字节"
    );
    assert_eq!(offset_of!(LooseHeader, kind), 4);
    assert_eq!(size_of::<LooseHeader>(), 6);
    assert!(
        size_of::<LooseHeader>() > 1 + 2 + 1,
        "6 > 4：多出的两字节是对齐空位"
    );
}

/// CLAIM: 无载荷 enum 的大小就是判别式本身；有载荷时大小严格大于最大载荷。
#[test]
fn enum_occupies_discriminant_plus_payload_space() {
    assert_eq!(size_of::<WireKind>(), 1);
    assert_eq!(align_of::<WireKind>(), 1);

    assert!(
        size_of::<ConnState>() > size_of::<u32>(),
        "ConnState 不能只等于最大载荷：判别式必须占空间（除非走 niche，而 repr(u8) 钉死了判别式）"
    );
    assert_eq!(
        align_of::<ConnState>(),
        align_of::<u32>(),
        "对齐由最大对齐字段决定"
    );
    assert_eq!(ConnState::Idle.tag(), 0);
    assert_eq!(ConnState::Connecting { attempt: 1 }.tag(), 1);
    assert_eq!(ConnState::Established { peer: 0 }.tag(), 2);
    assert_eq!(ConnState::Closed { reason: 0 }.tag(), 3);
}

/// CLAIM: `Option<&u8>` 借助文档化的 null-pointer optimization 与 `&u8` 同宽。
///
/// 标准库保证（`core/src/option.rs` Representation）：`T = &U` 时
/// `Option<T>` 与 `T` 同大小、同对齐、同调用 ABI。
#[test]
fn option_ref_has_the_same_width_as_the_reference() {
    assert_eq!(size_of::<Option<&u8>>(), size_of::<&u8>());
    assert_eq!(align_of::<Option<&u8>>(), align_of::<&u8>());
    assert_eq!(size_of::<Option<&u8>>(), size_of::<usize>());
}

/// CLAIM: `u8` 的值域已满，`Option<u8>` 必须另辟判别式，因此比 `u8` 更宽。
#[test]
fn option_u8_needs_an_extra_discriminant_byte() {
    assert_eq!(size_of::<u8>(), 1);
    assert!(
        size_of::<Option<u8>>() > size_of::<u8>(),
        "u8 没有可用 niche，None 不能编码进有效值域"
    );
}
