//! C-19 安全侧 —— `UnsafeCell` 合法内部可变。
//!
//! 运行：`cargo run -p m5-unsafe --example c19_aliasing`
//! PREDICT-UB: clean

use m5_unsafe::c19::write_twice;

fn main() {
    println!("=== UnsafeCell sequential writes ===");
    println!("  write_twice(0, 1, 2) = {}", write_twice(0, 1, 2));
}
