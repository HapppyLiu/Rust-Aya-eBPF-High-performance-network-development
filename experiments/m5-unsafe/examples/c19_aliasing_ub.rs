//! C-19 UB 对照 —— 两份可变引用叠在同一块内存上。
//!
//! 默认 Stacked Borrows 与 Tree Borrows 各跑一轮（见 tests）。
//!
//! 运行：`cargo run -p m5-unsafe --example c19_aliasing_ub`
//! PREDICT-UB: W1 + W8（SB）；W1 + W9（TB）

fn main() {
    let mut x = 0u8;
    let raw = &mut x as *mut u8;
    // SAFETY:
    // - 有效性：`raw` 指向仍活着的 `x`。
    // - 对齐：`u8` 对齐为 1。
    // - 别名：本块只造第一份 `&mut`。
    // - provenance：来自 `x`。
    // - 生命周期：`x` 活过 `main`。
    let a = unsafe { &mut *raw };
    // SAFETY:
    // - 有效性 / 对齐 / provenance / 生命周期：同上。
    // - 别名：故意不成立。`a` 仍活着，又造了第二份 `&mut` 指向同一位置。
    let b = unsafe { &mut *raw };
    *a = 1;
    *b = 2;
    println!("overlapping mut writes (NON-ASSERTION) = {x}");
}
