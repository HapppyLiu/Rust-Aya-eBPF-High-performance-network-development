//! EXPECT: E0502
//! CLAIM: 在不可变借用仍活跃时取可变借用（`Vec::push`），被借用检查器拒绝（E0502）。
//!
//! C-03：可变借用不能与不可变借用共存。
//!
//! 关注点：这条规则防的正是 C 里"边遍历边修改容器"的经典 bug ——
//! `push` 可能触发 realloc，此后所有旧指针悬垂。
//! Rust 把这类运行期灾难提前成了编译错误。

pub fn demo(data: &mut Vec<u8>) -> u8 {
    let first = &data[0];
    data.push(4);
    *first
}
