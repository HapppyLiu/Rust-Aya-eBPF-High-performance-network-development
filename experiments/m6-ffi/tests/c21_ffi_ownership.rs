//! C-21 跨边界所有权 —— C malloc，Rust free（CHK044）。

use m6_ffi::c21::{OWNED_MSG, take_c_message};

/// CLAIM: C 分配的字符串内容等于约定字面量，且本路径只释放一次（约定：Rust 负责 free）。
/// 故意双释放 / 泄漏不作为稳定断言运行；故障形态写在 concept.md 与 OBSERVATIONS。
#[test]
fn c_alloc_rust_free_roundtrips_literal() {
    let got = take_c_message();
    assert_eq!(got, OWNED_MSG);
}

/// CLAIM: 连续两次按约定接管，每次都能读到同一字面量（每次都是新的 malloc/free 对）。
#[test]
fn two_independent_c_allocs_each_freed_by_rust() {
    assert_eq!(take_c_message(), OWNED_MSG);
    assert_eq!(take_c_message(), OWNED_MSG);
}
