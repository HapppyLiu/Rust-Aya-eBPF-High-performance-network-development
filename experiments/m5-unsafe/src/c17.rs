//! C-17 Pointer arithmetic —— 分配内 `add` / `offset`，对照 `wrapping_add`。

use std::ptr;

/// 在 `[u32; 3]` 内走 `index` 步（`index < 3`），读到的元素。
#[must_use]
pub fn read_at(index: usize) -> Option<u32> {
    if index >= 3 {
        return None;
    }
    let a = [10u32, 20, 30];
    let p = a.as_ptr();
    // SAFETY:
    // - 有效性：`index < 3`，目标仍在 `a` 的三个元素之内，已初始化。
    // - 对齐：`u32` 数组元素按 `align_of::<u32>()` 排列；`add` 按元素步进，保持对齐。
    // - 别名：只读，无冲突的 `&mut`。
    // - provenance：`p` 来自 `a`；偏移未越出该分配（含末尾 one-past 之内的合法元素）。
    // - 生命周期：`a` 活过本函数；返回值是 `u32` 拷贝。
    let q = unsafe { p.add(index) };
    // SAFETY: `q` 由上一块保证指向已初始化的 `u32`。
    Some(unsafe { ptr::read(q) })
}

/// 分配内走一步：`add(1)` 与 `wrapping_add(1)` 得到同一指针。
///
/// `wrapping_add` 本身永远安全；这里只在分配内走一步，随后的读才需要 SAFETY。
#[must_use]
pub fn add_matches_wrapping_one_step() -> bool {
    let a = [10u32, 20, 30];
    let p = a.as_ptr();
    // SAFETY: 一步仍在三个元素之内，见 `read_at`。
    let added = unsafe { p.add(1) };
    let wrapped = p.wrapping_add(1);
    added == wrapped
}

/// 分配内走一步的字节差是否等于 `size_of::<u32>()`（地址的关系性质）。
#[must_use]
pub fn one_step_byte_distance() -> usize {
    let a = [10u32, 20, 30];
    let p = a.as_ptr();
    // SAFETY: 一步仍在分配内。
    let q = unsafe { p.add(1) };
    (q as usize).wrapping_sub(p as usize)
}

/// `offset(1)` 与 `add(1)` 在分配内重合。
#[must_use]
pub fn offset_matches_add() -> bool {
    let a = [10u32, 20, 30];
    let p = a.as_ptr();
    // SAFETY: 正一格仍在分配内。
    let a1 = unsafe { p.add(1) };
    // SAFETY: 同上，`offset` 的 `count` 为 +1。
    let o1 = unsafe { p.offset(1) };
    a1 == o1
}
