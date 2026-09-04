//! C-03 Borrowing —— 被 `c03_borrow` 的 example 与 test 复用的最小设施。
//!
//! 借用规则有两个执行位置，本模块各准备一个观察对象：
//!
//! - **编译期**（borrow checker）：[`Counter`] 的 `&self` / `&mut self` 方法，
//!   违规样本放在 `compile_fail/`，断言错误码；
//! - **运行期**（`RefCell` 的 `BorrowCounter`）：[`SharedCounter`]，违规时 panic。
//!
//! 同一条规则、两种执行时机 —— 这个对照是本能力的核心。

use std::cell::RefCell;

/// 编译期借用检查的观察对象。
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Counter {
    value: i64,
}

impl Counter {
    #[must_use]
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    /// 不可变借用：可以同时存在任意多个。
    #[must_use]
    pub fn get(&self) -> i64 {
        self.value
    }

    /// 可变借用：同一时刻只能存在一个，且不能与任何不可变借用共存。
    pub fn add(&mut self, delta: i64) {
        self.value += delta;
    }
}

/// 把借用检查推迟到运行期的版本。
///
/// `RefCell` 并没有**取消**借用规则，它只是把检查从编译期挪到了运行期：
/// 内部的 `BorrowCounter`（一个 `isize`）记录当前的借用状态，违规时 `panic` 而不是编译失败。
/// 代价是错误从"编译不过"变成了"线上崩溃"。
#[derive(Debug, Default)]
pub struct SharedCounter {
    value: RefCell<i64>,
}

impl SharedCounter {
    #[must_use]
    pub fn new(value: i64) -> Self {
        Self {
            value: RefCell::new(value),
        }
    }

    /// 注意签名是 `&self` —— 调用方只需不可变借用，却能改内部值。
    /// 这就是"内部可变性"，它把可变性的证明义务从编译器转移到了 `RefCell`。
    pub fn add(&self, delta: i64) {
        *self.value.borrow_mut() += delta;
    }

    #[must_use]
    pub fn get(&self) -> i64 {
        *self.value.borrow()
    }

    /// 故意在已持有可变借用的情况下再借一次，触发运行期 panic。
    ///
    /// 这个方法存在的唯一目的，是让"运行期借用冲突"成为可断言的事实
    /// （对照 `compile_fail/` 里同一条规则的编译期版本）。
    ///
    /// # Panics
    ///
    /// 总是 panic：`already borrowed` / `already mutably borrowed`。
    pub fn provoke_runtime_conflict(&self) {
        let _first = self.value.borrow_mut();
        let _second = self.value.borrow_mut(); // BorrowCounter 已为负，在此发现冲突
    }
}

/// 返回内部切片的不可变借用。返回值的生命周期绑定到入参 —— 这是省略规则的结果。
#[must_use]
pub fn first_half(data: &[u8]) -> &[u8] {
    &data[..data.len() / 2]
}
