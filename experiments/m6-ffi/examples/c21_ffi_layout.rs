//! C-21 布局 —— 两侧 `size_of` / `align_of` / `offset_of`。
//!
//! 运行：`cargo run -p m6-ffi --example c21_ffi_layout`
//! PREDICT-UB: clean（ASan；覆盖面窄于 Miri，见 OBSERVATIONS）

use m6_ffi::c21::{c_layout, rust_layout};

fn main() {
    let rust = rust_layout();
    let c = c_layout();
    println!("=== PacketHdr layout (Rust vs C) ===");
    println!("  size   rust={} c={}", rust.size, c.size);
    println!("  align  rust={} c={}", rust.align, c.align);
    println!("  off.kind rust={} c={}", rust.off_kind, c.off_kind);
    println!("  off.id   rust={} c={}", rust.off_id, c.off_id);
    println!("  off.len  rust={} c={}", rust.off_len, c.off_len);
    println!("  match = {}", rust == c);
}
