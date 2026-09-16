//! C-18 安全侧 —— 对齐关系式与 `read_unaligned`。
//!
//! 运行：`cargo run -p m5-unsafe --example c18_alignment`
//! PREDICT-UB: clean

use m5_unsafe::c18::{aligned_addr_mod, read_aligned, read_unaligned_u16, u64_align};

fn main() {
    println!("=== align_of / aligned read / read_unaligned ===");
    println!("  align_of::<u64>() = {}", u64_align());
    println!("  aligned addr % align = {}", aligned_addr_mod());
    println!("  read_aligned(7) = {}", read_aligned(7));
    println!("  read_unaligned u16 at +1 = {:#06x}", read_unaligned_u16());
}
