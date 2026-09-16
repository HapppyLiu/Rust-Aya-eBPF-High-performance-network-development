//! C-04 Lifetime —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c04-lifetime.md`。

use m1_ownership::c04::{Egress, Excerpt, Ingress, Tagged, first_word, longest};
use std::marker::PhantomData;

/// CLAIM: `'a` 表达的是"返回值不超过两个入参中较短的那个"，而非具体时长。
#[test]
fn longest_returns_the_longer_input() {
    let a = String::from("longer string");
    let b = String::from("short");
    assert_eq!(longest(&a, &b), "longer string");
    assert_eq!(longest(&b, &a), "longer string");
}

/// CLAIM: 省略规则在只有一个入参引用时足以推出返回值来源。
#[test]
fn elision_suffices_for_a_single_input_reference() {
    assert_eq!(first_word("hello world"), "hello");
    assert_eq!(first_word("single"), "single");
}

/// CLAIM: 生命周期标注**不改变运行期行为**：带标注与省略标注的等价函数结果相同。
#[test]
fn lifetime_annotations_are_compile_time_only() {
    fn elided(s: &str) -> &str {
        s
    }
    // clippy 正确地指出 'a 可以省略 —— 而这恰恰是本测试要断言的：
    // 省略与不省略在语义上等价。刻意保留标注，否则两个函数将完全相同，断言失去意义。
    #[allow(clippy::needless_lifetimes)]
    fn annotated<'a>(s: &'a str) -> &'a str {
        s
    }

    let text = String::from("erased at compile time");
    assert_eq!(elided(&text), annotated(&text));
}

/// CLAIM: 结构体可以持有引用，前提是标注出"实例不得比被引用数据活得更久"。
#[test]
fn struct_can_hold_a_reference_with_an_explicit_lifetime() {
    let text = String::from("first sentence. second sentence.");
    let excerpt = Excerpt::new(&text[..15]);
    assert_eq!(excerpt.part(), "first sentence.");
}

/// CLAIM: `Excerpt::part` 返回 `&'a str` 而非 `&'_ str`：返回值不依赖 `Excerpt` 本身的存活。
///
/// 若签名是省略形式（绑定到 `&self`），`part` 在 `excerpt` 被 drop 后即失效，
/// 这个测试将无法编译。
#[test]
fn part_outlives_the_excerpt_itself() {
    let text = String::from("outlives the wrapper");
    let part = {
        let excerpt = Excerpt::new(&text);
        excerpt.part() // excerpt 在此结束，part 仍有效
    };
    assert_eq!(part, "outlives the wrapper");
}

/// CLAIM: `PhantomData` 的大小为 0 —— 这是标准库**文档化的保证**，可作稳定断言。
#[test]
fn phantom_data_is_zero_sized() {
    assert_eq!(size_of::<PhantomData<Ingress>>(), 0);
    assert_eq!(size_of::<PhantomData<Egress>>(), 0);
    assert_eq!(size_of::<PhantomData<[u8; 4096]>>(), 0, "与 T 的大小无关");
}

/// CLAIM: 类型级区分是**零成本**的：加了 `PhantomData` 的类型与裸字段同样大小。
#[test]
fn phantom_tagging_costs_no_space() {
    assert_eq!(size_of::<Tagged<Ingress>>(), size_of::<u32>());
    assert_eq!(size_of::<Tagged<Egress>>(), size_of::<u32>());
    assert_eq!(align_of::<Tagged<Ingress>>(), align_of::<u32>());
}

/// CLAIM: 运行期表示相同，类型却不可互换。
///
/// 不可互换性由 `compile_fail/c04_phantom_type_mismatch.rs`（E0308）断言；
/// 这里断言的是它的另一半：**运行期确实没有差别**。
#[test]
fn differently_tagged_values_share_the_same_runtime_representation() {
    let ingress = Tagged::<Ingress>::new(7);
    let egress = Tagged::<Egress>::new(7);

    assert_eq!(ingress.raw, egress.raw);
    assert_eq!(size_of_val(&ingress), size_of_val(&egress));
}

/// CLAIM: 编译期违规样本的错误码断言（US1 AS1：预测与实际诊断一致）。
#[test]
fn compile_time_violations_report_expected_codes() {
    rf_harness::compile_fail::expect_errors("compile_fail/c04_missing_lifetime.rs", &["E0106"]);
    rf_harness::compile_fail::expect_errors("compile_fail/c04_dangling_ref.rs", &["E0597"]);
    rf_harness::compile_fail::expect_errors(
        "compile_fail/c04_phantom_type_mismatch.rs",
        &["E0308"],
    );
}
