//! C-16 安全侧 —— 取址、写入、读回。
//!
//! 运行：`cargo run -p m5-unsafe --example c16_raw_ptr`
//! PREDICT-UB: clean

use m5_unsafe::c16::{addrs_of, write_then_read};

fn main() {
    println!("=== addr_of / read / write ===");
    let saw = write_then_read(1, 9);
    println!("  write 9 then read = {saw}");
    let x = 4;
    let (p, q) = addrs_of(&x);
    println!("  two addr_of same place = {}", p == q);
}
