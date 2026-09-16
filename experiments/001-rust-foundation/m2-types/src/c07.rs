//! C-07 Generic —— 被 `c07_generic` 的 example 与 test 复用的最小设施。
//!
//! - [`max_of`]：`PartialOrd` bound，单态化后每种 `T` 一份机器码；
//! - [`count_gt`]：`Iterator` + `PartialOrd` 组合 bound；
//! - [`CmpValue`]：同一套比较逻辑的 trait 对象版本，用来对照行为等价。

/// 有 bound 的泛型比较。源码一份；`u8` / `u16` / `u32` 各调用一次就会各出一份实例。
#[must_use]
pub fn max_of<T: PartialOrd + Copy>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

/// 迭代器上的阈值计数。`Item` 是 `Iterator` 的关联类型，不是泛型参数。
#[must_use]
pub fn count_gt<I>(items: I, threshold: I::Item) -> usize
where
    I: IntoIterator,
    I::Item: PartialOrd,
{
    items.into_iter().filter(|x| *x > threshold).count()
}

/// 对象安全的取值接口，用来构造与 [`max_of`] 行为等价的动态分发版本。
pub trait CmpValue {
    fn as_i64(&self) -> i64;
}

impl CmpValue for i32 {
    fn as_i64(&self) -> i64 {
        i64::from(*self)
    }
}

impl CmpValue for i16 {
    fn as_i64(&self) -> i64 {
        i64::from(*self)
    }
}

/// 泛型版：调用点已知具体类型，单态化后是直接比较。
#[must_use]
pub fn max_cmp<T: CmpValue>(a: T, b: T) -> i64 {
    let (x, y) = (a.as_i64(), b.as_i64());
    if x >= y { x } else { y }
}

/// 动态分发版：同一套逻辑，实现地址走 vtable。
#[must_use]
pub fn max_cmp_dyn(a: &dyn CmpValue, b: &dyn CmpValue) -> i64 {
    let (x, y) = (a.as_i64(), b.as_i64());
    if x >= y { x } else { y }
}

/// 单态化在布局上的直接后果：同一份结构定义，不同 `T` 对应不同大小。
#[repr(transparent)]
pub struct Wrapper<T>(pub T);
