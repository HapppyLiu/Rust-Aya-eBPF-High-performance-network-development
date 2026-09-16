//! C-10 Closure —— 非断言观察输出。
//!
//! 运行：`cargo run -p m3-composition --example c10_closure`

use m3_composition::c10::{call_as_fn, call_as_fn_mut, call_as_fn_once};

fn main() {
    println!("=== 1. 按共享引用捕获：可重复调用 ===");
    let s = "xdp";
    println!("  len twice: {} {}", call_as_fn(s), call_as_fn(s));

    println!();
    println!("=== 2. 按可变引用捕获：每次调用改计数 ===");
    let mut n = 0;
    let after = call_as_fn_mut(&mut n);
    println!("  after two calls: counter={n} last={after}");

    println!();
    println!("=== 3. 按值捕获：调用一次就消费 String ===");
    let owned = String::from("pkt");
    println!("  took = {:?}", call_as_fn_once(owned));
    // `owned` 此时已不可用 —— 这就是 FnOnce。

    println!();
    println!("=== 4. move + 拥有 String：可以进线程（对照 compile_fail 的借用版） ===");
    let payload = String::from("ok");
    let handle = std::thread::spawn(move || payload.len());
    println!("  spawned len = {}", handle.join().unwrap());
}
