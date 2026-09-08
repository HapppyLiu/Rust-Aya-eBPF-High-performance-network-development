//! # m5-unsafe —— C-15…C-20
//!
//! | C-ID | Capability | 安全侧 | UB 对照 |
//! |------|-----------|--------|---------|
//! | C-15 | Unsafe Rust | `c15_unsafe` | `c15_unsafe_ub` |
//! | C-16 | Raw pointer | `c16_raw_ptr` | `c16_raw_ptr_ub` |
//! | C-17 | Pointer arithmetic | `c17_ptr_arith` | `c17_ptr_arith_ub` |
//! | C-18 | Alignment | `c18_alignment` | `c18_alignment_ub` |
//! | C-19 | Aliasing | `c19_aliasing` | `c19_aliasing_ub` |
//! | C-20 | Memory safety | `c20_mem_safety` | `c20_mem_safety_ub` |
//!
//! 本 crate 的 `src/` 只放**安全侧**被 example 与 test 复用的最小函数。
//! 故意 UB 只存在于 `examples/*_ub.rs`，避免 `cargo +nightly miri test` 直接踩雷。
//! 现象观察在 `examples/`，稳定断言在 `tests/` —— 两者物理隔离（R-05）。

pub mod c15;
pub mod c16;
pub mod c17;
pub mod c18;
pub mod c19;
pub mod c20;
