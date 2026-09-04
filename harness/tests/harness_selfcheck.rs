//! `rf-harness` 自检（T018）。
//!
//! 验证**设施本身**的行为。全部 24 项能力的验收都建立在这个 crate 之上，
//! 设施不可靠则一切验收失效 —— 所以这个文件先于任何学习模块存在。
//!
//! 注意区分：本文件测的是"断言器会不会说谎"，不是任何 Rust 语言知识点。
//! 学习内容一律不放这里（harness-api.md §非目标）。

use rf_harness::counting_alloc::{self, CountingAllocator};

// 计数需要全局分配器。本 crate 的测试因此全程被计数 —— 这正是要在
// OBSERVATIONS 中说明"CountingAllocator 对整个 crate 全局生效"的原因。
#[global_allocator]
static ALLOC: CountingAllocator = CountingAllocator::new();

// ---------------------------------------------------------------- compile_fail

/// CLAIM: `expect_errors` 对**故意报错**的样本能成功匹配错误码。
#[test]
fn expect_errors_matches_a_failing_sample() {
    rf_harness::compile_fail::expect_errors("compile_fail/selfcheck_e0382.rs", &["E0382"]);
}

/// CLAIM: `expect_errors` 的断言语义是**子集**——只声明实际错误码的一部分也应通过。
///
/// 依据 experiment-contract §C4.4：rustc 可能追加派生诊断，追加项不构成失败。
#[test]
fn expect_errors_uses_subset_semantics() {
    let outcome = rf_harness::compile_fail::try_compile("compile_fail/selfcheck_e0382.rs");
    assert!(!outcome.success, "样本本应编译失败");
    assert!(outcome.has_code("E0382"));
    // 只断言实际码集合的一个子集：即使 rustc 追加了别的码，这里也应通过。
    rf_harness::compile_fail::expect_errors("compile_fail/selfcheck_e0382.rs", &["E0382"]);
}

/// CLAIM: `expect_errors` 对**可以编译**的样本会 panic ——
/// 这是防止"样本其实没触发规则却被记为通过"的那道闸。
#[test]
#[should_panic(expected = "竟然编译**成功**了")]
fn expect_errors_panics_on_compiling_sample() {
    rf_harness::compile_fail::expect_errors("compile_fail/selfcheck_ok.rs", &["E0382"]);
}

/// CLAIM: 期望的错误码未出现时 `expect_errors` 会 panic，且失败信息同时给出期望与实际。
#[test]
#[should_panic(expected = "错误码不匹配")]
fn expect_errors_panics_on_wrong_code() {
    // 样本实际触发 E0382；这里故意预测一个别的码。
    rf_harness::compile_fail::expect_errors("compile_fail/selfcheck_e0382.rs", &["E0499"]);
}

/// CLAIM: `codes()` 只认 `error[EXXXX]` 形式，不把正文里偶然出现的错误码字样计入。
#[test]
fn codes_are_extracted_from_error_headers_only() {
    let outcome = rf_harness::compile_fail::try_compile("compile_fail/selfcheck_e0382.rs");
    let codes = outcome.codes();
    assert!(codes.contains(&"E0382".to_owned()), "实际提取到：{codes:?}");
    // 去重：同一个码出现多次也只记一次。
    let mut sorted = codes.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        codes.len(),
        "codes() 应已去重，实际：{codes:?}"
    );
}

/// CLAIM: 空的 expected_codes 被拒绝 —— 只断言"编译失败"而不断言"因何失败"，
/// 会让学习者用错误的理由通过验收（R-06 拒绝 doctest compile_fail 的同一理由）。
#[test]
#[should_panic(expected = "expected_codes 不能为空")]
fn empty_expected_codes_is_rejected() {
    rf_harness::compile_fail::expect_errors("compile_fail/selfcheck_e0382.rs", &[]);
}

// -------------------------------------------------------------- counting_alloc

