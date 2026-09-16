//! # m6-ffi —— C-21 FFI
//!
//! | C-ID | Capability | 实验 |
//! |------|-----------|------|
//! | C-21 | FFI | `c21_ffi_layout` / `c21_ffi` / `c21_ffi_ownership` / `c21_errno` |
//!
//! 本 crate 是 Feature 内**唯一**允许 `libc` + `cc` 的地方（R-08）。
//! UB 判定用 ASan，不用 Miri（真实 C 调用，R-02）。
//! 现象观察在 `examples/`，稳定断言在 `tests/`（R-05）。

pub mod c21;
