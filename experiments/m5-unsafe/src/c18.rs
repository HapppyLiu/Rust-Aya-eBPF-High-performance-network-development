//! C-18 Alignment —— `align_of`、对齐关系式、`read_unaligned`。

use std::mem::align_of;
use std::ptr;

/// `u64` 的 ABI 对齐要求（类型属性，不是某块内存的实测）。
#[must_use]
pub fn u64_align() -> usize {
    align_of::<u64>()
}

/// 独立的 `u64` 局部变量，地址对 `align_of::<u64>()` 取模为 0。
#[must_use]
pub fn aligned_addr_mod() -> usize {
    let x = 7u64;
    let p = ptr::addr_of!(x);
    (p as usize) % align_of::<u64>()
}

/// 对齐满足时用普通 `read` 读回原值。
#[must_use]
pub fn read_aligned(value: u64) -> u64 {
    let x = value;
    let p = ptr::addr_of!(x);
    // SAFETY:
    // - 有效性：`p` 指向活着的 `x`，已初始化。
    // - 对齐：`x` 是独立 `u64`，满足 `align_of::<u64>()`（由 `aligned_addr_mod` 钉住关系）。
    // - 别名：只读，无 `&mut`。
    // - provenance：来自 `x`，无偏移。
    // - 生命周期：`x` 活过本函数；返回拷贝。
    unsafe { ptr::read(p) }
}

/// 故意造一个未对齐地址，用 `read_unaligned` 读两个字节拼成的载荷。
///
/// 载荷放在偏移 1 处，避免和"对齐读"混为一谈。
#[must_use]
pub fn read_unaligned_u16() -> u16 {
    let buf = [0u8, 0x34, 0x12, 0];
    let p = buf.as_ptr().wrapping_add(1).cast::<u16>();
    // SAFETY:
    // - 有效性：`buf` 长 4，偏移 1 起有 2 个已初始化字节。
    // - 对齐：不适用 —— `read_unaligned` 明确不要求对齐；本指针对 `u16` 很可能未对齐。
    // - 别名：只读。
    // - provenance：来自 `buf`，偏移 1 仍在分配内。
    // - 生命周期：`buf` 活过本函数；返回拷贝。
    unsafe { ptr::read_unaligned(p) }
}
