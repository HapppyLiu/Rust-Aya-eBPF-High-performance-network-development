//! # m3-composition —— C-08…C-11
//!
//! | C-ID | Capability | 实验 |
//! |------|-----------|------|
//! | C-08 | Error handling | `c08_error` |
//! | C-09 | Iterator | `c09_iterator` |
//! | C-10 | Closure | `c10_closure` |
//! | C-11 | Smart pointer | `c11_smart_ptr` |
//!
//! 本 crate 的 `src/` 只放**被 example 与 test 复用**的最小类型与函数。
//! 现象观察在 `examples/`，稳定断言在 `tests/` —— 两者物理隔离（R-05）。
//!
//! `CountingAllocator` 只在 `tests/` 侧启用（T049）。它对**整个测试二进制**
//! 全局生效，但计数按线程隔离；使用 `measure` 的测试另用 [`MEASURE_LOCK`]
//! 串行化（T056 / harness-api §并发约束）。

use std::sync::Mutex;

/// 串行化所有 `measure` 调用，避免同一测试二进制里多个 `#[test]` 在同一线程
/// 被 rustc 复用时区间重叠。计数器本身已经是 thread-local；这把锁是额外的
/// crate 内互斥（T056）。
pub static MEASURE_LOCK: Mutex<()> = Mutex::new(());

pub mod c08;
pub mod c09;
pub mod c10;
pub mod c11;
