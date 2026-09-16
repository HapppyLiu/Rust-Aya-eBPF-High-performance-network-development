//! C-11 Smart pointer —— 被 `c11_smart_ptr` 的 example 与 test 复用的最小设施。
//!
//! 三种合同：
//! - [`Box`]：独占，一个所有者；
//! - [`Rc`]：单线程共享，`clone` 加计数；
//! - [`Arc`]：跨线程共享，`clone` 同样只加计数。

use std::rc::Rc;
use std::sync::Arc;

/// 堆上独占。调用方拿到唯一的 `Box`。
#[must_use]
pub fn box_unique(n: u32) -> Box<u32> {
    Box::new(n)
}

/// 单线程共享一对。两个句柄指向同一份分配，强引用计数为 2。
#[must_use]
pub fn rc_pair(n: u32) -> (Rc<u32>, Rc<u32>) {
    let a = Rc::new(n);
    let b = Rc::clone(&a);
    (a, b)
}

/// 可跨线程共享一对。合同与 [`rc_pair`] 相同，只是 `T: Send + Sync` 时 `Arc` 本身也是。
#[must_use]
pub fn arc_pair(n: u32) -> (Arc<u32>, Arc<u32>) {
    let a = Arc::new(n);
    let b = Arc::clone(&a);
    (a, b)
}
