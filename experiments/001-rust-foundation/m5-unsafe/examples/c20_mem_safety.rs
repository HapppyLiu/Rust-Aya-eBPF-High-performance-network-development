//! C-20 安全侧 —— 对外安全的切片拼装与边界检查解析。
//!
//! 运行：`cargo run -p m5-unsafe --example c20_mem_safety`
//! PREDICT-UB: clean

use m5_unsafe::c20::{AlignedBytes, as_u16_slice, parse_u16_be_checked};

fn main() {
    let packed = AlignedBytes::demo();
    let buf = &packed.0;
    println!("=== safe parse / from_raw_parts wrapper ===");
    println!("  parse off 0 = {:?}", parse_u16_be_checked(buf, 0));
    println!("  parse off 3 (miss) = {:?}", parse_u16_be_checked(buf, 3));
    println!(
        "  as_u16_slice len = {:?}",
        as_u16_slice(buf).map(<[u16]>::len)
    );
    println!(
        "  odd len rejected = {:?}",
        as_u16_slice(&[1, 2, 3]).map(<[u16]>::len)
    );
}
