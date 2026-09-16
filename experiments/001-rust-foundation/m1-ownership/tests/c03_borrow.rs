//! C-03 Borrowing —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c03-borrow.md`。
//!
//! 本文件里有几个测试**靠"能编译"本身**来证明命题（例如 NLL）。
//! 这类断言的有效性来自：若命题为假，这个文件根本无法编译。

use m1_ownership::c03::{Counter, SharedCounter, first_half};

/// CLAIM: 任意多个不可变借用可以同时存活。
#[test]
fn many_shared_borrows_coexist() {
    let c = Counter::new(10);
    let r1 = &c;
    let r2 = &c;
    let r3 = &c;
    assert_eq!((r1.get(), r2.get(), r3.get()), (10, 10, 10));
}

/// CLAIM: NLL：可变借用在**最后一次使用**后即结束，而非在作用域末尾。
///
/// 若借用持续到作用域末尾，`c.get()` 这一行会与 `m` 冲突，本文件将无法编译。
/// 因此"编译通过"就是这条命题的证明。
#[test]
fn nll_ends_borrow_at_last_use() {
    let mut c = Counter::new(0);
    let m = &mut c;
    m.add(5);
    // m 的生命周期在上一行结束
    assert_eq!(c.get(), 5);
}

/// CLAIM: 借用不转移所有权：调用后原值仍可用。
#[test]
fn borrowing_does_not_move() {
    let data = [1u8, 2, 3, 4, 5, 6];
    assert_eq!(first_half(&data), &[1, 2, 3]);
    assert_eq!(data.len(), 6, "借用后原数组仍归调用方所有");
}

/// CLAIM: 内部可变性：通过 `&self` 修改内部值。
///
/// 可变性的证明义务从编译器转移给了 `RefCell`。
#[test]
fn refcell_allows_mutation_through_shared_reference() {
    let s = SharedCounter::new(0);
    let alias = &s; // 两个 &SharedCounter 同时存活

    s.add(3);
    alias.add(4);

    assert_eq!(s.get(), 7);
}

/// CLAIM: 运行期借用冲突的后果是 **panic**，而非编译失败。
///
/// 断言只针对"是否 panic"这一确定性事实；panic 的**措辞**属非断言产物。
#[test]
#[should_panic(expected = "already")]
fn refcell_double_borrow_panics_at_runtime() {
    let s = SharedCounter::new(0);
    s.provoke_runtime_conflict();
}

/// CLAIM: 同一条规则的两种执行时机对照。
///
/// - 编译期版本：`compile_fail/c03_two_mut_borrows.rs`（错误码 E0499）
/// - 运行期版本：上一个测试（panic）
///
/// 这里断言的是：**合法**的顺序访问在两个版本下都能通过，
/// 说明 `RefCell` 并未放宽规则，只是改变了检查时机。
#[test]
fn sequential_borrows_are_legal_in_both_regimes() {
    // 编译期版本
    let mut compile_time = Counter::new(0);
    {
        let m = &mut compile_time;
        m.add(1);
    }
    {
        let m = &mut compile_time;
        m.add(1);
    }
    assert_eq!(compile_time.get(), 2);

    // 运行期版本：两次 borrow_mut 不重叠，因此不 panic
    let run_time = SharedCounter::new(0);
    run_time.add(1);
    run_time.add(1);
    assert_eq!(run_time.get(), 2);
}

/// CLAIM: 编译期违规样本的错误码断言（US1 AS1：预测与实际诊断一致）。
#[test]
fn compile_time_violations_report_expected_codes() {
    rf_harness::compile_fail::expect_errors("compile_fail/c03_two_mut_borrows.rs", &["E0499"]);
    rf_harness::compile_fail::expect_errors("compile_fail/c03_mut_while_shared.rs", &["E0502"]);
}
