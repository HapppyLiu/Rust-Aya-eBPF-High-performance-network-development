//! C-17 安全侧 —— 分配内偏移；`wrapping_add` 在分配内与 `add` 重合。
//!
//! 运行：`cargo run -p m5-unsafe --example c17_ptr_arith`
//! PREDICT-UB: clean

use m5_unsafe::c17::{
    add_matches_wrapping_one_step, offset_matches_add, one_step_byte_distance, read_at,
};

fn main() {
    println!("=== 分配内 add / offset / wrapping_add ===");
    println!("  read_at(1) = {:?}", read_at(1));
    println!(
        "  add == wrapping_add (one step) = {}",
        add_matches_wrapping_one_step()
    );
    println!("  offset == add = {}", offset_matches_add());
    println!("  one-step bytes = {}", one_step_byte_distance());
    let a = [10u32, 20, 30];
    let far = a.as_ptr().wrapping_add(100);
    println!("  wrapping_add(100) produced a pointer (do not deref): {far:?}");
}
