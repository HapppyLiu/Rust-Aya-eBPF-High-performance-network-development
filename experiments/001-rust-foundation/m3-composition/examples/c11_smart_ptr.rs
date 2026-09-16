//! C-11 Smart pointer —— 非断言观察输出。
//!
//! 运行：`cargo run -p m3-composition --example c11_smart_ptr`

use m3_composition::c11::{arc_pair, box_unique, rc_pair};
use std::rc::Rc;
use std::sync::Arc;

fn main() {
    println!("=== 1. Box：独占，解引用得到那个值 ===");
    let b = box_unique(7);
    println!("  box = {b}");

    println!();
    println!("=== 2. Rc：clone 加计数，不复制堆上的值 ===");
    let (a, c) = rc_pair(7);
    println!("  strong after clone = {}", Rc::strong_count(&a));
    println!("  both see {} {}", *a, *c);
    drop(c);
    println!("  strong after drop  = {}", Rc::strong_count(&a));

    println!();
    println!("=== 3. Arc：合同相同，但可以进线程 ===");
    let (x, y) = arc_pair(7);
    println!("  strong = {}", Arc::strong_count(&x));
    let h = std::thread::spawn(move || *y);
    println!("  other thread saw {}", h.join().unwrap());
    println!("  strong after join = {}", Arc::strong_count(&x));
}
