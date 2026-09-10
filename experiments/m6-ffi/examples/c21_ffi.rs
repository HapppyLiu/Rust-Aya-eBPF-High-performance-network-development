//! C-21 双向调用 —— Rust→C 与 C→Rust 分开观察。
//!
//! 运行：`cargo run -p m6-ffi --example c21_ffi`
//! PREDICT-UB: clean（ASan；覆盖面窄于 Miri，见 OBSERVATIONS）

use m6_ffi::c21::{
    PacketHdr, c_filled_hdr, c_to_rust_hdr_sum, c_to_rust_mul, rust_to_c_add, rust_to_c_hdr_sum,
};

fn main() {
    println!("=== Rust → C ===");
    println!("  c_add(6, 7) = {}", rust_to_c_add(6, 7));
    let hdr = PacketHdr {
        kind: 1,
        id: 2,
        len: 3,
    };
    println!("  c_hdr_sum(1,2,3) = {}", rust_to_c_hdr_sum(&hdr));

    println!("=== C → Rust ===");
    println!("  rust_mul via C (6, 7) = {}", c_to_rust_mul(6, 7));
    println!("  rust_hdr_sum via C (1,2,3) = {}", c_to_rust_hdr_sum());

    let filled = c_filled_hdr();
    println!(
        "=== C filled hdr ===\n  kind={} id={} len={}",
        filled.kind, filled.id, filled.len
    );
}
