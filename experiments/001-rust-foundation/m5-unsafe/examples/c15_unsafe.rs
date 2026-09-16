//! C-15 安全侧 —— 边界内 `get_unchecked`。
//!
//! 运行：`cargo run -p m5-unsafe --example c15_unsafe`
//! PREDICT-UB: clean

use m5_unsafe::c15::get_in_bounds;

fn main() {
    let data = [10u8, 20, 30, 40];
    println!("=== 边界内 get_unchecked ===");
    println!("  idx 2 = {:?}", get_in_bounds(&data, 2));
    println!("  idx 9 (checked miss) = {:?}", get_in_bounds(&data, 9));
}
