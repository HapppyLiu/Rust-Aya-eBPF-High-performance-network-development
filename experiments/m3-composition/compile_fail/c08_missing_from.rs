//! EXPECT: E0277
//! CLAIM: `?` 把内层 `Result<_, Inner>` 传播到外层 `Result<_, Outer>` 时，
//! 要求 `Outer: From<Inner>`。本样本两个错误类型之间没有 `From`，
//! 编译器在单态化之前就拒绝（trait bound not satisfied）。

enum Inner {
    Boom,
}

enum Outer {
    Other,
}

fn inner() -> Result<(), Inner> {
    Err(Inner::Boom)
}

pub fn outer() -> Result<(), Outer> {
    inner()?;
    Ok(())
}
