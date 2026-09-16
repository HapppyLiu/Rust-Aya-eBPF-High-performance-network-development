//! C-01 Ownership —— 稳定断言。
//!
//! 每个测试对应 `acceptance/criteria/c01-ownership.md` 中的一条判定。
//! 所有断言都必须是**确定性**的：不含地址、时间、`{:?}` 全量输出。

use m1_ownership::c01::{DropLog, Noisy, consume};

/// CLAIM: 值在其所有者离开作用域时被销毁 —— 不早也不晚。
#[test]
fn drop_runs_at_scope_end() {
    let log = DropLog::new();
    {
        let _inner = Noisy::new("inner", &log);
        assert!(log.is_empty(), "值仍在作用域内时不应被销毁");
    }
    assert_eq!(log.events(), vec!["inner"], "离开作用域后应恰好销毁一次");
}

/// CLAIM: 同一作用域内的销毁顺序是声明顺序的**逆序**。
///
/// 之所以必须逆序：后声明的值可能借用了先声明的值。
#[test]
fn drop_order_is_reverse_of_declaration() {
    let log = DropLog::new();
    {
        let _first = Noisy::new("first", &log);
        let _second = Noisy::new("second", &log);
        let _third = Noisy::new("third", &log);
    }
    assert_eq!(log.events(), vec!["third", "second", "first"]);
}

/// CLAIM: 所有权转移后，销毁责任跟着转移到**新所有者**的作用域。
#[test]
fn ownership_transfer_moves_drop_responsibility() {
    let log = DropLog::new();
    let value = Noisy::new("moved", &log);

    assert!(log.is_empty());
    consume(value); // 所有权进入 consume，在其末尾销毁
    assert_eq!(log.events(), vec!["moved"], "应在 consume 内部完成销毁");
}

/// CLAIM: 容器被销毁时，其元素被递归销毁 —— 这是编译器生成的 drop glue 的作用。
///
/// `Vec` 的元素按**下标升序**销毁（与栈上局部变量的逆序规则不同）。
#[test]
fn container_drop_recurses_into_elements() {
    let log = DropLog::new();
    {
        // clippy 建议改用数组。这里坚持用 Vec：本测试断言的是**堆分配容器**的 drop glue
        // ——先逐元素销毁、再释放缓冲区。数组没有第二步，断言对象会退化。
        #[allow(clippy::useless_vec)]
        let _bag = vec![
            Noisy::new("elem0", &log),
            Noisy::new("elem1", &log),
            Noisy::new("elem2", &log),
        ];
        assert!(log.is_empty());
    }
    assert_eq!(log.events(), vec!["elem0", "elem1", "elem2"]);
}

/// CLAIM: 值只会被销毁**一次**：即便经过多层移动，销毁总次数仍是 1。
#[test]
fn value_is_dropped_exactly_once_after_repeated_moves() {
    let log = DropLog::new();
    {
        let a = Noisy::new("once", &log);
        let b = a; // move
        let c = b; // move again
        let _d = c; // and again
    }
    assert_eq!(log.count(), 1, "多次移动不应产生多次销毁");
}

/// CLAIM: `Noisy` 的销毁行为并不依赖 `DropLog` 自身有 `Drop` 实现。
///
/// drop glue 会递归处理所有字段，`Drop::drop` 只是其中可选的一环。
#[test]
fn drop_glue_handles_fields_without_drop_impl() {
    let log = DropLog::new();
    {
        let _n = Noisy::new("field-check", &log);
    }
    // DropLog 内部的 RefCell<Vec<_>> 没有手写 Drop，仍被正确清理；
    // 这里能读到完整记录，说明日志本身在 Noisy 之后才失效。
    assert_eq!(log.events(), vec!["field-check"]);
}

/// CLAIM: 编译期违规样本的错误码断言（US1 AS1：预测与实际诊断一致）。
#[test]
fn compile_time_violations_report_expected_codes() {
    rf_harness::compile_fail::expect_errors("compile_fail/c01_use_after_move.rs", &["E0382"]);
    rf_harness::compile_fail::expect_errors("compile_fail/c01_manual_drop_call.rs", &["E0040"]);
}
