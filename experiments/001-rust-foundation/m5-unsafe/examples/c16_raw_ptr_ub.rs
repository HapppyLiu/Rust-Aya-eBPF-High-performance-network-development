//! C-16 UB 对照 —— 释放后再读（use-after-free）。
//!
//! 运行：`cargo run -p m5-unsafe --example c16_raw_ptr_ub`
//! PREDICT-UB: W1 + W6

use std::ptr;

fn main() {
    let p = Box::into_raw(Box::new(7i32));
    // SAFETY:
    // - 有效性：`p` 此刻仍指向 `into_raw` 交出的那块堆内存，尚未释放。
    // - 对齐：`Box<i32>` 的分配满足 `i32` 对齐。
    // - 别名：`into_raw` 之后没有别的所有者；本块把它唯一地拼回去。
    // - provenance：来自刚才那次 `Box` 分配。
    // - 生命周期：拼回 `Box` 后由 `drop` 结束该分配的寿命。
    unsafe { drop(Box::from_raw(p)) };
    // SAFETY:
    // - 有效性：故意不成立。上一块已经释放。
    // - 对齐：地址数值可能仍对齐，但分配已不在。
    // - 别名：不适用（指向的对象已不存在）。
    // - provenance：指针还记得旧分配，但该分配已结束。
    // - 生命周期：故意在寿命结束后使用。
    let v = unsafe { ptr::read(p) };
    println!("use-after-free read (NON-ASSERTION) = {v}");
}
