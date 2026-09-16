//! C-19 Aliasing —— `UnsafeCell` 上的合法内部可变性。

use std::cell::UnsafeCell;

/// 通过 `UnsafeCell::get` 顺序写两次，没有两份同时活着的 `&mut`。
#[must_use]
pub fn write_twice(start: i32, first: i32, second: i32) -> i32 {
    let cell = UnsafeCell::new(start);
    let p = cell.get();
    // SAFETY:
    // - 有效性：`p` 指向 `cell` 内的 `i32`，分配仍在，即将写入完整的 `i32`。
    // - 对齐：`UnsafeCell<i32>` 透明包装，`i32` 对齐由局部变量保证。
    // - 别名：此时没有任何指向内部的引用；`get` 只交出裸指针。
    //   本块只做一次写，下一次写在下一个块，两份 `&mut` 不同时存在。
    // - provenance：来自 `cell` 这一次分配，无越界偏移。
    // - 生命周期：`cell` 活过本函数。
    unsafe { *p = first };
    // SAFETY: 同上；上一块的写已经结束，没有残留的引用。
    unsafe { *p = second };
    // SAFETY: 读回；仍无冲突引用，内存由上一块初始化。
    unsafe { *p }
}
