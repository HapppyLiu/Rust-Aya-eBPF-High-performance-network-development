//! C-16 Raw pointer —— `addr_of!` / `ptr::read` / `ptr::write` 的正确用法。

use std::ptr;

/// 对栈上的 `i32` 取址、写入、读回。返回写进去的值。
///
/// 不断言具体地址数值，只保证"写进去的能读回来"。
#[must_use]
pub fn write_then_read(start: i32, next: i32) -> i32 {
    let mut x = start;
    let p = ptr::addr_of_mut!(x);
    // SAFETY:
    // - 有效性：`p` 指向仍然活着的栈变量 `x`，大小为 `size_of::<i32>()`。
    // - 对齐：`x` 是独立的 `i32` 局部变量，满足 `align_of::<i32>()`。
    // - 别名：本语句期间没有同时存在的 `&` / `&mut`；`addr_of_mut!` 不创建引用。
    // - provenance：`p` 来自 `x` 这一次分配，未做偏移。
    // - 生命周期：`x` 活过本函数；`p` 不逃出本函数。
    unsafe { ptr::write(p, next) };
    // SAFETY: 同上；`write` 刚把 `next` 写入，内存已初始化。
    unsafe { ptr::read(p) }
}

/// 对同一位置取两次址。调用方比较两个指针是否相等（关系，不是具体数值）。
#[must_use]
pub fn addrs_of(x: &i32) -> (*const i32, *const i32) {
    let p = ptr::addr_of!(*x);
    let q = ptr::addr_of!(*x);
    (p, q)
}
