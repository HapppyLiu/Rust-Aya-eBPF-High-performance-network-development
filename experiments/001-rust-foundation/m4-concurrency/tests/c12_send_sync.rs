//! C-12 Send / Sync —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c12.md`。

use m4_concurrency::c12::{assert_send, assert_sync, move_cell_to_thread, share_mutex_cell};
use std::cell::Cell;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

/// CLAIM: `Send` 与 `Sync` 分开约束：`u64` 两者皆是（基线）。
#[test]
fn primitive_is_send_and_sync() {
    assert_send::<u64>();
    assert_sync::<u64>();
}

/// CLAIM: `Cell<u32>` 是 `Send`（可移动）但不是 `Sync`（不可共享引用）。
#[test]
fn cell_is_send_not_asserted_sync() {
    assert_send::<Cell<u32>>();
    // Sync 的负向在 compile_fail/c12_cell_across_threads.rs。
}

/// CLAIM: `Mutex<Cell<u32>>` 两者皆是——锁的 `Sync` 条件是 `T: Send`，不是 `T: Sync`。
#[test]
fn mutex_of_cell_is_send_and_sync() {
    assert_send::<Mutex<Cell<u32>>>();
    assert_sync::<Mutex<Cell<u32>>>();
}

/// CLAIM: 原子类型两者皆是；同样含内部可变性，但修改走原子指令。
#[test]
fn atomic_usize_is_send_and_sync() {
    assert_send::<AtomicUsize>();
    assert_sync::<AtomicUsize>();
}

/// CLAIM: 把 `Cell` 移动到另一线程之后读到的是原来的值（移动，不是共享）。
#[test]
fn moving_cell_preserves_value() {
    assert_eq!(move_cell_to_thread(7), 7);
}

/// CLAIM: 经 `Mutex` 共享 `Cell` 后，加法发生一次，终值确定。
#[test]
fn sharing_mutex_cell_applies_add() {
    assert_eq!(share_mutex_cell(10, 3), 13);
}

/// CLAIM: 两个 scoped 线程共享同一个 `Cell` 被拒绝，错误码为 E0277（`Cell: !Sync`）。
#[test]
fn cell_across_threads_is_e0277() {
    rf_harness::compile_fail::expect_errors("compile_fail/c12_cell_across_threads.rs", &["E0277"]);
}
