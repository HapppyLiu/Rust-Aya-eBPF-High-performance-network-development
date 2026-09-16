//! # m8-capstone —— 面向字节缓冲区的最小报文解析器（US8 / C-01…C-24）
//!
//! 库 crate 是 `#![no_std]` + `alloc`：语言原语与堆集合可用，文件 / 线程 / 环境变量
//! 不可用。测试 crate 才链 `std`（起线程、注册 [`rf_harness::counting_alloc`]）。
//!
//! | 分层 | 文件 | 能力 |
//! |------|------|------|
//! | 所有权 / 寿命 | [`buffer`] | C-01…C-04 |
//! | 类型 / trait / 泛型 | [`parse`] | C-05…C-07 |
//! | 错误 / 迭代 / 智能指针 | [`error`] / [`iter`] | C-08…C-11 |
//! | 并发 / 原子 | [`stats`] | C-12…C-14 |
//! | unsafe 安全封装 | [`raw`] | C-15…C-20 |
//! | 布局 / FFI 定位 | [`wire`] | C-21 |
//! | 本文件的 crate 属性 | crate root | C-22…C-24 |
//!
//! 稳定断言在 `tests/`。本文件不打印、不断言（R-05）。

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub mod buffer;
pub mod error;
pub mod iter;
pub mod parse;
pub mod raw;
pub mod stats;
pub mod wire;

pub use buffer::{OwnedBytes, PacketBuf};
pub use error::ParseError;
pub use iter::FrameIter;
pub use parse::{Frame, HeaderParser, MagicParser, ParseHeader, parse_frame};
pub use stats::ParseStats;
pub use wire::{FrameKind, HEADER_SIZE, MAGIC, VERSION, WireHeader};
