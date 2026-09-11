//! C-24：最小 panic 处理 + 把 bump 注册为全局分配器，使 `Vec` 可链接。
//!
//! 分配能力由 **本 crate 的 `BumpAlloc`** 提供，不是 `alloc` crate 自带堆。
//! 该前提在 eBPF 受限环境中**不成立**：那里没有可供 bump 的通用可写 arena，
//! 也没有与 `GlobalAlloc` 对等的内核分配入口（见 concept.md / OBSERVATIONS）。

#[cfg(not(feature = "probe-vec-no-global-alloc"))]
use crate::bump::BumpAlloc;

/// 缺省路径：提供全局分配器。
/// `probe-vec-no-global-alloc` 打开时故意不注册，以暴露步骤 3 构造 B。
#[cfg(not(feature = "probe-vec-no-global-alloc"))]
#[global_allocator]
static ALLOC: BumpAlloc = BumpAlloc::new();

/// CLAIM: 在已注册 `#[global_allocator]` 的前提下，`alloc::vec::Vec` 可以链接进裸机产物。
#[cfg(all(
    not(feature = "probe-vec-no-alloc-crate"),
    not(feature = "probe-vec-no-global-alloc")
))]
#[must_use]
#[allow(clippy::vec_init_then_push)] // 教学点是 Vec::new + push 走 GlobalAlloc，不是 vec! 语法。
pub fn vec_push_demo() -> usize {
    let mut v = alloc::vec::Vec::new();
    v.push(7u8);
    v.push(8u8);
    let n = v.len();
    core::mem::forget(v);
    n
}

#[cfg(any(
    feature = "probe-vec-no-alloc-crate",
    feature = "probe-vec-no-global-alloc"
))]
#[must_use]
pub fn vec_push_demo() -> usize {
    // 探测路径：仍然提到 `Vec`，以便缺 alloc crate 或缺全局分配器时失败。
    #[cfg(feature = "probe-vec-no-alloc-crate")]
    {
        let _v: alloc::vec::Vec<u8> = alloc::vec::Vec::new();
    }
    #[cfg(all(
        feature = "probe-vec-no-global-alloc",
        not(feature = "probe-vec-no-alloc-crate")
    ))]
    {
        let mut v = alloc::vec::Vec::new();
        v.push(1u8);
        core::mem::forget(v);
    }
    0
}

#[inline(never)]
pub fn touch() -> usize {
    vec_push_demo()
}
