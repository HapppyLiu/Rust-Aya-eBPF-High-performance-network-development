//! C-09 Iterator —— 被 `c09_iterator` 的 example 与 test 复用的最小设施。
//!
//! 同一条"翻倍再收集 / 求和"逻辑，三种写法：
//! 手写循环、惰性链 + `sum`、惰性链 + `collect`。
//! 元素结果应当一致；分配次数不必一致 —— 那正是本实验要断言的差。

/// 手写循环求和：`(0..n)` 每个元素翻倍后累加。不构造堆上容器。
#[must_use]
pub fn double_sum_loop(n: i32) -> i32 {
    let mut total = 0;
    let mut i = 0;
    while i < n {
        total += i * 2;
        i += 1;
    }
    total
}

/// 迭代器链求和。适配器本身不分配；`sum` 也不构造 `Vec`。
#[must_use]
pub fn double_sum_iter(n: i32) -> i32 {
    (0..n).map(|x| x * 2).sum()
}

/// 迭代器链收集。`Range` 的 `size_hint` 是精确的，`map` 原样转发，
/// 因此 `collect::<Vec<_>>` 可以一次按长度分配。
#[must_use]
pub fn double_collect_iter(n: i32) -> Vec<i32> {
    (0..n).map(|x| x * 2).collect()
}

/// 手写 `Vec::new` + `push`。起点容量为 0，增长次数由 `Vec` 的增长策略决定，
/// 不必与 [`double_collect_iter`] 相同。
#[must_use]
pub fn double_collect_loop(n: i32) -> Vec<i32> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        out.push(i * 2);
        i += 1;
    }
    out
}

/// 只构造链、不消费。`Iterator` 本身已是 `must_use`：丢掉它等于什么都没做。
pub fn double_filter_lazy(n: i32) -> impl Iterator<Item = i32> {
    (0..n).map(|x| x * 2).filter(|x| *x > 0)
}
