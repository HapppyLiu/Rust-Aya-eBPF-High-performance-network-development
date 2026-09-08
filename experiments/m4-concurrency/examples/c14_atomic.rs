//! C-14 Atomic —— 非断言观察输出。
//!
//! 运行：`cargo run -p m4-concurrency --example c14_atomic`
//!
//! 放宽内存序之后哪些顺序变为可能：语言规则允许"旗标已立、载荷仍是 0"。
//! 本机 x86_64 的 TSO 常常把它掩盖掉 —— 一次打印看见 42 **不能**证明弱序安全。

use m4_concurrency::c14::{relaxed_handshake_seen, release_acquire_seen, seqcst_sum};

fn main() {
    println!("=== 1. SeqCst 累加：终值确定 ===");
    println!("  seqcst 4×25 = {}", seqcst_sum(4, 25));

    println!();
    println!("=== 2. Release/Acquire 握手：看见旗标就看见载荷 ===");
    println!("  acquire seen = {}", release_acquire_seen());

    println!();
    println!("=== 3. 两边 Relaxed：语言允许旧载荷，本机可能仍打印 42 ===");
    println!(
        "  relaxed seen (NON-ASSERTION) = {}",
        relaxed_handshake_seen()
    );
    println!("  若这里是 42，只能说明这一次没观察到弱序窗口，不能推广到 aarch64。");
}
