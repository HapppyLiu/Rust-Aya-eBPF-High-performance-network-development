//! `#[repr(C)]` 报文头（C-21）与线上编码。
//!
//! 本层只承担**布局**与**显式小端编码**。不做真实 C 调用：本 crate 是 `no_std` 库，
//! 双向 FFI 与 syscall 已在 `m6-ffi` 验收。SC-009 允许"产物或配套说明"——
//! 配套说明见 `acceptance/capability-location-map.md` 的 C-21 行。
//!
//! `repr(C)` 保证字段顺序与偏移；**不**保证与线上字节序相同。线上编码固定为小端，
//! 由 [`encode_header`] / [`crate::raw::read_u16_le`] 处理，避免把本机端序写进断言。

use core::mem::{align_of, offset_of, size_of};

/// 线上魔数（小端写入）。
pub const MAGIC: u16 = 0xA5A5;
/// 本解析器接受的唯一版本。
pub const VERSION: u8 = 1;
/// 报文头线上长度（字节）。与 [`size_of::<WireHeader>()`] 相同，因为字段排列无填充。
pub const HEADER_SIZE: usize = 6;

/// 报文种类。用 enum 而不是魔法数（C-05），线上仍以 `u8` 出现。
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameKind {
    Data = 0,
    Control = 1,
}

impl FrameKind {
    pub fn from_wire(v: u8) -> Result<Self, crate::error::ParseError> {
        match v {
            0 => Ok(Self::Data),
            1 => Ok(Self::Control),
            _ => Err(crate::error::ParseError::BadKind),
        }
    }
}

/// 与 C `struct { uint16_t magic; uint8_t version; uint8_t kind; uint16_t payload_len; }`
/// 对应的 Rust 侧头。字段值是**逻辑**值（本机端序）；线上字节由编解码函数处理。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WireHeader {
    pub magic: u16,
    pub version: u8,
    pub kind: u8,
    pub payload_len: u16,
}

impl WireHeader {
    pub fn kind(self) -> Result<FrameKind, crate::error::ParseError> {
        FrameKind::from_wire(self.kind)
    }
}

const _: () = {
    assert!(size_of::<WireHeader>() == HEADER_SIZE);
    assert!(align_of::<WireHeader>() == 2);
    assert!(offset_of!(WireHeader, magic) == 0);
    assert!(offset_of!(WireHeader, version) == 2);
    assert!(offset_of!(WireHeader, kind) == 3);
    assert!(offset_of!(WireHeader, payload_len) == 4);
};

/// 把逻辑头编成 6 字节小端线上表示。不含载荷。
#[must_use]
pub fn encode_header(kind: FrameKind, payload_len: u16) -> [u8; HEADER_SIZE] {
    let mut out = [0u8; HEADER_SIZE];
    out[0..2].copy_from_slice(&MAGIC.to_le_bytes());
    out[2] = VERSION;
    out[3] = kind as u8;
    out[4..6].copy_from_slice(&payload_len.to_le_bytes());
    out
}

/// 头 + 载荷拼成一帧（测试与 example 用）。分配一次。
#[must_use]
pub fn encode_frame(kind: FrameKind, payload: &[u8]) -> alloc::vec::Vec<u8> {
    let len = u16::try_from(payload.len()).expect("payload fits in u16");
    let mut out = alloc::vec::Vec::with_capacity(HEADER_SIZE + payload.len());
    out.extend_from_slice(&encode_header(kind, len));
    out.extend_from_slice(payload);
    out
}

/// 布局数字，供测试断言（确定性量，FR-003）。
#[must_use]
pub const fn layout() -> (usize, usize, usize) {
    (
        size_of::<WireHeader>(),
        align_of::<WireHeader>(),
        offset_of!(WireHeader, payload_len),
    )
}
