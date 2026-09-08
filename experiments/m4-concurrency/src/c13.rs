//! C-13 Concurrency —— 被 `c13_concurrency` 的 example 与 test 复用的最小设施。
//!
//! 互斥锁保证同一时刻最多一个线程碰到被保护的数据。
//! 它**不**保证线程完成的先后顺序 —— 顺序是 NON-ASSERTION。

use std::sync::Mutex;
use std::thread;

/// `threads` 个 scoped 线程经互斥锁各加 `each`，返回总和。
/// 总和与完成顺序无关，因此可以断言。
#[must_use]
pub fn scoped_mutex_sum(threads: usize, each: u64) -> u64 {
    let acc = Mutex::new(0u64);
    thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                let mut guard = acc.lock().unwrap();
                *guard += each;
            });
        }
    });
    acc.into_inner().unwrap()
}

/// 把每个线程的编号推进日志。返回值的**集合**确定，**排列**不确定。
#[must_use]
pub fn scoped_push_ids(n: u32) -> Vec<u32> {
    let log = Mutex::new(Vec::new());
    thread::scope(|scope| {
        let log = &log;
        for id in 0..n {
            scope.spawn(move || {
                log.lock().unwrap().push(id);
            });
        }
    });
    log.into_inner().unwrap()
}
