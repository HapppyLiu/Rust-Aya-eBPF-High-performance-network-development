//! 错误分层（C-08）：失败是带标签的枚举，不是碰巧叫 errno 的整数。

use core::fmt;

/// 解析失败。每条路径对应一种可断言的原因，禁止用"出错了"合并。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// 剩余字节不够读完头或载荷。
    Truncated,
    /// 魔数不是 [`crate::wire::MAGIC`]。
    BadMagic,
    /// 版本不是 [`crate::wire::VERSION`]。
    BadVersion,
    /// `kind` 字节不是已知的 [`crate::wire::FrameKind`]。
    BadKind,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => f.write_str("truncated"),
            Self::BadMagic => f.write_str("bad magic"),
            Self::BadVersion => f.write_str("bad version"),
            Self::BadKind => f.write_str("bad kind"),
        }
    }
}

impl core::error::Error for ParseError {}