/// CLAIM: `measure` 对**已知分配次数**的闭包返回确定值。
///
/// 一次 `Box::new` = 恰好一次 alloc；离开作用域 = 恰好一次 dealloc。
/// 这个断言若不成立，US3 全部的"预测分配次数"验收都不可信。
///
/// 计数器按线程隔离，所以这些断言**不需要**与其他测试串行执行。
/// 该性质本身由 `measure_is_immune_to_other_threads` 单独把关。
#[test]
fn measure_counts_known_allocations() {
    // 单次 Box 分配。
    let (value, stats) = counting_alloc::measure(|| {
        let b = Box::new(42_u64);
        *b
    });
    assert_eq!(value, 42);
    assert_eq!(
        stats.allocs, 1,
        "一次 Box::new 应恰好一次 alloc，实际 {stats:?}"
    );
    assert_eq!(
        stats.deallocs, 1,
        "Box 离开作用域应恰好一次 dealloc，实际 {stats:?}"
    );
    assert_eq!(
        stats.bytes_allocated,
        size_of::<u64>() as u64,
        "字节数应为**请求值**，不含分配器内部开销，实际 {stats:?}"
    );

    // 不分配的闭包：三个计数全为 0。这条排除"计数器把测试框架自身的分配算进来"。
    let ((), stats) = counting_alloc::measure(|| {});
    assert_eq!(
        (stats.allocs, stats.deallocs, stats.reallocs),
        (0, 0, 0),
        "实际 {stats:?}"
    );
    assert_eq!(stats.bytes_allocated, 0);

    // 已知容量的 Vec：一次分配，无重分配。
    let (len, stats) = counting_alloc::measure(|| {
        let mut v: Vec<u8> = Vec::with_capacity(16);
        v.extend_from_slice(&[1, 2, 3, 4]);
        v.len()
    });
    assert_eq!(len, 4);
    assert_eq!(
        stats.allocs, 1,
        "with_capacity 后不应再分配，实际 {stats:?}"
    );
    assert_eq!(stats.reallocs, 0, "容量充足时不应 realloc，实际 {stats:?}");

    // 峰值：同时活着的两个 Box 的峰值 > 单个的大小。
    let ((), stats) = counting_alloc::measure(|| {
        let a = Box::new([0_u8; 64]);
        let b = Box::new([0_u8; 64]);
        assert_eq!(a.len() + b.len(), 128);
    });
    assert_eq!(
        stats.peak_bytes, 128,
        "两个 64 字节分配同时存活，峰值应为 128，实际 {stats:?}"
    );
}

/// CLAIM: 别的线程再怎么分配，也不会污染本线程的 `measure` 结果。
///
/// # 这条测试为什么存在
///
/// 计数器最初是进程全局的，于是 `measure_counts_known_allocations`
/// 在 `cargo test --workspace` 下约每 5 次失败 1 次：并行跑的其他测试
/// 把它们的分配算进了统计。那种量不配写进 `#[test]`（FR-003）。
///
/// 改成 thread-local 之后本测试用一个**持续分配的背景线程**把该场景固定下来 ——
/// 若哪天有人把计数器改回全局，这里会稳定失败，而不是变成偶发抖动。
#[test]
fn measure_is_immune_to_other_threads() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    /// 测量窗口内要求背景线程完成的分配轮数。
    ///
    /// 取一个远大于 1 的数是关键：若计数器退回全局，`allocs` 会变成约 1 + 2×该值，
    /// 断言以巨大的差距**稳定**失败，而不是偶尔差一两次。
    /// 仅靠"起一个背景线程"是不够的 —— 测量窗口只有几微秒，两者大概率不重叠。
    const OVERLAP_ROUNDS: u64 = 2_000;

    let stop = Arc::new(AtomicBool::new(false));
    let rounds = Arc::new(AtomicU64::new(0));

    let noise = {
        let (stop, rounds) = (Arc::clone(&stop), Arc::clone(&rounds));
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                // 每轮两次堆分配。若计数器是进程全局的，它们会被算进主线程的统计。
                let v: Vec<u8> = Vec::with_capacity(1024);
                std::hint::black_box(v.capacity());
                let b = Box::new([7_u8; 256]);
                std::hint::black_box(b[0]);
                rounds.fetch_add(1, Ordering::Relaxed);
            }
        })
    };

    // 等背景线程真正开始分配，否则测量窗口可能整个落在它启动之前。
    while rounds.load(Ordering::Relaxed) == 0 {
        std::hint::spin_loop();
    }

    // 窗口内做 1 次已知分配，并等背景线程再跑 OVERLAP_ROUNDS 轮 ——
    // 这把"偶发竞争"变成了"必然重叠"。
    let start = rounds.load(Ordering::Relaxed);
    let (value, stats) = counting_alloc::measure(|| {
        let b = Box::new(42_u64);
        while rounds.load(Ordering::Relaxed) < start + OVERLAP_ROUNDS {
            std::hint::spin_loop();
        }
        *b
    });

    stop.store(true, Ordering::Relaxed);
    noise.join().expect("背景线程不应 panic");

    assert_eq!(value, 42);
    assert_eq!(
        (stats.allocs, stats.deallocs),
        (1, 1),
        "测量窗口内背景线程至少跑了 {OVERLAP_ROUNDS} 轮、每轮 2 次分配，\
         这些分配不得计入本线程统计，实际 {stats:?}"
    );
    assert_eq!(
        stats.bytes_allocated,
        size_of::<u64>() as u64,
        "字节数同样不得被其他线程污染，实际 {stats:?}"
    );
}

