//! C-18 UB 对照 —— 本 Feature 的核心教学对照。
//!
//! 普通运行：正常退出并打印"合理"结果。
//! 同一源码交给 Miri：判定 UB。
//!
//! 运行：`cargo run -p m5-unsafe --example c18_alignment_ub`
//! 同一源码：`cargo +nightly miri run -p m5-unsafe --example c18_alignment_ub`
//!
//! PREDICT-UB: W1 + W2

use std::ptr;

fn main() {
    let mut buf = [0u8; 8];
    // SAFETY:
    // - 有效性：`buf` 长 8，`add(1)` 仍在分配内（本步只造指针）。
    // - 对齐：`u8` 对齐为 1。
    // - 别名：只构造指针。
    // - provenance：来自 `buf`。
    // - 生命周期：`buf` 活过 `main`。
    let p = unsafe { buf.as_mut_ptr().add(1) };
    let p64 = p.cast::<u64>();
    // SAFETY:
    // - 有效性：故意不成立。偏移 1 起只剩 7 字节，写 8 字节越出分配。
    // - 对齐：故意不成立。起始地址 +1 通常不满足 `u64` 对齐。
    //   Miri 1.100 会**先**报越界（W2），未对齐是同一操作的另一面。
    // - 别名：此时没有指向这些字节的引用。
    // - provenance：写越出了 `buf` 的可访问范围。
    // - 生命周期：`buf` 仍活着。
    // 本块在 x86_64 上常常"能跑完"并打印 2。那不是合法的证据。
    unsafe { ptr::write(p64, 2u64) };
    // SAFETY: 与上一块相同的故意未对齐；读回那个"看起来合理"的 2。
    let v = unsafe { ptr::read(p64) };
    println!("{v}");
}
