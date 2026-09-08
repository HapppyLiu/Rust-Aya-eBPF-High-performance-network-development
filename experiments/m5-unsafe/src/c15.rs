//! C-15 Unsafe Rust —— 边界内 `get_unchecked` 的正确用法。
//!
//! `unsafe` 打开的是**义务**：调用方保证下标合法。编译器不再代劳边界检查。

/// 先用安全 API 确认 `idx` 合法，再走不检查边界的路径。
///
/// 返回 `None` 表示下标越界 —— 那条路径根本不会进入 `unsafe`。
#[must_use]
pub fn get_in_bounds(slice: &[u8], idx: usize) -> Option<u8> {
    let _ = slice.get(idx)?;
    // SAFETY:
    // - 有效性：`slice.get(idx)` 刚返回 `Some`，故 `idx < slice.len()`，
    //   指向的字节已初始化且分配仍在。
    // - 对齐：`u8` 对齐为 1，任意地址都满足。
    // - 别名：只产生共享引用，与已有 `&[u8]` 兼容，无冲突的 `&mut`。
    // - provenance：指针来自 `slice` 这一次分配，未越出其范围。
    // - 生命周期：返回的 `u8` 是拷贝，不延长引用；`get_unchecked` 的临时引用
    //   不逃出本语句。
    Some(*unsafe { slice.get_unchecked(idx) })
}
