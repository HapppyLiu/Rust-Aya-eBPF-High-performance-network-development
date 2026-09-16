//! C-12 Send / Sync 题集 —— 编译器是最终裁判（learning-artifact-contract §F4）。
//!
//! 正向：`assert_send` / `assert_sync`。
//! 负向：`compile_fail/quiz_*.rs` 断言 E0277。
//! 题集定义见 `acceptance/send-sync-quiz.md`。

use m4_concurrency::c12::{assert_send, assert_sync};
use m4_concurrency::quiz_types::{
    Callback, Guarded, Held, Peek, RawCell, Shared, SharedMut, Slot, Tick, Ticker,
};

/// CLAIM: `Tick { count: u64 }` 两者皆是（字段全是，自动推导通过）。
#[test]
fn quiz_tick_is_send_sync() {
    assert_send::<Tick>();
    assert_sync::<Tick>();
}

/// CLAIM: `Slot { Cell<u32> }` 是 `Send`。`!Sync` 见 `quiz_slot_not_sync.rs`。
#[test]
fn quiz_slot_is_send() {
    assert_send::<Slot>();
}

/// CLAIM: `Guarded { Mutex<Cell<u32>> }` 两者皆是（`Mutex` 的 `Sync` 条件是 `T: Send`）。
#[test]
fn quiz_guarded_is_send_sync() {
    assert_send::<Guarded>();
    assert_sync::<Guarded>();
}

/// CLAIM: `Held { MutexGuard }` 是 `Sync`。`!Send` 见 `quiz_held_not_send.rs`。
#[test]
fn quiz_held_is_sync() {
    assert_sync::<Held<'static>>();
}

/// CLAIM: `Ticker { AtomicUsize }` 两者皆是。
#[test]
fn quiz_ticker_is_send_sync() {
    assert_send::<Ticker>();
    assert_sync::<Ticker>();
}

/// CLAIM: `Callback { Box<dyn Fn() + Send> }` 是 `Send`。`!Sync` 见 `quiz_callback_not_sync.rs`。
#[test]
fn quiz_callback_is_send() {
    assert_send::<Callback>();
}

/// CLAIM: `RawCell { UnsafeCell<u32> }` 是 `Send`。`!Sync` 见 `quiz_rawcell_not_sync.rs`。
#[test]
fn quiz_rawcell_is_send() {
    assert_send::<RawCell>();
}

/// CLAIM: 题集中应当不满足的标记全部以 E0277 被拒绝。
#[test]
fn quiz_negatives_are_e0277() {
    const CASES: &[(&str, &[&str])] = &[
        ("compile_fail/quiz_slot_not_sync.rs", &["E0277"]),
        ("compile_fail/quiz_shared_not_send.rs", &["E0277"]),
        ("compile_fail/quiz_rawslot_not_send.rs", &["E0277"]),
        ("compile_fail/quiz_sharedmut_not_send.rs", &["E0277"]),
        ("compile_fail/quiz_held_not_send.rs", &["E0277"]),
        ("compile_fail/quiz_marked_not_send.rs", &["E0277"]),
        ("compile_fail/quiz_callback_not_sync.rs", &["E0277"]),
        ("compile_fail/quiz_rawcell_not_sync.rs", &["E0277"]),
        ("compile_fail/quiz_peek_not_send.rs", &["E0277"]),
    ];
    for (path, codes) in CASES {
        rf_harness::compile_fail::expect_errors(path, codes);
    }
}

// 下面几个类型在正向侧没有任何成立的标记，只靠负向样本覆盖。
// 写出名字是为了让"题集 12 题都进了裁判"可被 grep 核对。
#[allow(dead_code)]
fn quiz_types_covered_only_negatively() {
    let _ = std::any::type_name::<Shared>();
    let _ = std::any::type_name::<SharedMut>();
    let _ = std::any::type_name::<Peek<'static>>();
}
