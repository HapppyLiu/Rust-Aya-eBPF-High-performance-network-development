//! C-22：`#![no_std]` 的编译期断言。
//!
//! 裸机产物跑不了 `#[test]`。按 T003 / CHK041，合法断言载体是：
//! - 本文件的 `const` 断言与 `compile_error!`
//! - `build.rs` 对 `TARGET` 的退出码
//! - `tools/check-nostd-artifact.sh` 的符号 / ELF 检查

/// CLAIM: 本 crate 只在 `target_os = "none"` 上通过类型检查。
/// 在 host OS 上编会在这里 `compile_error!`（与 `build.rs` 双保险）。
#[cfg(not(target_os = "none"))]
compile_error!("C-22: m7-nostd 必须针对 x86_64-unknown-none 构建（见 .cargo/config.toml / R-03）");

/// CLAIM: 指针宽度在本 target 上是 8。这是 x86_64 的事实，不是"所有 no_std 都是 8"。
pub const PTR_WIDTH: usize = core::mem::size_of::<*const u8>();
const _: () = assert!(PTR_WIDTH == 8);

/// CLAIM: 本翻译单元看得到 `core` 的 `Option`，不需要 `std`。
pub const fn core_option_is_available() -> Option<u8> {
    Some(1)
}

const _: () = assert!(matches!(core_option_is_available(), Some(1)));

/// 给 `_start` 一个不会被优化掉的挂钩。
#[inline(never)]
pub fn touch() -> usize {
    PTR_WIDTH
}
