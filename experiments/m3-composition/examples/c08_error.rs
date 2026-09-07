//! C-08 Error handling —— 非断言观察输出。
//!
//! 运行：`cargo run -p m3-composition --example c08_error`

use m3_composition::c08::{parse_port, parse_port_manual};

fn show(label: &str, s: &str) {
    println!("  {label:?}");
    println!("    manual = {:?}", parse_port_manual(s));
    println!("    ?      = {:?}", parse_port(s));
}

fn main() {
    println!("=== 同一组输入：手写 match vs ? ===");
    show("合法端口", "443");
    show("空串", "");
    show("非数字", "x");
    show("本层拒绝（0）", "0");
    show("本层拒绝（太大）", "8080");
}
