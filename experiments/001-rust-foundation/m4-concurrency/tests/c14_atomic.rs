//! C-14 Atomic —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c14.md`。
//! Relaxed 握手看见的值是 NON-ASSERTION，本文件不断言它。

use m4_concurrency::c12::{assert_send, assert_sync};
use m4_concurrency::c14::{release_acquire_seen, seqcst_sum};
use std::sync::atomic::{AtomicUsize, Ordering};

/// CLAIM: 4 个线程各 `SeqCst` 自增 25 次，终值是 100。
#[test]
fn seqcst_increments_sum_to_product() {
    assert_eq!(seqcst_sum(4, 25), 100);
}

/// CLAIM: Release 存储旗标 + Acquire 加载旗标之后，载荷必须是 42。
#[test]
fn release_acquire_handshake_sees_payload() {
    assert_eq!(release_acquire_seen(), 42);
}

/// CLAIM: `fetch_add` 返回**加之前**的值。
#[test]
fn fetch_add_returns_previous() {
    let n = AtomicUsize::new(7);
    assert_eq!(n.fetch_add(3, Ordering::SeqCst), 7);
    assert_eq!(n.load(Ordering::SeqCst), 10);
}

/// CLAIM: `Ordering` 的变体可区分；`Relaxed` 与 `SeqCst` 不是同一个值。
#[test]
fn ordering_variants_are_distinct() {
    assert_ne!(Ordering::Relaxed, Ordering::SeqCst);
    assert_ne!(Ordering::Release, Ordering::Acquire);
}

/// CLAIM: `AtomicUsize` 与 `usize` 同宽（包装不另占字）。
#[test]
fn atomic_usize_has_usize_width() {
    assert_eq!(
        size_of::<AtomicUsize>(),
        size_of::<usize>(),
        "AtomicUsize 与 usize 同宽"
    );
}

/// CLAIM: `AtomicUsize` 既是 `Send` 又是 `Sync`（显式 impl，不是因为没有内部可变性）。
#[test]
fn atomic_is_send_and_sync() {
    assert_send::<AtomicUsize>();
    assert_sync::<AtomicUsize>();
}
