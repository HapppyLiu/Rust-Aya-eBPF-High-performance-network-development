//! C-09 Iterator —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c09.md`。
//!
//! `#[global_allocator]` 对本测试二进制全局生效（T049 / harness-api）。
//! 使用 `measure` 的断言集中在一个 `#[test]`，并用 crate 内互斥量串行化（T056）。

use m3_composition::MEASURE_LOCK;
use m3_composition::c09::{
    double_collect_iter, double_collect_loop, double_filter_lazy, double_sum_iter, double_sum_loop,
};
use rf_harness::counting_alloc::{CountingAllocator, measure};

#[global_allocator]
static A: CountingAllocator = CountingAllocator::new();

const N: i32 = 8;

/// CLAIM: 手写循环与迭代器链对同一输入求出相同的和、相同的元素序列。
#[test]
fn loop_and_iterator_agree_on_values() {
    assert_eq!(double_sum_loop(N), double_sum_iter(N));
    assert_eq!(double_collect_loop(N), double_collect_iter(N));
    assert_eq!(double_sum_iter(N), (0..N).map(|x| x * 2).sum::<i32>());
}

/// CLAIM: 分配次数由"要不要构造堆上容器"以及"事先知不知道容量"决定，
/// 不是由"写了 map"决定。本测试集中全部 `measure` 断言（T056）。
#[test]
fn iterator_chain_allocation_counts() {
    let _guard = MEASURE_LOCK.lock().expect("MEASURE_LOCK");

    let (sum, sum_stats) = measure(|| double_sum_iter(N));
    assert_eq!(sum, double_sum_loop(N));
    assert_eq!(
        sum_stats.allocs, 0,
        "sum 一条整数链不应当堆分配，实际 allocs={}",
        sum_stats.allocs
    );

    let (collected, collect_stats) = measure(|| double_collect_iter(N));
    assert_eq!(collected, double_collect_loop(N));
    assert_eq!(
        collect_stats.allocs, 1,
        "Range+map 的 size_hint 精确，collect 一次按长度分配，实际 allocs={}",
        collect_stats.allocs
    );
    assert_eq!(collect_stats.reallocs, 0, "精确容量不应再 realloc");

    let (_, push_stats) = measure(|| double_collect_loop(N));
    assert_ne!(
        push_stats.allocs + push_stats.reallocs,
        collect_stats.allocs,
        "Vec::new+push 从容量 0 增长，总分配事件应多于一次到位的 collect"
    );
    assert!(
        push_stats.allocs >= 1,
        "从空 Vec push {N} 个元素至少发生一次 alloc"
    );

    let (_, lazy_stats) = measure(|| {
        let _iter = double_filter_lazy(N);
    });
    assert_eq!(
        lazy_stats.allocs, 0,
        "只构造适配器、不消费，不应当堆分配，实际 allocs={}",
        lazy_stats.allocs
    );
}
