//! C-15…C-20 在综合实验中的稳定断言：边界检查、未对齐拒绝、载荷视图。

use m8_capstone::buffer::PacketBuf;
use m8_capstone::error::ParseError;
use m8_capstone::raw::{payload_view, read_u8, read_u16_le, try_as_u16_slice};

/// CLAIM: `read_u8` / `read_u16_le` 在越界时返回 `Truncated`，不读穿。
#[test]
fn short_slice_is_truncated_not_ub() {
    let buf = PacketBuf::new(&[0x11]);
    assert_eq!(read_u8(buf, 1).unwrap_err(), ParseError::Truncated);
    assert_eq!(read_u16_le(buf, 0).unwrap_err(), ParseError::Truncated);
    assert_eq!(read_u8(buf, 0).unwrap(), 0x11);
}

/// CLAIM: `read_u16_le` 按小端组装，不依赖本机端序，也不要求 `u16` 对齐。
#[test]
fn le_u16_from_unaligned_bytes() {
    let bytes = [0xAB, 0xCD, 0xEF];
    let buf = PacketBuf::new(&bytes[1..]);
    assert_eq!(read_u16_le(buf, 0).unwrap(), 0xEFCD);
}

/// CLAIM: `payload_view` 越界失败；合法区间的视图长度等于请求长度。
#[test]
fn payload_view_bounds() {
    let bytes = [1, 2, 3, 4];
    let buf = PacketBuf::new(&bytes);
    assert_eq!(payload_view(buf, 2, 3).unwrap_err(), ParseError::Truncated);
    let view = payload_view(buf, 1, 2).unwrap();
    assert_eq!(view.as_bytes(), &[2, 3]);
}

/// CLAIM: 起始地址未按 `u16` 对齐时 `try_as_u16_slice` 返回 `None`；对齐且偶长度则成功。
#[test]
fn u16_slice_requires_alignment() {
    #[repr(align(2))]
    struct Aligned([u8; 8]);
    let aligned = Aligned([0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]);
    let ok = PacketBuf::new(&aligned.0[0..4]);
    let slice = try_as_u16_slice(ok).expect("aligned even slice");
    assert_eq!(slice.len(), 2);

    let bad = PacketBuf::new(&aligned.0[1..5]);
    assert!(
        try_as_u16_slice(bad).is_none(),
        "offset-1 from align(2) base is not u16-aligned"
    );

    let odd = PacketBuf::new(&aligned.0[0..3]);
    assert!(try_as_u16_slice(odd).is_none());
}
