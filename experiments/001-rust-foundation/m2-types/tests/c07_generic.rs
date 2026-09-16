//! C-07 Generic —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c07.md`。

use m2_types::c07::{Wrapper, count_gt, max_cmp, max_cmp_dyn, max_of};
use std::mem::size_of;

/// CLAIM: 同一份泛型函数可以对多种具体类型求出正确结果（单态化的功能后果）。
#[test]
fn max_of_works_for_each_monomorphized_type() {
    assert_eq!(max_of(3u8, 9u8), 9u8);
    assert_eq!(max_of(3u16, 9u16), 9u16);
    assert_eq!(max_of(3u32, 9u32), 9u32);
}

/// CLAIM: 单态化改变布局：`Wrapper<T>` 的大小等于 `T`，随 `T` 变化。
#[test]
fn monomorphization_produces_type_dependent_layout() {
    assert_eq!(size_of::<Wrapper<u8>>(), size_of::<u8>());
    assert_eq!(size_of::<Wrapper<u64>>(), size_of::<u64>());
    assert_ne!(size_of::<Wrapper<u8>>(), size_of::<Wrapper<u64>>());
}

/// CLAIM: `Iterator` 的 `Item` 是关联类型；`PartialOrd` bound 约束的是元素，不是迭代器本身。
#[test]
fn iterator_bound_filters_by_partial_ord_on_item() {
    assert_eq!(count_gt([1, 5, 3, 9], 4), 2);
    assert_eq!(count_gt(std::iter::empty::<i32>(), 0), 0);
}

/// CLAIM: 同一套比较逻辑，泛型版本与 trait 对象版本行为等价。
#[test]
fn generic_and_dyn_comparison_are_behaviorally_equivalent() {
    let a = 4i32;
    let b = 11i32;
    assert_eq!(max_cmp(a, b), max_cmp_dyn(&a, &b));
    assert_eq!(max_cmp(b, a), max_cmp_dyn(&b, &a));
    assert_eq!(max_cmp(a, a), max_cmp_dyn(&a, &a));

    let c = 7i16;
    let d = 2i16;
    assert_eq!(max_cmp(c, d), max_cmp_dyn(&c, &d));
}

/// CLAIM: 调用要求 `T: PartialOrd` 的函数时，调用方缺少该 bound → E0277。
///
/// 直接对无 bound 的 `T` 写 `>` 会先报 E0369。本样本走"调用带 bound 的函数"
/// 以对准"缺 bound"这条规则，而不是"运算符找不到"。
#[test]
fn missing_partial_ord_bound_is_e0277() {
    rf_harness::compile_fail::expect_errors("compile_fail/c07_missing_bound.rs", &["E0277"]);
}
