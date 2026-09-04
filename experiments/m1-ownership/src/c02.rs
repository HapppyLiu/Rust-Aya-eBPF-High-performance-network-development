//! C-02 Move semantics —— 被 `c02_move` 的 example 与 test 复用的最小设施。
//!
//! 这里刻意准备了两种类型：
//!
//! - [`Meters`]：`Copy`，赋值是**复制**，原值仍可用；
//! - [`Tracked`]：非 `Copy` 且实现 `Drop`，赋值是**移动**，原值失效。
//!
//! 二者的差别不是语法糖，而是"谁负责销毁"这一责任的归属方式不同。

use std::cell::Cell;

/// 一个 `Copy` 类型。按位复制即语义正确，因此没有"移动"可言。
///
/// 注意它**不能**实现 `Drop`：`Copy` 与 `Drop` 互斥。
/// 原因正是责任归属 —— 若一个值可以被随意复制，就无法确定该由哪一份负责释放。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Meters(pub u32);

/// 一个非 `Copy`、可 `Clone`、带 `Drop` 的类型。
///
/// `drops` 是一个**借用**的计数器，销毁时自增。它让"这个值到底有没有被销毁"
/// 成为可断言的事实 —— 这是观察 [`std::mem::forget`] 效果的唯一办法。
pub struct Tracked<'c> {
    pub id: u32,
    drops: &'c Cell<usize>,
}

impl<'c> Tracked<'c> {
    #[must_use]
    pub fn new(id: u32, drops: &'c Cell<usize>) -> Self {
        Self { id, drops }
    }

    /// 手工实现 `Clone` 的语义：产生一个**独立**的、同样会计入销毁的值。
    #[must_use]
    pub fn duplicate(&self) -> Tracked<'c> {
        Tracked {
            id: self.id,
            drops: self.drops,
        }
    }
}

impl Drop for Tracked<'_> {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

/// 一个可 `Default` 的持有堆内存的类型，用于观察 [`std::mem::take`]。
///
/// `take` 需要 `Default`：它要放一个"合法但空"的值回原处，
/// 因为原处**不能**留下一个无效状态 —— 那正是 Rust 与 C 的分野。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Label {
    pub text: String,
}

impl Label {
    #[must_use]
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
        }
    }
}

/// 取得所有权（移动进来），返回其长度后销毁它。
pub fn consume_label(label: Label) -> usize {
    label.text.len()
}

/// 只借用，不取得所有权。调用后原值仍归调用方所有。
#[must_use]
pub fn borrow_label(label: &Label) -> usize {
    label.text.len()
}
