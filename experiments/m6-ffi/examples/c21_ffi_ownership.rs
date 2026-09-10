//! C-21 所有权 —— C malloc，Rust free。
//!
//! 运行：`cargo run -p m6-ffi --example c21_ffi_ownership`
//! PREDICT-UB: clean（ASan；覆盖面窄于 Miri，见 OBSERVATIONS）

use m6_ffi::c21::{OWNED_MSG, take_c_message};

fn main() {
    let got = take_c_message();
    println!("=== C alloc / Rust free ===");
    println!("  got = {got:?}");
    println!("  matches convention literal = {}", got == OWNED_MSG);
    println!("  owner-to-free = Rust (libc::free pairs with C malloc)");
}
