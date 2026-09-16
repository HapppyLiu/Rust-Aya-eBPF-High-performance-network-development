//! C-03 Borrowing —— 非断言观察输出。
//!
//! 运行：`cargo run -p m1-ownership --example c03_borrow`

use m1_ownership::c03::{Counter, SharedCounter, first_half};

fn main() {
    println!("=== 1. 多个不可变借用可以共存 ===");
    let c = Counter::new(10);
    let r1 = &c;
    let r2 = &c;
    let r3 = &c;
    println!(
        "  三个 &Counter 同时存活：{} {} {}",
        r1.get(),
        r2.get(),
        r3.get()
    );

    println!();
    println!("=== 2. 可变借用是独占的，但 NLL 让它在最后一次使用后即结束 ===");
    let mut c = Counter::new(0);
    let m = &mut c;
    m.add(5);
    // m 的最后一次使用到此为止 —— 借用在这里结束，而非在作用域末尾
    println!("  可变借用结束后可以再取不可变借用：{}", c.get());

    println!();
    println!("=== 3. 借用不转移所有权 ===");
    let data = [1u8, 2, 3, 4, 5, 6];
    println!("  first_half(&data) = {:?}", first_half(&data));
    println!("  data 仍然可用：{:?}", data);

    println!();
    println!("=== 4. RefCell：同一条规则，检查挪到运行期 ===");
    let s = SharedCounter::new(0);
    s.add(3);
    s.add(4);
    println!("  通过 &self 修改内部值，当前值 = {}", s.get());
    println!("  —— 注意 add 的签名是 &self，可变性由 RefCell 在运行期把关");

    println!();
    println!("=== 5. 运行期借用冲突的表现形式 ===");
    let s = SharedCounter::new(0);
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        s.provoke_runtime_conflict()
    })) {
        Ok(()) => println!("  未 panic（不应发生）"),
        Err(_) => println!("  发生 panic —— 与编译期版本的错误码是同一条规则的两种后果"),
    }
    println!("  （完整 panic 消息属非断言产物，其措辞随版本变化）");
}
