//! EXPECT: E0499
//! CLAIM: 安全 Rust 里两个 scoped 线程不能同时持有同一局部变量的 `&mut`。
//! 编译器拒绝的是借用规则（同一时刻至多一个可变借用），不是"线程 API 忘了加锁"。
//! 这就是 US4 AS3：数据竞争在安全语言里被拦截的位置。绕过它需要 `unsafe` 并承担
//! 不制造数据竞争的义务 —— 本模块不绕。

use std::thread;

pub fn unsynchronized_increment() {
    let mut hits = 0u64;
    thread::scope(|scope| {
        scope.spawn(|| {
            hits += 1;
        });
        scope.spawn(|| {
            hits += 1;
        });
    });
}
