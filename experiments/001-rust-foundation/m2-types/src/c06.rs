//! C-06 Trait —— 被 `c06_trait` 的 example 与 test 复用的最小设施。
//!
//! - [`Score`]：同一 trait 的静态分发与动态分发对照；
//! - [`Label`]：`Display` + `Deref`，展示标准库 trait 如何把语法糖解析到具体实现。

use std::fmt;
use std::ops::Deref;

/// 对象安全的计分接口：方法不返回 `Self`，可以被做成 trait 对象。
pub trait Score {
    fn score(&self) -> u32;
    fn label(&self) -> &'static str;
}

pub struct Ping;

pub struct Http;

impl Score for Ping {
    fn score(&self) -> u32 {
        1
    }
    fn label(&self) -> &'static str {
        "ping"
    }
}

impl Score for Http {
    fn score(&self) -> u32 {
        80
    }
    fn label(&self) -> &'static str {
        "http"
    }
}

/// 静态分发：`S` 在调用点已知，编译器把 `score` 解析成具体函数地址。
#[must_use]
pub fn score_static<S: Score>(s: &S) -> u32 {
    s.score()
}

/// 动态分发：调用点只知道"实现了 `Score`"，具体地址存在 vtable 里。
#[must_use]
pub fn score_dynamic(s: &dyn Score) -> u32 {
    s.score()
}

/// 新类型包装。`Display` 让 `{}` 能打印它，`Deref` 让 `label.len()` 落到内部 `str`。
pub struct Label(pub String);

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for Label {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}