// ------------------------------------------------------------------------ miri

// 下面三条测的是同一件事的三个面：**Miri 未运行时，这个类型拒绝给出"没有 UB"的答案**。
//
// 用 `skipped_for_selfcheck` 直接构造状态，而不是用 `RF_SKIP_MIRI` 环境变量 ——
// `set_var` 在 edition 2024 里是 unsafe 的，而 cargo 并行跑测试，
// 那是真实的数据竞争。要测的是"处于该状态时的行为"，不是"怎么进入该状态"。

/// CLAIM: `skipped()` 为真时 `reported_ub()` 确实 panic。
///
/// 这是 FR-019 在类型层面的强制。若它返回 `false`，
/// `assert!(!out.reported_ub())` 会在 Miri 缺席时照常通过，验收变成自欺。
#[test]
#[should_panic(expected = "reported_ub() 在 Miri 未运行时被调用")]
fn reported_ub_panics_when_miri_skipped() {
    let out = rf_harness::miri::MiriOutcome::skipped_for_selfcheck("selfcheck");
    assert!(out.skipped());
    let _ = out.reported_ub(); // 必须在此 panic
}

/// CLAIM: 跳过时 `stderr_contains` 同样 panic —— 否则
/// `assert!(!out.stderr_contains(..))` 形式的断言会变成恒真
/// （stderr 为空时任何 contains 都返回 false）。
#[test]
#[should_panic(expected = "stderr_contains() 在 Miri 未运行时被调用")]
fn stderr_contains_panics_when_miri_skipped() {
    let out = rf_harness::miri::MiriOutcome::skipped_for_selfcheck("selfcheck");
    let _ = out.stderr_contains("Undefined Behavior");
}

/// CLAIM: 跳过时 `skipped()` 与 `skip_reason()` 可安全查询，用于把 `ub_verdict` 记为 `n/a`。
/// 这两个查询是**唯一**在跳过状态下不 panic 的接口 —— 它们是给出 `n/a` 的正当路径。
#[test]
fn skip_reason_is_queryable_without_panic() {
    let out = rf_harness::miri::MiriOutcome::skipped_for_selfcheck("selfcheck reason");
    assert!(out.skipped());
    assert_eq!(
        out.skip_reason(),
        Some("selfcheck reason"),
        "跳过时 MUST 能说明原因，供 OBSERVATIONS 记录为何 ub_verdict = n/a"
    );
}

// ------------------------------------------------------------------------- env

/// CLAIM: `EnvironmentRecord::to_markdown()` 产出 data-model §9 要求的全部字段。
#[test]
fn env_record_contains_all_required_fields() {
    let md = rf_harness::env::record()
        .with_command("cargo test -p rf-harness")
        .to_markdown();
    for field in [
        "rustc_stable",
        "rustc_nightly",
        "edition",
        "kernel",
        "arch",
        "target",
        "command",
    ] {
        assert!(md.contains(field), "环境块缺字段 `{field}`：\n{md}");
    }
    assert!(
        md.starts_with("## 环境记录"),
        "环境块标题 MUST 与 shell 版一致：\n{md}"
    );
    assert!(
        md.contains("cargo test -p rf-harness"),
        "with_command 未生效：\n{md}"
    );
}

/// CLAIM: 采集到的 stable 版本确实可用（而非静默降级为 UNAVAILABLE）。
/// 若这条失败，说明 pinned 工具链没就位，后续所有环境记录都不可信。
#[test]
fn stable_toolchain_is_reachable() {
    let rec = rf_harness::env::record();
    assert_ne!(
        rec.rustc_stable, "UNAVAILABLE",
        "无法执行 `rustc -Vv`——pinned 工具链未就位（rust-toolchain.toml / R-01）"
    );
    assert!(
        rec.rustc_stable.starts_with("1.98.0"),
        "实际：{}",
        rec.rustc_stable
    );
}
