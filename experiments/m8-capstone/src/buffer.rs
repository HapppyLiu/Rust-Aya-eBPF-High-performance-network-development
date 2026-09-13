//! 所有权与生命周期分层（C-01…C-04）。
//!
//! - [`OwnedBytes`] 独占一块 `Vec<u8>`（C-01 / C-02：拥有、可移动、非 `Copy`）。
//! - [`PacketBuf`] 是零拷贝视图：只保存 `&'a [u8]`（C-03 / C-04：借用 + 寿命钉在输入切片上）。

use crate::error::ParseError;
use alloc::vec::Vec;

/// 零拷贝字节视图。不拥有内存，因此 `Copy`：复制的是指针与长度，不是载荷。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketBuf<'a> {
    bytes: &'a [u8],
}

impl<'a> PacketBuf<'a> {
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self { bytes: &[] }
    }

    #[must_use]
    pub const fn len(self) -> usize {
        self.bytes.len()
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.bytes.is_empty()
    }

    #[must_use]
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }

    /// 切成 `[0, mid)` 与 `[mid, len)`。`mid` 越界 → [`ParseError::Truncated`]。
    pub fn split_at(self, mid: usize) -> Result<(Self, Self), ParseError> {
        if mid > self.bytes.len() {
            return Err(ParseError::Truncated);
        }
        let (head, tail) = self.bytes.split_at(mid);
        Ok((Self::new(head), Self::new(tail)))
    }

    /// 丢掉前 `n` 字节。越界 → [`ParseError::Truncated`]。
    pub fn advance(self, n: usize) -> Result<Self, ParseError> {
        Ok(self.split_at(n)?.1)
    }
}

/// 独占拥有一段字节。移动后原绑定失效；借用必须短于拥有者（C-01 / C-02 / C-04）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnedBytes {
    inner: Vec<u8>,
}

impl OwnedBytes {
    #[must_use]
    pub fn from_slice(bytes: &[u8]) -> Self {
        Self {
            inner: bytes.to_vec(),
        }
    }

    #[must_use]
    pub fn as_buf(&self) -> PacketBuf<'_> {
        PacketBuf::new(&self.inner)
    }

    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.inner
    }
}
