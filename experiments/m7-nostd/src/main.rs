//! US7 / C-22…C-24：`x86_64-unknown-none` 上的最小 `no_std` 产物。
//!
//! "可运行" = 构建成功（spec.md US7 Independent Test）。本二进制没有操作系统可返回，
//! 不能在本机执行。

#![cfg_attr(not(feature = "probe-std-crate"), no_std)]
#![cfg_attr(not(feature = "probe-std-crate"), no_main)]

#[cfg(all(
    not(feature = "probe-vec-no-alloc-crate"),
    not(feature = "probe-std-crate")
))]
extern crate alloc;

#[cfg(not(feature = "probe-std-crate"))]
mod bump;
#[cfg(not(feature = "probe-std-crate"))]
mod c22_nostd;
#[cfg(not(feature = "probe-std-crate"))]
mod c23_core_alloc_std;
#[cfg(not(feature = "probe-std-crate"))]
mod c24_panic_alloc;

/// 步骤 1 构造 A：关掉本 feature 时提供 panic 处理；打开则故意缺失。
#[cfg(all(
    not(feature = "probe-no-panic-handler"),
    not(feature = "probe-std-crate")
))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

/// 裸机入口。host 的 `main` 只在 `probe-std-crate` 探测路径上出现（且应当编不过）。
#[cfg(not(feature = "probe-std-crate"))]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let ptr_w = c22_nostd::touch();
    let summed = c23_core_alloc_std::touch();
    let n = c24_panic_alloc::touch();
    core::hint::black_box((ptr_w, summed, n));
    loop {
        core::hint::spin_loop();
    }
}

#[cfg(feature = "probe-std-crate")]
fn main() {
    // 步骤 2 构造 B：不再 `no_std`，强制去链接 std。裸机 target 上应当失败。
    let _ = std::env::args();
}
