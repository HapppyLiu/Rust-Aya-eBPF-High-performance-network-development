//! 迭代器与智能指针分层（C-09…C-11）。

use crate::buffer::PacketBuf;
use crate::error::ParseError;
use crate::parse::{Frame, MagicParser, ParseHeader, parse_frame};
use crate::stats::ParseStats;
use crate::wire::MAGIC;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// 在一段缓冲上迭代完整帧。空缓冲结束；中途失败产出 `Err` 后停止。
pub struct FrameIter<'a> {
    rest: PacketBuf<'a>,
    stats: Option<&'a ParseStats>,
}

impl<'a> FrameIter<'a> {
    #[must_use]
    pub fn new(buf: PacketBuf<'a>) -> Self {
        Self {
            rest: buf,
            stats: None,
        }
    }

    #[must_use]
    pub fn with_stats(buf: PacketBuf<'a>, stats: &'a ParseStats) -> Self {
        Self {
            rest: buf,
            stats: Some(stats),
        }
    }
}

impl<'a> Iterator for FrameIter<'a> {
    type Item = Result<Frame<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            return None;
        }
        match parse_frame(self.rest) {
            Ok((frame, rest)) => {
                self.rest = rest;
                if let Some(stats) = self.stats {
                    stats.record_ok();
                }
                Some(Ok(frame))
            }
            Err(e) => {
                self.rest = PacketBuf::empty();
                if let Some(stats) = self.stats {
                    stats.record_err();
                }
                Some(Err(e))
            }
        }
    }
}

/// 对成功解析的载荷应用闭包（C-10）。失败则停止。收集结果会堆分配（C-24 对照）。
pub fn map_ok_payloads<'a, F, R>(iter: FrameIter<'a>, mut f: F) -> Vec<R>
where
    F: FnMut(PacketBuf<'a>) -> R,
{
    let mut out = Vec::new();
    for item in iter {
        match item {
            Ok(frame) => out.push(f(frame.payload)),
            Err(_) => break,
        }
    }
    out
}

/// 堆上的头解析器（C-11）：`Box` 独占带状态的 `dyn ParseHeader`。
///
/// 装箱的是 [`MagicParser`] 而不是零大小的 [`crate::parse::HeaderParser`]：
/// ZST 的 `Box` 不向分配器要内存，无法演示"独占一块堆"。
pub fn boxed_header_parser() -> Box<dyn ParseHeader> {
    Box::new(MagicParser {
        expected_magic: MAGIC,
    })
}
