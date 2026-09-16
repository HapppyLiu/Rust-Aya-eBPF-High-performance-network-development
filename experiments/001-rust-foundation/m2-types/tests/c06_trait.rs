//! C-06 Trait —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c06.md`。

use m2_types::c06::{Http, Label, Ping, Score, score_dynamic, score_static};
use std::mem::size_of;

/// CLAIM: 指向 trait 对象的引用是胖指针，宽度为两个 `usize`（数据指针 + vtable）。
#[test]
fn dyn_trait_reference_is_two_usizes() {
    assert_eq!(size_of::<&dyn Score>(), 2 * size_of::<usize>());
    assert_eq!(size_of::<&Ping>(), size_of::<usize>());
    assert!(
        size_of::<&dyn Score>() > size_of::<&Ping>(),
        "宽出来的那个 usize 是 vtable 指针"
    );
}

/// CLAIM: 同一输入走静态分发与动态分发，返回值相等 —— 差的是选定实现的时机，不是结果。
#[test]
fn static_and_dynamic_dispatch_agree_on_results() {
    let ping = Ping;
    let http = Http;
    assert_eq!(score_static(&ping), score_dynamic(&ping));
    assert_eq!(score_static(&http), score_dynamic(&http));
    assert_eq!(score_static(&ping), 1);
    assert_eq!(score_static(&http), 80);
}

/// CLAIM: `Display` 把 `{}` 解析到 `fmt`；`Deref` 把方法调用落到 `Target = str`。
#[test]
fn display_and_deref_resolve_to_the_impls() {
    let label = Label(String::from("xdp-pass"));
    assert_eq!(format!("{label}"), "xdp-pass");
    assert_eq!(label.len(), 8);
    assert!(label.starts_with("xdp"));
    assert_eq!(&*label, "xdp-pass");
}
