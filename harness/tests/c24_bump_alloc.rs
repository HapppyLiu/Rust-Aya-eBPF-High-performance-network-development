//! C-24 host 侧：直接调用裸机 crate 的 bump 分配器（T117）。
//!
//! 裸机产物无法跑 Miri。本文件把 `experiments/m7-nostd/src/bump.rs` 引进 host 测试，
//! 让 `cargo +nightly miri test -p rf-harness --test c24_bump_alloc` 取得 `ub_verdict`。
//!
//! 不把 `BumpAlloc` 注册为本测试 crate 的 `#[global_allocator]`：测试框架自己会分配，
//! 64KiB 静态 arena 撑不住 harness；也避免与 `harness_selfcheck` 的 CountingAllocator 混淆。
//! 学习对象仍在 m7，这里只是验证设施入口（harness-api §非目标）。

#[path = "../../experiments/m7-nostd/src/bump.rs"]
mod bump;

use bump::{ARENA_SIZE, BumpAlloc};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

/// CLAIM: 8 对齐的 16 字节请求得到非空指针，地址满足对齐，写入后读回同一值。
#[test]
fn bump_alloc_aligned_roundtrip() {
    let a = BumpAlloc::new();
    let layout = Layout::from_size_align(16, 8).unwrap();
    // SAFETY: `layout` 非零且对齐合法，arena 为空，满足 `GlobalAlloc::alloc` 的调用方义务。
    let p = unsafe { a.alloc(layout) };
    assert!(!p.is_null(), "16 字节 bump 不应耗尽空 arena");
    assert_eq!(
        (p as usize) % 8,
        0,
        "返回地址必须满足 layout.align()（关系性质，不断言绝对值）"
    );
    // SAFETY: `p` 来自上面的成功分配，16 字节已对齐，写 `u64` 不越界。
    unsafe { ptr::write(p.cast::<u64>(), 0x1111_2222_3333_4444) };
    // SAFETY: 第二个 `u64` 落在同一块 16 字节分配的后半，仍在区间内。
    let p_hi = unsafe { p.add(8) };
    // SAFETY: `p_hi` 由上一行从同一分配算出，目标 `u64` 仍在 16 字节区间内。
    unsafe { ptr::write(p_hi.cast::<u64>(), 0xaaaa_bbbb_cccc_dddd) };
    // SAFETY: 刚写入的低位 `u64` 已初始化。
    let lo = unsafe { ptr::read(p.cast::<u64>()) };
    // SAFETY: 刚写入的高位 `u64` 已初始化。
    let hi = unsafe { ptr::read(p_hi.cast::<u64>()) };
    assert_eq!(lo, 0x1111_2222_3333_4444);
    assert_eq!(hi, 0xaaaa_bbbb_cccc_dddd);
    // SAFETY: `p`/`layout` 与分配时一致（bump 的 dealloc 是空操作，仍须按契约调用）。
    unsafe { a.dealloc(p, layout) };
}

/// CLAIM: 两次成功分配返回不同指针（区间不重叠）；这是 bump CAS 提交的可观察后果。
#[test]
fn two_allocs_are_disjoint() {
    let a = BumpAlloc::new();
    let layout = Layout::from_size_align(8, 8).unwrap();
    // SAFETY: 空 arena，非零 layout。
    let p1 = unsafe { a.alloc(layout) };
    // SAFETY: 第一次已提交 CAS，第二次切下一段不相交区间。
    let p2 = unsafe { a.alloc(layout) };
    assert!(!p1.is_null() && !p2.is_null());
    assert_ne!(p1, p2, "两次 bump 不能交出同一块区间");
    // SAFETY: `p1` 来自本分配器、layout 一致。
    unsafe { a.dealloc(p1, layout) };
    // SAFETY: `p2` 来自本分配器、layout 一致。
    unsafe { a.dealloc(p2, layout) };
}

/// CLAIM: 请求超过 arena 时返回空指针，而不是 panic 或越出静态缓冲。
#[test]
fn exhausted_arena_returns_null() {
    let a = BumpAlloc::new();
    let layout = Layout::from_size_align(ARENA_SIZE, 1).unwrap();
    // SAFETY: size 等于 arena，第一次应成功。
    let first = unsafe { a.alloc(layout) };
    assert!(!first.is_null(), "空 arena 应能切出一整块");
    let one = Layout::from_size_align(1, 1).unwrap();
    // SAFETY: arena 已满，实现必须返回 null，不得对 buf 做越界 add。
    let second = unsafe { a.alloc(one) };
    assert!(second.is_null(), "耗尽后必须返回 null 走 OOM 路径");
    // SAFETY: `first`/`layout` 与分配时一致。
    unsafe { a.dealloc(first, layout) };
}
