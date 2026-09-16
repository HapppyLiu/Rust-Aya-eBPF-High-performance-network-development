//! C-20 Memory safety —— 由 unsafe 实现、对外暴露**安全**接口的最小抽象。

use std::mem::align_of;
use std::slice;

/// 带边界检查的大端 `u16` 读取。越界返回 `None`，调用方无法逼出 UB。
#[must_use]
pub fn parse_u16_be_checked(buf: &[u8], off: usize) -> Option<u16> {
    let bytes = buf.get(off..off.checked_add(2)?)?;
    Some(u16::from_be_bytes([bytes[0], bytes[1]]))
}

/// 调用方**已经**保证 `off + 2 <= buf.len()` 时的裸指针路径。
///
/// # Safety
///
/// `off + 2 <= buf.len()`。本函数不对外暴露；对外入口是 [`parse_u16_be_checked`]。
#[must_use]
pub unsafe fn parse_u16_be_unchecked(buf: &[u8], off: usize) -> u16 {
    let p = buf.as_ptr();
    // SAFETY:
    // - 有效性：调用方保证 `off + 2 <= buf.len()`，`add(off)` 仍在分配内。
    // - 对齐：`u8` 对齐为 1。
    // - 别名：只读。
    // - provenance：来自 `buf`。
    // - 生命周期：`buf` 活过本调用。
    let p0 = unsafe { p.add(off) };
    // SAFETY: 同上，目标字节已初始化。
    let b0 = unsafe { *p0 };
    // SAFETY: 调用方保证 `off + 1` 仍在分配内。
    let p1 = unsafe { p.add(off + 1) };
    // SAFETY: 目标字节已初始化。
    let b1 = unsafe { *p1 };
    u16::from_be_bytes([b0, b1])
}

/// 起始地址按 `u16` 对齐的 4 字节演示缓冲。`[u8; N]` 本身对齐为 1，
/// 栈上不一定满足 `align_of::<u16>()`，所以用 `repr(align(2))` 钉住。
#[repr(C, align(2))]
pub struct AlignedBytes<const N: usize>(pub [u8; N]);

impl AlignedBytes<4> {
    #[must_use]
    pub fn demo() -> Self {
        Self([0x12, 0x34, 0x56, 0x78])
    }
}

/// 把长度偶数、且起始地址按 `u16` 对齐的字节切片，看成 `&[u16]`。
///
/// 任一前置条件不满足则返回 `None` —— 调用方无法用公开 API 拼出 UB。
#[must_use]
pub fn as_u16_slice(bytes: &[u8]) -> Option<&[u16]> {
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    if !(bytes.as_ptr() as usize).is_multiple_of(align_of::<u16>()) {
        return None;
    }
    let ptr = bytes.as_ptr().cast::<u16>();
    let len = bytes.len() / 2;
    // SAFETY:
    // - 有效性：`bytes` 活着且已初始化；`len * size_of::<u16>() == bytes.len()`。
    // - 对齐：上一分支已拒绝未对齐起始地址。
    // - 别名：返回共享切片，与原 `&[u8]` 只读别名兼容；不产生 `&mut`。
    // - provenance：指针来自 `bytes` 的分配，覆盖范围恰好等于该分配的字节数。
    // - 生命周期：返回引用的寿命绑定 `bytes`（`'a` 与输入相同）。
    Some(unsafe { slice::from_raw_parts(ptr, len) })
}
