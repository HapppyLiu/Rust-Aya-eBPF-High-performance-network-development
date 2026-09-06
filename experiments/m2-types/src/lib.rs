//! # m2-types —— C-05…C-07
//!
//! | C-ID | Capability | 实验 |
//! |------|-----------|------|
//! | C-05 | Struct / Enum | `c05_layout` |
//! | C-06 | Trait | `c06_trait` |
//! | C-07 | Generic | `c07_generic` |
//!
//! 本 crate 的 `src/` 只放**被 example 与 test 复用**的最小类型与函数。
//! 现象观察在 `examples/`，稳定断言在 `tests/` —— 两者物理隔离（R-05）。

pub mod c05;
pub mod c06;
pub mod c07;
