//! C-20 UB 对照 —— 谎报长度暴露未初始化内存；以及越界切片。
//!
//! 本文件演示"对外看起来像安全接口、其实一用就穿帮"的调用序列（US5 AS3 后半）。
//!
//! 运行：`cargo run -p m5-unsafe --example c20_mem_safety_ub`
//! PREDICT-UB: W1 + W10

#[allow(clippy::uninit_vec)] // 本函数就是要演示"封装撒谎"：clippy 指出的正是要教的 UB
fn lying_bytes(cap: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(cap);
    // SAFETY:
    // - 有效性：故意不成立。`set_len` 只改长度，不初始化新露出来的槽。
    // - 对齐：`Vec<u8>` 的堆分配满足 `u8` 对齐，本项不是要打破的。
    // - 别名：此时没有指向这些槽的引用。
    // - provenance：分配来自 `Vec`，容量够；撒谎的是"这些字节已经写过"。
    // - 生命周期：`v` 仍活着。
    // 这个函数**没有**标 `unsafe`，所以调用方看不出炸弹 —— 这正是抽象在撒谎。
    unsafe { v.set_len(cap) };
    v
}

fn main() {
    let v = lying_bytes(4);
    // 调用方没做错：拿到 `Vec<u8>` 就按普通切片读。问题在封装。
    let first = v[0];
    println!("uninit via lying Vec (NON-ASSERTION) = {first}");
}
