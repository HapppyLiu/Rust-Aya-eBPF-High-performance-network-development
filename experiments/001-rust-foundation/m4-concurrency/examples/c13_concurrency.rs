//! C-13 Concurrency —— 非断言观察输出。
//!
//! 运行：`cargo run -p m4-concurrency --example c13_concurrency`
//!
//! 线程完成顺序是 NON-ASSERTION，禁止拿一次打印当稳定事实。

use m4_concurrency::c13::{scoped_mutex_sum, scoped_push_ids};

fn main() {
    println!("=== 1. scope + Mutex：总和确定，完成顺序不确定 ===");
    let sum = scoped_mutex_sum(4, 25);
    println!("  sum = {sum}");

    println!();
    println!("=== 2. 各线程推入自己的编号（排列是 NON-ASSERTION） ===");
    let ids = scoped_push_ids(4);
    println!("  order (NON-ASSERTION) = {ids:?}");
    let set_sum: u32 = ids.iter().sum();
    println!("  id-set sum = {set_sum}");
}
