//! trait 抽象与泛型分层（C-05…C-07）。
//!
//! [`ParseHeader`] 不带关联类型上的输入寿命，因此可以做成 [`dyn ParseHeader`]（C-06）。
//! 完整帧 [`Frame<'a>`] 携带载荷视图的寿命，适合泛型组合器，不适合直接 `dyn`。

use crate::buffer::PacketBuf;
use crate::error::ParseError;
use crate::raw;
use crate::wire::{FrameKind, HEADER_SIZE, MAGIC, VERSION, WireHeader};

/// 一帧：逻辑头 + 零拷贝载荷视图。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame<'a> {
    pub header: WireHeader,
    pub payload: PacketBuf<'a>,
}

impl<'a> Frame<'a> {
    pub fn kind(self) -> Result<FrameKind, ParseError> {
        self.header.kind()
    }
}

/// 只解析 6 字节头。对象安全（C-06 的 `dyn` 落点）。
pub trait ParseHeader {
    fn parse_header<'a>(
        &self,
        buf: PacketBuf<'a>,
    ) -> Result<(WireHeader, PacketBuf<'a>), ParseError>;
}

/// 默认头解析器：魔数 / 版本 / 种类 / 长度。零大小类型，`Box` 它不会堆分配。
#[derive(Clone, Copy, Debug, Default)]
pub struct HeaderParser;

impl ParseHeader for HeaderParser {
    fn parse_header<'a>(
        &self,
        buf: PacketBuf<'a>,
    ) -> Result<(WireHeader, PacketBuf<'a>), ParseError> {
        MagicParser {
            expected_magic: MAGIC,
        }
        .parse_header(buf)
    }
}

/// 带状态的头解析器。字段占用空间，因此 `Box<MagicParser>` 会堆分配（C-11）。
#[derive(Clone, Copy, Debug)]
pub struct MagicParser {
    pub expected_magic: u16,
}

impl ParseHeader for MagicParser {
    fn parse_header<'a>(
        &self,
        buf: PacketBuf<'a>,
    ) -> Result<(WireHeader, PacketBuf<'a>), ParseError> {
        if buf.len() < HEADER_SIZE {
            return Err(ParseError::Truncated);
        }
        let magic = raw::read_u16_le(buf, 0)?;
        if magic != self.expected_magic {
            return Err(ParseError::BadMagic);
        }
        let version = raw::read_u8(buf, 2)?;
        if version != VERSION {
            return Err(ParseError::BadVersion);
        }
        let kind = raw::read_u8(buf, 3)?;
        FrameKind::from_wire(kind)?;
        let payload_len = raw::read_u16_le(buf, 4)?;
        let header = WireHeader {
            magic,
            version,
            kind,
            payload_len,
        };
        let rest = buf.advance(HEADER_SIZE)?;
        Ok((header, rest))
    }
}

/// 泛型组合器（C-07）：先跑一个 [`ParseHeader`]，再把头和剩余缓冲交给闭包（C-10）。
pub fn parse_then<'a, P, F, T>(
    parser: &P,
    buf: PacketBuf<'a>,
    then: F,
) -> Result<(T, PacketBuf<'a>), ParseError>
where
    P: ParseHeader + ?Sized,
    F: FnOnce(WireHeader, PacketBuf<'a>) -> Result<(T, PacketBuf<'a>), ParseError>,
{
    let (header, rest) = parser.parse_header(buf)?;
    then(header, rest)
}

/// trait 对象入口（C-06）：分发落到 vtable，不单态化某个具体解析器。
pub fn parse_header_dyn<'a>(
    parser: &dyn ParseHeader,
    buf: PacketBuf<'a>,
) -> Result<(WireHeader, PacketBuf<'a>), ParseError> {
    parser.parse_header(buf)
}

/// 解析一帧：头 + 恰好 `payload_len` 字节的载荷，返回帧与剩余缓冲。
pub fn parse_frame(buf: PacketBuf<'_>) -> Result<(Frame<'_>, PacketBuf<'_>), ParseError> {
    parse_then(&HeaderParser, buf, |header, rest| {
        let n = usize::from(header.payload_len);
        let payload = raw::payload_view(rest, 0, n)?;
        let rest = rest.advance(n)?;
        Ok((Frame { header, payload }, rest))
    })
}
