//! # m4-concurrency —— C-12…C-14
//!
//! | C-ID | Capability | 实验 |
//! |------|-----------|------|
//! | C-12 | Send / Sync | `c12_send_sync` |
//! | C-13 | Concurrency | `c13_concurrency` |
//! | C-14 | Atomic | `c14_atomic` |
//!
//! 本 crate 的 `src/` 只放**被 example 与 test 复用**的最小类型与函数。
//! 现象观察在 `examples/`，稳定断言在 `tests/` —— 两者物理隔离（R-05）。
//!
//! `quiz_types` 是 T024 题集里 12 个自定义类型的可编译副本，供
//! `tests/c12_send_sync_quiz.rs` 做正向 `assert_send` / `assert_sync`。
//! 负向判定在 `compile_fail/quiz_*.rs`（独立编译，不依赖本 crate）。

pub mod c12;
pub mod c13;
pub mod c14;
pub mod quiz_types;
