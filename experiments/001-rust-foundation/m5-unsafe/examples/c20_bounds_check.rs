//! C-20 边界检查对照（T090 / US5 AS4）。
//!
//! 带检查的解析 → 去掉检查后的裸指针路径（仅在已确认合法的下标上走）。
//! 越界硬闯见 `c20_mem_safety_ub` 与 `c15_unsafe_ub`。
//! MUST NOT 编写 eBPF 程序（FR-017）。
//!
//! 运行：`cargo run -p m5-unsafe --example c20_bounds_check`
//! PREDICT-UB: clean

use m5_unsafe::c20::{parse_u16_be_checked, parse_u16_be_unchecked};

fn main() {
    let pkt = [0x08u8, 0x00, 0x45, 0x00];
    println!("=== bounds-checked parse ===");
    println!("  off 0 = {:?}", parse_u16_be_checked(&pkt, 0));
    println!(
        "  off 3 (would be oob) = {:?}",
        parse_u16_be_checked(&pkt, 3)
    );

    println!("=== same offset, check then unchecked ===");
    let off = 2;
    let checked = parse_u16_be_checked(&pkt, off);
    let unchecked = checked.map(|_| {
        // SAFETY:
        // - 有效性：`parse_u16_be_checked` 刚对同一 `off` 返回 `Some`，故 `off + 2 <= len`。
        // - 对齐：按字节读，`u8` 对齐为 1。
        // - 别名：只读。
        // - provenance：来自 `pkt`，偏移仍在分配内。
        // - 生命周期：`pkt` 活过 `main`。
        unsafe { parse_u16_be_unchecked(&pkt, off) }
    });
    println!("  checked = {checked:?}");
    println!("  unchecked after check = {unchecked:?}");
    println!(
        "  removing the check and using off=3 is the UB path (see c20_mem_safety_ub / concept C-20)."
    );
}
