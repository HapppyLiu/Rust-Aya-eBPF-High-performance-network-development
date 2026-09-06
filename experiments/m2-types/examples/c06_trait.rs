//! C-06 Trait —— 非断言观察输出。
//!
//! 运行：`cargo run -p m2-types --example c06_trait`
//!
//! 导出 IR：`tools/emit-llvm-ir.sh m2-types --example c06_trait`
//! 在 IR 里对照 `score_static`（直接调用）与 `score_dynamic`（经 vtable）。

use m2_types::c06::{Http, Label, Ping, Score, score_dynamic, score_static};
use std::mem::size_of;

fn main() {
    println!("=== 1. 同一 trait，两条分发路径 ===");
    let ping = Ping;
    let http = Http;
    println!(
        "  static  ping={} http={}",
        score_static(&ping),
        score_static(&http)
    );
    println!(
        "  dynamic ping={} http={}",
        score_dynamic(&ping),
        score_dynamic(&http)
    );

    println!();
    println!("=== 2. 引用宽度：具体类型 vs trait 对象 ===");
    println!(
        "  size(&Ping)={}  size(&dyn Score)={}  size(usize)={}",
        size_of::<&Ping>(),
        size_of::<&dyn Score>(),
        size_of::<usize>()
    );
    println!("  —— 多出来的那个 usize 是 vtable 指针，不是数据本身");

    println!();
    println!("=== 3. Display / Deref：语法糖落到哪份实现 ===");
    let label = Label(String::from("xdp-pass"));
    println!("  Display => {label}");
    println!(
        "  Deref   => len={} starts_with(xdp)? {}",
        label.len(),
        label.starts_with("xdp")
    );
}
