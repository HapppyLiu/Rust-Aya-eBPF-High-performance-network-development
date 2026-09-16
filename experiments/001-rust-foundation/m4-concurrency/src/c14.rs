//! C-14 Atomic —— 被 `c14_atomic` 的 example 与 test 复用的最小设施。
//!
//! 原子操作先保证单次读写不撕裂。内存序是另一问：周围那些别的读写何时可见。

use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

/// `threads` 个线程各 `SeqCst` 自增 `each` 次。终值确定，与交错顺序无关。
#[must_use]
pub fn seqcst_sum(threads: usize, each: usize) -> usize {
    let acc = AtomicUsize::new(0);
    thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                for _ in 0..each {
                    acc.fetch_add(1, Ordering::SeqCst);
                }
            });
        }
    });
    acc.load(Ordering::SeqCst)
}

/// 释放 / 获取握手：写侧先放载荷再立旗标，读侧看见旗标后再读载荷。
/// 该配对下，看见旗标就蕴含看见载荷。
#[must_use]
pub fn release_acquire_seen() -> usize {
    let data = AtomicUsize::new(0);
    let flag = AtomicUsize::new(0);
    let seen = AtomicUsize::new(0);
    thread::scope(|scope| {
        scope.spawn(|| {
            data.store(42, Ordering::Relaxed);
            flag.store(1, Ordering::Release);
        });
        scope.spawn(|| {
            while flag.load(Ordering::Acquire) == 0 {
                std::hint::spin_loop();
            }
            seen.store(data.load(Ordering::Relaxed), Ordering::Relaxed);
        });
    });
    seen.load(Ordering::SeqCst)
}

/// 两边都用 `Relaxed` 的对照。返回读侧看见的载荷。
///
/// 语言规则允许"旗标已立、载荷仍是 0"。本函数**不**把返回值当成稳定断言：
/// x86_64 的 TSO 常常把它掩盖成 42。观察留给 example / OBSERVATIONS。
#[must_use]
pub fn relaxed_handshake_seen() -> usize {
    let data = AtomicUsize::new(0);
    let flag = AtomicUsize::new(0);
    let seen = AtomicUsize::new(0);
    thread::scope(|scope| {
        scope.spawn(|| {
            data.store(42, Ordering::Relaxed);
            flag.store(1, Ordering::Relaxed);
        });
        scope.spawn(|| {
            while flag.load(Ordering::Relaxed) == 0 {
                std::hint::spin_loop();
            }
            seen.store(data.load(Ordering::Relaxed), Ordering::Relaxed);
        });
    });
    seen.load(Ordering::Relaxed)
}
