//! 并发 / 原子分层（C-12…C-14）。
//!
//! 库是 `no_std`：没有 `std::thread`（那是 OS services，C-13 在库内必须换实现）。
//! 计数走 [`AtomicUsize`]，测试 crate 再起线程来验证终值确定。
//!
//! ## 为何自动 `Send` + `Sync`（C-12）——禁止手写 `unsafe impl`
//!
//! `ParseStats` 的字段只有 `AtomicUsize`。
//! - **`Send`**：原子整数可以**移动**到另一线程。移动后原线程不再持有该位置，
//!   不存在"两个线程各拿一份非同步的可变别名"。
//! - **`Sync`**：`&AtomicUsize` 可以同时存在于多线程，因为所有写都走原子指令，
//!   不经过 `&mut T`。`AtomicUsize: Sync` 由标准库提供，本结构体因此自动 `Sync`。
//!
//! 自己写 `unsafe impl Send/Sync` 会跳过编译器对字段的检查，本层明确不这么做。
//! 正向验收用测试里的 `assert_send` / `assert_sync`（编译器为最终裁判）。

use core::sync::atomic::{AtomicUsize, Ordering};

/// 解析计数。所有修改走原子，因此 `&ParseStats` 可跨线程共享。
#[derive(Debug)]
pub struct ParseStats {
    frames: AtomicUsize,
    errors: AtomicUsize,
}

impl ParseStats {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            frames: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
        }
    }

    pub fn record_ok(&self) {
        self.frames.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_err(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    #[must_use]
    pub fn frames(&self) -> usize {
        self.frames.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn errors(&self) -> usize {
        self.errors.load(Ordering::Relaxed)
    }
}

impl Default for ParseStats {
    fn default() -> Self {
        Self::new()
    }
}
