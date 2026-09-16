//! C-17 UB 对照 —— `add` 越出分配（不必解引用，算出指针即犯规）。
//!
//! 运行：`cargo run -p m5-unsafe --example c17_ptr_arith_ub`
//! PREDICT-UB: W1 + W7

fn main() {
    let a = [10u32, 20, 30];
    let p = a.as_ptr();
    // SAFETY:
    // - 有效性：故意不成立。分配只有 3 个 `u32`，`add(8)` 越出可访问范围。
    // - 对齐：步进按元素大小，对齐本身不是本实验要打破的。
    // - 别名：不适用（没有解引用）。
    // - provenance：故意越出 `a` 这次分配的范围。文档写明 `add` 越界本身就是 UB。
    // - 生命周期：`a` 仍活着；打破的是 provenance，不是寿命。
    let q = unsafe { p.add(8) };
    println!("oob add pointer (NON-ASSERTION) = {q:?}");
}
