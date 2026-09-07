//! C-08 Error handling —— 稳定断言。
//!
//! 对应 `acceptance/criteria/c08.md`。

use m3_composition::c08::{AppErr, ParseErr, parse_port, parse_port_manual};

/// CLAIM: 手写 `match` + `From` 与 `?` 对同一组输入给出相同的 `Result`。
#[test]
fn manual_and_question_mark_agree_on_every_path() {
    let cases: &[(&str, Result<u16, AppErr>)] = &[
        ("443", Ok(443)),
        ("", Err(AppErr::Parse(ParseErr::Empty))),
        ("x", Err(AppErr::Parse(ParseErr::BadDigit))),
        ("0", Err(AppErr::OutOfRange)),
        ("8080", Err(AppErr::OutOfRange)),
    ];
    for &(input, ref expected) in cases {
        assert_eq!(parse_port_manual(input), *expected, "manual {input:?}");
        assert_eq!(parse_port(input), *expected, "? {input:?}");
        assert_eq!(
            parse_port_manual(input),
            parse_port(input),
            "two paths diverged on {input:?}"
        );
    }
}

/// CLAIM: `?` 缺少 `From` 转换时，编译器报 E0277（trait bound not satisfied），
/// 拒绝发生在运行之前。
#[test]
fn missing_from_impl_is_e0277() {
    rf_harness::compile_fail::expect_errors("compile_fail/c08_missing_from.rs", &["E0277"]);
}
