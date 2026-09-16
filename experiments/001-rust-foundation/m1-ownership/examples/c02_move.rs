//! C-02 Move semantics —— 非断言观察输出。
//!
//! 运行：`cargo run -p m1-ownership --example c02_move`

use m1_ownership::c02::{Label, Meters, Tracked, borrow_label, consume_label};
use std::cell::Cell;
use std::mem;

fn main() {
    println!("=== 1. Copy 类型：赋值后原值仍可用 ===");
    let a = Meters(42);
    let b = a; // 复制，不是移动
    println!("  a = {a:?}, b = {b:?}  —— 两个独立的值");

    println!();
    println!("=== 2. 非 Copy 类型：移动后销毁责任转移，总销毁次数不变 ===");
    let drops = Cell::new(0);
    {
        let first = Tracked::new(1, &drops);
        let _second = first; // move
        println!("  移动完成，作用域尚未结束，销毁次数 = {}", drops.get());
    }
    println!("  离开作用域，销毁次数 = {}", drops.get());

    println!();
    println!("=== 3. Clone 产生独立的值，销毁次数随之增加 ===");
    let drops = Cell::new(0);
    {
        let original = Tracked::new(2, &drops);
        let _copy = original.duplicate();
    }
    println!("  一次 duplicate 之后，销毁次数 = {}", drops.get());

    println!();
    println!("=== 4. mem::forget 抑制销毁 ===");
    let drops = Cell::new(0);
    {
        let t = Tracked::new(3, &drops);
        mem::forget(t);
    }
    println!("  forget 之后销毁次数 = {}", drops.get());
    println!("  —— forget 是**安全**函数：泄漏不违反内存安全");

    println!();
    println!("=== 5. replace / take：搬走值的同时不留下无效状态 ===");
    let mut label = Label::new("original");
    let old = mem::replace(&mut label, Label::new("replacement"));
    println!("  replace 取出 {:?}，原处现在是 {:?}", old.text, label.text);

    let mut label = Label::new("to-be-taken");
    let taken = mem::take(&mut label);
    println!("  take 取出 {:?}，原处现在是 {:?}", taken.text, label.text);
    println!("  —— take 要求 T: Default，因为它必须自己找一个替补值放回去");

    println!();
    println!("=== 6. 传参：移动 vs 借用 ===");
    let label = Label::new("hello world");
    println!(
        "  borrow_label 返回 {}，之后仍可用：{:?}",
        borrow_label(&label),
        label.text
    );
    println!(
        "  consume_label 返回 {}，之后 label 不可再用",
        consume_label(label)
    );
}
