//! C-10 Closure —— 被 `c10_closure` 的 example 与 test 复用的最小设施。
//!
//! 三种捕获对应三个调用 trait 的 `self` 形式：
//! `&self`（[`Fn`]）/ `&mut self`（[`FnMut`]）/ 按值 `self`（[`FnOnce`]）。

/// 按共享引用捕获。可以重复调用、不修改捕获物 → 满足 `Fn`（因而也满足后两层）。
pub fn call_as_fn(s: &str) -> usize {
    let f = || s.len();
    apply_fn(f)
}

/// 按可变引用捕获。每次调用改计数 → 满足 `FnMut`，不满足 `Fn`。
pub fn call_as_fn_mut(counter: &mut i32) -> i32 {
    let mut f = || {
        *counter += 1;
        *counter
    };
    apply_fn_mut(&mut f);
    apply_fn_mut(&mut f)
}

/// 按值拿走 `String`。调用一次就消费捕获物 → 只保证 `FnOnce`。
pub fn call_as_fn_once(s: String) -> String {
    let f = || s;
    apply_fn_once(f)
}

pub fn apply_fn<F: Fn() -> usize>(f: F) -> usize {
    f()
}

pub fn apply_fn_mut<F: FnMut() -> i32>(f: &mut F) -> i32 {
    f()
}

pub fn apply_fn_once<F: FnOnce() -> String>(f: F) -> String {
    f()
}
