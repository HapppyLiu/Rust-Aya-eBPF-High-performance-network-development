//! C-01 Ownership —— 被 `c01_ownership` 的 example 与 test 复用的最小设施。
//!
//! 核心手法：让"值被销毁"这件**本来不可见**的事变得可观察。
//! [`Noisy`] 在自己的 `Drop` 里往 [`DropLog`] 写一条记录，
//! 于是"谁在什么时候被销毁"就变成了一个可以断言的序列。

use std::cell::RefCell;

/// 记录销毁事件的日志。
///
/// 用 `RefCell` 而非 `&mut`：多个 [`Noisy`] 需要**同时**持有对同一日志的访问权，
/// 而它们的 `drop` 只拿得到 `&self`。这把"共享 + 可变"的需求推到了运行期检查
/// （对照 C-03 的编译期借用检查）。
#[derive(Default)]
pub struct DropLog {
    events: RefCell<Vec<&'static str>>,
}

impl DropLog {
    #[must_use]
    pub fn new() -> Self {
        Self {
            events: RefCell::new(Vec::new()),
        }
    }

    /// 追加一条销毁记录。
    pub fn record(&self, name: &'static str) {
        self.events.borrow_mut().push(name);
    }

    /// 目前为止的销毁顺序快照。
    #[must_use]
    pub fn events(&self) -> Vec<&'static str> {
        self.events.borrow().clone()
    }

    /// 已发生的销毁次数。
    #[must_use]
    pub fn count(&self) -> usize {
        self.events.borrow().len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }
}

/// 一个在销毁时留下痕迹的值。
///
/// `'log` 表达的约束是：日志 MUST 活得比记录它的值更久。
/// 这不是装饰 —— 若反过来，`drop` 里访问的就是已销毁的日志。
/// 编译器据此拒绝把 `Noisy` 声明在 `DropLog` 之前的写法。
pub struct Noisy<'log> {
    name: &'static str,
    log: &'log DropLog,
}

impl<'log> Noisy<'log> {
    #[must_use]
    pub fn new(name: &'static str, log: &'log DropLog) -> Self {
        Self { name, log }
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl Drop for Noisy<'_> {
    fn drop(&mut self) {
        self.log.record(self.name);
    }
}

/// 取得所有权后立即让它离开作用域。
///
/// 用途：证明"销毁发生在**新所有者**的作用域末尾"，而不是在调用点。
/// 这是所有权转移最容易被忽略的可观察后果。
pub fn consume<T>(_value: T) {}
