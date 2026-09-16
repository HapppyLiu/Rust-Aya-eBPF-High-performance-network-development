//! C-11 Smart pointer —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c11.md`。
//!
//! `#[global_allocator]` 对本测试二进制全局生效（T049）。
//! 使用 `measure` 的断言集中在一个 `#[test]`，并用 crate 内互斥量串行化（T056）。

use m3_composition::MEASURE_LOCK;
use m3_composition::c11::{arc_pair, box_unique, rc_pair};
use rf_harness::counting_alloc::{CountingAllocator, measure};
use std::rc::Rc;
use std::sync::Arc;

#[global_allocator]
static A: CountingAllocator = CountingAllocator::new();

/// CLAIM: `Rc::clone` 增加强引用计数，不复制堆上的值；`drop` 一个句柄则计数减一。
#[test]
fn rc_clone_shares_and_bumps_strong_count() {
    let (a, b) = rc_pair(7);
    assert_eq!(*a, 7);
    assert_eq!(*b, 7);
    assert_eq!(Rc::strong_count(&a), 2);
    drop(b);
    assert_eq!(Rc::strong_count(&a), 1);
    assert_eq!(*a, 7);
}

/// CLAIM: `Arc::clone` 同样只加计数；`u32` 满足 `Send + Sync`，因此 `Arc<u32>` 可跨线程。
#[test]
fn arc_clone_shares_and_is_send() {
    let (a, b) = arc_pair(7);
    assert_eq!(Arc::strong_count(&a), 2);
    let h = std::thread::spawn(move || *b);
    assert_eq!(h.join().unwrap(), 7);
    assert_eq!(Arc::strong_count(&a), 1);
}

/// CLAIM: `Box` 是独占所有权，解引用得到被装箱的值。
#[test]
fn box_is_unique_owner() {
    let b = box_unique(7);
    assert_eq!(*b, 7);
}

/// CLAIM: 新建 `Box`/`Rc`/`Arc` 各一次堆分配；随后的 `Rc::clone`/`Arc::clone` 不再分配。
/// 本测试集中全部 `measure` 断言（T056）。
#[test]
fn smart_pointer_allocation_counts() {
    let _guard = MEASURE_LOCK.lock().expect("MEASURE_LOCK");

    let (_, box_stats) = measure(|| {
        let _b = box_unique(7);
    });
    assert_eq!(
        box_stats.allocs, 1,
        "Box::new 一次堆分配，实际 {}",
        box_stats.allocs
    );

    let (_, rc_stats) = measure(|| {
        let _pair = rc_pair(7);
    });
    assert_eq!(
        rc_stats.allocs, 1,
        "Rc::new 一次分配，Rc::clone 只加计数，实际 allocs={}",
        rc_stats.allocs
    );

    let (_, arc_stats) = measure(|| {
        let _pair = arc_pair(7);
    });
    assert_eq!(
        arc_stats.allocs, 1,
        "Arc::new 一次分配，Arc::clone 只加计数，实际 allocs={}",
        arc_stats.allocs
    );
}

/// CLAIM: `Rc<u32>` 即使 `move` 进 `thread::spawn` 也被拒绝（E0277，`Rc: !Send`）。
#[test]
fn rc_across_threads_is_e0277() {
    rf_harness::compile_fail::expect_errors("compile_fail/c11_rc_across_threads.rs", &["E0277"]);
}
