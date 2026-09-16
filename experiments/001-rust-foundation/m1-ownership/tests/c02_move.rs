//! C-02 Move semantics —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c02-move.md`。

use m1_ownership::c02::{Label, Meters, Tracked, borrow_label, consume_label};
use std::cell::Cell;
use std::mem;

/// CLAIM: `Copy` 类型赋值后原值仍可用，且两份完全相等。
#[test]
fn copy_types_remain_usable_after_assignment() {
    let a = Meters(42);
    let b = a;
    assert_eq!(a, Meters(42), "Copy 类型的原值不应失效");
    assert_eq!(b, Meters(42));
}

/// CLAIM: 移动不产生额外的销毁：链式移动之后销毁总次数仍是 1。
///
/// 这是"移动只是簿记、不是复制"最直接的证据。
#[test]
fn move_does_not_duplicate_drops() {
    let drops = Cell::new(0);
    {
        let first = Tracked::new(1, &drops);
        let second = first;
        let _third = second;
        assert_eq!(drops.get(), 0, "移动过程中不应发生销毁");
    }
    assert_eq!(drops.get(), 1, "链式移动后应恰好销毁一次");
}

/// CLAIM: 与移动相对：`Clone` 产生**独立**的值，因此销毁两次。
#[test]
fn clone_creates_an_independent_value() {
    let drops = Cell::new(0);
    {
        let original = Tracked::new(2, &drops);
        let _duplicate = original.duplicate();
    }
    assert_eq!(drops.get(), 2, "clone 出的值有独立的销毁责任");
}

/// CLAIM: `mem::forget` 抑制销毁，且它是**安全**函数。
#[test]
fn forget_suppresses_drop() {
    let drops = Cell::new(0);
    {
        let t = Tracked::new(3, &drops);
        mem::forget(t); // 无 unsafe：泄漏不违反内存安全
    }
    assert_eq!(drops.get(), 0, "被 forget 的值不应被销毁");
}

/// CLAIM: `mem::replace` 取出旧值并放入调用方提供的新值，原处始终持有合法值。
#[test]
fn replace_swaps_in_a_caller_supplied_value() {
    let mut label = Label::new("original");
    let old = mem::replace(&mut label, Label::new("replacement"));

    assert_eq!(old, Label::new("original"));
    assert_eq!(label, Label::new("replacement"));
}

/// CLAIM: `mem::take` 取出旧值并放入 `Default::default()` —— 这正是它要求 `T: Default` 的原因。
#[test]
fn take_leaves_the_default_value_behind() {
    let mut label = Label::new("to-be-taken");
    let taken = mem::take(&mut label);

    assert_eq!(taken, Label::new("to-be-taken"));
    assert_eq!(
        label,
        Label::default(),
        "原处应留下 Default 值，而非无效状态"
    );
    assert!(label.text.is_empty());
}

/// CLAIM: `replace` 与 `take` 都不会让原位置出现无效状态：紧接着读它是合法的。
#[test]
fn neither_replace_nor_take_leaves_an_invalid_state() {
    let mut a = Label::new("a");
    let _ = mem::replace(&mut a, Label::new("a2"));
    assert_eq!(a.text.len(), 2);

    let mut b = Label::new("b");
    let _ = mem::take(&mut b);
    assert_eq!(b.text.len(), 0);
}

/// CLAIM: 借用不转移所有权；移动转移。
#[test]
fn borrowing_preserves_ownership_while_moving_transfers_it() {
    let label = Label::new("hello world");

    assert_eq!(borrow_label(&label), 11);
    assert_eq!(label.text, "hello world", "借用后原值仍归调用方所有");

    assert_eq!(consume_label(label), 11);
    // 此后 label 不可用 —— 该事实由 compile_fail/c01_use_after_move.rs 断言
}

/// CLAIM: `Meters` 是 `Copy`，`Label` 不是。用一个只接受 `Copy` 的泛型函数把这个差别变成事实。
#[test]
fn copy_bound_distinguishes_the_two_type_families() {
    fn duplicate_via_copy<T: Copy>(v: T) -> (T, T) {
        (v, v)
    }

    let (x, y) = duplicate_via_copy(Meters(7));
    assert_eq!(x, y);
    // duplicate_via_copy(Label::new("x")) 不通过编译：Label 不是 Copy。
}

/// CLAIM: 编译期违规样本的错误码断言（US1 AS1：预测与实际诊断一致）。
#[test]
fn compile_time_violations_report_expected_codes() {
    rf_harness::compile_fail::expect_errors("compile_fail/c02_copy_with_drop.rs", &["E0184"]);
    rf_harness::compile_fail::expect_errors("compile_fail/c02_move_out_of_borrow.rs", &["E0507"]);
}
