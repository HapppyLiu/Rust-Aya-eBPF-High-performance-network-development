//! EXPECT: E0499
//! CLAIM: 两个活跃区间重叠的 `&mut` 指向同一位置，被借用检查器拒绝（E0499）。
//!
//! C-03：同一时刻不能存在两个可变借用。
//!
//! 关注点：这与 `SharedCounter::provoke_runtime_conflict` 是**同一条规则**，
//! 只是执行时机不同 —— 那边 panic，这边编译失败。
//! 两个借用必须"同时存活"才会冲突：`_first` 在 `_second` 之后仍被使用，
//! 否则 NLL 会让第一个借用提前结束，这段代码就合法了。

pub struct Counter {
    pub value: i64,
}

pub fn demo(c: &mut Counter) {
    let first = &mut c.value;
    let second = &mut c.value;
    *second += 1;
    *first += 1;
}
