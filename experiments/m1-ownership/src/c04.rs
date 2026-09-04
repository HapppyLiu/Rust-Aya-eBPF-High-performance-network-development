//! C-04 Lifetime —— 被 `c04_lifetime` 的 example 与 test 复用的最小设施。
//!
//! 生命周期标注不产生任何运行期代码。它是**给借用检查器看的约束声明**：
//! 说明返回的引用可以活多久、由哪个入参决定。
//!
//! 本模块的三组对象分别对应三个问题：
//!
//! - [`longest`] / [`first_word`]：省略规则什么时候够用、什么时候不够；
//! - [`Excerpt`]：结构体持有引用时，标注表达的是什么约束；
//! - [`Tagged`]：不占空间的类型参数如何影响类型检查（`PhantomData`）。

use std::marker::PhantomData;

/// 需要**显式**标注才能编译的函数。
///
/// 两个入参、一个引用返回值 —— 省略规则在这里失效：
/// 规则只会在"恰好一个入参引用"或"有 `&self`"时才能推出返回值的来源。
/// 两个候选来源时，编译器不猜，要求作者说明。
///
/// `'a` 在这里的含义是：返回值的有效期，不超过 `a` 与 `b` 中**较短**的那个。
#[must_use]
pub fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

/// **不需要**显式标注的函数：只有一个入参引用，省略规则足以推出返回值来源。
///
/// 它的完整形式是 `fn first_word<'a>(s: &'a str) -> &'a str`。
#[must_use]
pub fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

/// 持有引用的结构体。
///
/// `'a` 表达的约束：`Excerpt` 的实例**不能**比它引用的字符串活得更久。
/// 没有这个标注，编译器无法知道 `part` 指向的数据什么时候失效。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Excerpt<'a> {
    pub part: &'a str,
}

impl<'a> Excerpt<'a> {
    #[must_use]
    pub fn new(part: &'a str) -> Self {
        Self { part }
    }

    /// 返回值绑定的是 `'a`（数据的生命周期），而**不是** `&self` 的生命周期。
    ///
    /// 省略规则在有 `&self` 时会把返回值绑定到 `&self`，那比这里想要的更严格：
    /// 返回的切片其实只依赖被引用的原始数据，不依赖 `Excerpt` 本身活多久。
    /// 所以这里显式写出 `'a` 来放宽约束。
    #[must_use]
    pub fn part(&self) -> &'a str {
        self.part
    }
}

/// 用类型参数区分用途、但**不占运行期空间**的类型。
///
/// `PhantomData<T>` 的大小为 0：它不存任何东西，只是告诉编译器
/// "本类型在类型层面与 `T` 有关"，从而让类型检查把 `Tagged<A>` 与 `Tagged<B>` 视为不同类型。
///
/// 这个"零成本的类型级区分"是后续 eBPF map 类型标注一类设计的基础形状。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tagged<T> {
    pub raw: u32,
    _marker: PhantomData<T>,
}

impl<T> Tagged<T> {
    #[must_use]
    pub fn new(raw: u32) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }
}

/// 供 [`Tagged`] 使用的标记类型之一。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ingress;

/// 供 [`Tagged`] 使用的标记类型之二。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Egress;
