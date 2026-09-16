//! C-01 Ownership —— 非断言观察输出。
//!
//! 这里只**打印**现象，不做任何断言。稳定的结论全部在
//! `tests/c01_ownership.rs`（见 experiment-contract §C：断言与非断言产物物理分离）。
//!
//! 运行：`cargo run -p m1-ownership --example c01_ownership`

use m1_ownership::c01::{DropLog, Noisy, consume};

fn main() {
    println!("=== 1. 销毁时刻由静态作用域决定 ===");
    let log = DropLog::new();
    {
        let _inner = Noisy::new("inner", &log);
        println!("  进入内层作用域后，已销毁：{:?}", log.events());
    }
    println!("  离开内层作用域后，已销毁：{:?}", log.events());

    println!();
    println!("=== 2. 同一作用域内，销毁顺序是声明顺序的逆序 ===");
    let log = DropLog::new();
    {
        let _first = Noisy::new("first", &log);
        let _second = Noisy::new("second", &log);
        let _third = Noisy::new("third", &log);
    }
    println!("  声明顺序 first, second, third");
    println!("  销毁顺序 {:?}", log.events());

    println!();
    println!("=== 3. 所有权转移后，销毁发生在新所有者的作用域末尾 ===");
    let log = DropLog::new();
    let moved = Noisy::new("moved", &log);
    println!("  调用 consume 之前，已销毁：{:?}", log.events());
    consume(moved);
    println!("  consume 返回之后，已销毁：{:?}", log.events());
    println!("  —— 注意：销毁发生在 consume 内部，而非本函数末尾");

    println!();
    println!("=== 4. 嵌套所有权：容器销毁时递归销毁其元素 ===");
    let log = DropLog::new();
    {
        // clippy 建议改用数组。这里坚持用 Vec：本段观察的是**堆分配容器**的 drop glue
        // ——先逐元素销毁、再释放缓冲区。数组没有第二步，会弱化观察对象。
        #[allow(clippy::useless_vec)]
        let _bag = vec![
            Noisy::new("elem0", &log),
            Noisy::new("elem1", &log),
            Noisy::new("elem2", &log),
        ];
        println!("  Vec 尚在作用域内，已销毁：{:?}", log.events());
    }
    println!("  Vec 离开作用域后，已销毁：{:?}", log.events());
    println!("  —— 这段递归销毁的代码是编译器生成的 drop glue，源码里并不存在");
}
