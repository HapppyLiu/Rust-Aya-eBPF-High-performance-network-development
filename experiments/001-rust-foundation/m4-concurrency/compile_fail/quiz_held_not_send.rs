//! EXPECT: E0277
//! CLAIM: `MutexGuard` 显式 `!Send`（解锁必须发生在加锁的同一线程），
//! 故 `Held: !Send`。`Sync` 仍可成立 —— 两条标记独立。

use std::sync::MutexGuard;

pub struct Held<'a> {
    guard: MutexGuard<'a, u32>,
}

fn assert_send<T: Send>() {}

pub fn check() {
    assert_send::<Held<'static>>();
}
