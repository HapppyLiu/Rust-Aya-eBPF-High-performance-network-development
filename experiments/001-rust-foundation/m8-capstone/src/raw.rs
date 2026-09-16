//! unsafe 解析核心与安全封装（C-15…C-20）。
//!
//! 对外函数全部安全：边界检查在 `unsafe` **之外**完成，调用方无法用公开 API 逼出 UB。
//! 每个 `unsafe` 块只含一个 unsafe 操作（clippy `multiple_unsafe_ops_per_block`）。

use crate::buffer::PacketBuf;
use crate::error::ParseError;
use core::mem::align_of;
use core::ptr;
use core::slice;

/// 带边界检查的单字节读取（C-15 / C-16）。越界 → [`ParseError::Truncated`]。
pub fn read_u8(buf: PacketBuf<'_>, offset: usize) -> Result<u8, ParseError> {
    let bytes = buf.as_bytes();
    if offset >= bytes.len() {
        return Err(ParseError::Truncated);
    }
    let p = bytes.as_ptr();
    // SAFETY:
    // - 有效性：`offset < len`，目标是已初始化的 `u8`。
    // - 对齐：`u8` 对齐为 1，任意地址都满足。
    // - 别名：只读；本函数不产生 `&mut`，与输入的共享切片兼容（C-19）。
    // - provenance：`p` 来自 `bytes` 这一次分配，偏移未越界。
    // - 生命周期：`bytes` 活过本调用；返回值是 `u8` 拷贝，不携带引用。
    let q = unsafe { p.add(offset) };
    // SAFETY: `q` 由上一块保证指向分配内已初始化的 `u8`。
    Ok(unsafe { ptr::read(q) })
}

/// 带边界检查的小端 `u16` 读取（C-17 / C-18）。
///
/// 报文缓冲**不**保证按 `u16` 对齐，因此走 [`ptr::read_unaligned`]，
/// 再 [`u16::from_le_bytes`] 固定端序。硬件是否容忍未对齐访问与本函数无关。
pub fn read_u16_le(buf: PacketBuf<'_>, offset: usize) -> Result<u16, ParseError> {
    let bytes = buf.as_bytes();
    let end = offset.checked_add(2).ok_or(ParseError::Truncated)?;
    if end > bytes.len() {
        return Err(ParseError::Truncated);
    }
    let p = bytes.as_ptr();
    // SAFETY:
    // - 有效性：`offset + 2 <= len`，`add(offset)` 仍在分配内（含随后 2 字节）。
    // - 对齐：不适用按 `u16` 对齐的义务——下一步走 `read_unaligned`。
    //   `u8` 指针本身对齐为 1。
    // - 别名：只读，无冲突 `&mut`。
    // - provenance：来自 `bytes`，偏移未越出该分配。
    // - 生命周期：`bytes` 活过本调用。
    let q = unsafe { p.add(offset) };
    // SAFETY:
    // - 有效性：`q` 起连续 2 字节已初始化。
    // - 对齐：`read_unaligned` 卸掉对齐义务（C-18）；有效性仍在。
    // - 别名：只读。
    // - provenance：同一分配内的派生指针，未截断。
    // - 生命周期：立即拷贝成 `[u8; 2]`，不逃逸。
    let arr = unsafe { ptr::read_unaligned(q.cast::<[u8; 2]>()) };
    Ok(u16::from_le_bytes(arr))
}

/// 从 `buf` 切出 `[start, start+len)` 作为新的零拷贝视图（C-20）。
///
/// 检查通过后用 [`slice::from_raw_parts`] 拼切片——与 `&bytes[start..end]` 等价，
/// 但把"长度与指针必须匹配"的义务写进 SAFETY，而不是藏在索引语法里。
pub fn payload_view(
    buf: PacketBuf<'_>,
    start: usize,
    len: usize,
) -> Result<PacketBuf<'_>, ParseError> {
    let bytes = buf.as_bytes();
    let end = start.checked_add(len).ok_or(ParseError::Truncated)?;
    if end > bytes.len() {
        return Err(ParseError::Truncated);
    }
    let p = bytes.as_ptr();
    // SAFETY:
    // - 有效性：`start + len <= bytes.len()`，`add(start)` 指向分配内；
    //   若 `start == len(bytes)` 且 `len == 0`，指向 one-past，随后 `from_raw_parts` 长度为 0，合法。
    // - 对齐：`u8` 对齐为 1。
    // - 别名：返回共享切片，与原 `&[u8]` 只读别名兼容；不产生 `&mut`（C-19）。
    // - provenance：指针来自 `bytes`，未改造成别的分配。
    // - 生命周期：返回引用的寿命与输入 `buf` 相同（`'a`）。
    let q = unsafe { p.add(start) };
    // SAFETY: 上一块加上 `len` 字节仍在 `bytes` 的范围内；元素已初始化。
    let slice = unsafe { slice::from_raw_parts(q, len) };
    Ok(PacketBuf::new(slice))
}

/// 仅当长度偶数**且**起始地址按 `u16` 对齐时，把字节看成 `&[u16]`（C-18 / C-20）。
///
/// 任一条件不满足返回 `None`。调用方无法用本函数拼出未对齐的 `u16` 切片。
pub fn try_as_u16_slice(buf: PacketBuf<'_>) -> Option<&[u16]> {
    let bytes = buf.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    if !(bytes.as_ptr() as usize).is_multiple_of(align_of::<u16>()) {
        return None;
    }
    let ptr = bytes.as_ptr().cast::<u16>();
    let count = bytes.len() / 2;
    // SAFETY:
    // - 有效性：`count * size_of::<u16>() == bytes.len()`，内存已初始化。
    // - 对齐：上一分支已拒绝未对齐起始地址。
    // - 别名：共享切片，只读。
    // - provenance：来自 `bytes`，覆盖范围恰好等于该切片的字节数。
    // - 生命周期：返回引用绑定 `buf` 的寿命。
    Some(unsafe { slice::from_raw_parts(ptr, count) })
}
