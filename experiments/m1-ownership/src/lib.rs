//! # m1-ownership —— C-01…C-04
//!
//! | C-ID | Capability | 实验 |
//! |------|-----------|------|
//! | C-01 | Ownership | `c01_ownership` |
//! | C-02 | Move semantics | `c02_move` |
//! | C-03 | Borrowing | `c03_borrow` |
//! | C-04 | Lifetime | `c04_lifetime` |
//!
//! 本 crate 的 `src/` 只放**被 example 与 test 复用**的最小类型与函数。
//! 现象观察在 `examples/`，稳定断言在 `tests/` —— 两者物理隔离（R-05）。
//!
//! 每个能力一个子模块，使四项能力的实验互不冲突（可并行推进）。

pub mod c01;
pub mod c02;
pub mod c03;
pub mod c04;
