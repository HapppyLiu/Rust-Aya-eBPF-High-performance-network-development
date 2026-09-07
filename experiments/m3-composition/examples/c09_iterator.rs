//! C-09 Iterator —— 非断言观察输出。
//!
//! 运行：`cargo run -p m3-composition --example c09_iterator`

use m3_composition::c09::{
    double_collect_iter, double_collect_loop, double_sum_iter, double_sum_loop,
};

fn main() {
    println!("=== 1. 惰性：副作用出现在消费时，且按元素走完整条链 ===");
    let iter = (0..4)
        .map(|x| {
            println!("    map {x}");
            x * 2
        })
        .filter(|x| {
            println!("    filter {x}");
            *x > 2
        });
    println!("  链已构造，尚未消费");
    println!("  开始 for_each：");
    iter.for_each(|x| println!("    for_each {x}"));

    println!();
    println!("=== 2. 同一逻辑：手写循环 vs 迭代器，元素结果 ===");
    println!(
        "  sum  loop={} iter={}",
        double_sum_loop(4),
        double_sum_iter(4)
    );
    println!(
        "  collect loop={:?} iter={:?}",
        double_collect_loop(4),
        double_collect_iter(4)
    );
}
