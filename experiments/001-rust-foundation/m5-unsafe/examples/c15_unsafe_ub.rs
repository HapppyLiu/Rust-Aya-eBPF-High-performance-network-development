//! C-15 UB 对照 —— `get_unchecked` 越界。
//!
//! 运行：`cargo run -p m5-unsafe --example c15_unsafe_ub`
//! 同一源码：`cargo +nightly miri run -p m5-unsafe --example c15_unsafe_ub`
//!
//! PREDICT-UB: W1 + W2

fn main() {
    let data = [10u8, 20, 30];
    // SAFETY:
    // - 有效性：故意不成立。`3 == data.len()`，文档写明即便结果不用也已经犯规。
    // - 对齐：`u8` 对齐为 1，本项不是本实验要打破的。
    // - 别名：只读，本项不是本实验要打破的。
    // - provenance：指针仍来自 `data`，但索引越出了该分配的可访问范围。
    // - 生命周期：`data` 仍活着；打破的是有效性 / provenance，不是寿命。
    let r = unsafe { data.get_unchecked(3) };
    println!("oob get_unchecked (NON-ASSERTION) = {r}");
}
