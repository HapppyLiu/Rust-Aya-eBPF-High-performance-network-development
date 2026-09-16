//! C-12 Send / Sync —— 非断言观察输出。
//!
//! 运行：`cargo run -p m4-concurrency --example c12_send_sync`

use m4_concurrency::c12::{move_cell_to_thread, share_mutex_cell};

fn main() {
    println!("=== 1. 移动 Cell：Send 成立，原线程不再持有 ===");
    let saw = move_cell_to_thread(7);
    println!("  other thread saw {saw}");

    println!();
    println!("=== 2. 共享 Mutex<Cell>：锁把可移动升级成可共享 ===");
    let after = share_mutex_cell(10, 3);
    println!("  after scoped add = {after}");

    println!();
    println!("=== 3. 共享裸 Cell 被拒绝（见 compile_fail/c12_cell_across_threads.rs） ===");
    println!("  那是 Sync 一侧；本 example 不演示编译失败。");
}
