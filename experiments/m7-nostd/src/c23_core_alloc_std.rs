//! C-23：同一段求和逻辑的三层版本。
//!
//! - `sum_core`：只碰切片与整数，任何 `no_std` 都能编。
//! - `sum_alloc`：把输入 `push` 进 `Vec` 再求和 —— 需要 `extern crate alloc` **和** 全局分配器。
//! - `sum_std`：只用 `probe-std-type` 编译，引入 std 专属类型，在裸机 target 上必须失败。

/// CLAIM: 只使用 `core` 的求和在 `x86_64-unknown-none` 上可编译。
#[must_use]
pub fn sum_core(xs: &[i32]) -> i32 {
    let mut n = 0i32;
    let mut i = 0;
    while i < xs.len() {
        n = n.saturating_add(xs[i]);
        i += 1;
    }
    n
}

/// CLAIM: 同一段逻辑加上 `Vec` 之后，依赖的是 alloc 层 + 本 crate 的分配器，不是 std。
#[cfg(not(feature = "probe-vec-no-alloc-crate"))]
#[must_use]
#[allow(clippy::vec_init_then_push)] // 与 core 版对照：同一段求和改走 Vec::push。
pub fn sum_alloc(xs: &[i32]) -> i32 {
    let mut v = alloc::vec::Vec::new();
    let mut i = 0;
    while i < xs.len() {
        v.push(xs[i]);
        i += 1;
    }
    sum_core(&v)
}

/// 步骤 2 构造 A：文件系统类型不在 core 里（OS services）。
/// 走 `core::fs` 路径，避免被 "找不到 crate std" 抢走这条错误。
#[cfg(feature = "probe-std-type")]
pub fn sum_std(_xs: &[i32]) -> i32 {
    let _f: core::fs::File;
    0
}

#[inline(never)]
pub fn touch() -> i32 {
    let xs = [1i32, 2, 3];
    let core_sum = sum_core(&xs);
    #[cfg(not(feature = "probe-vec-no-alloc-crate"))]
    {
        core_sum.saturating_add(sum_alloc(&xs))
    }
    #[cfg(feature = "probe-vec-no-alloc-crate")]
    {
        core_sum
    }
}
