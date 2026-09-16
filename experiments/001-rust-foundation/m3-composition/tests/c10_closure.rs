//! C-10 Closure —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c10.md`。

use m3_composition::c10::{
    apply_fn, apply_fn_mut, apply_fn_once, call_as_fn, call_as_fn_mut, call_as_fn_once,
};

/// CLAIM: 按共享引用捕获满足 `Fn`，因而可以重复调用且不修改捕获物。
#[test]
fn shared_ref_capture_is_fn() {
    assert_eq!(call_as_fn("xdp"), 3);
    let s = "pkt";
    let f = || s.len();
    assert_eq!(apply_fn(f), 3);
    assert_eq!(apply_fn(f), 3);
}

/// CLAIM: 按可变引用捕获满足 `FnMut`：两次调用各加一。
#[test]
fn mut_ref_capture_is_fn_mut() {
    let mut n = 10;
    let last = call_as_fn_mut(&mut n);
    assert_eq!(n, 12);
    assert_eq!(last, 12);

    let mut c = 0;
    let mut f = || {
        c += 1;
        c
    };
    assert_eq!(apply_fn_mut(&mut f), 1);
    assert_eq!(apply_fn_mut(&mut f), 2);
}

/// CLAIM: 按值捕获 `String` 满足 `FnOnce`：调用一次即消费捕获物。
#[test]
fn by_value_capture_is_fn_once() {
    assert_eq!(call_as_fn_once(String::from("ok")), "ok");
    let s = String::from("move");
    let f = || s;
    assert_eq!(apply_fn_once(f), "move");
}

/// CLAIM: 按引用捕获局部 `String` 再交给 `thread::spawn`（不 `move`）→ E0373。
/// 捕获方式决定闭包能否跨线程（US3 AS2 → US4 衔接）。
#[test]
fn borrow_escaping_to_thread_is_e0373() {
    rf_harness::compile_fail::expect_errors(
        "compile_fail/c10_borrow_escapes_thread.rs",
        &["E0373"],
    );
}
