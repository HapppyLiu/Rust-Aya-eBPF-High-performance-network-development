//! EXPECT: （无——本样本**可以**编译）
//! CLAIM: 用于验证 `expect_errors` 在样本意外编译成功时会 panic。
//!
//! 本文件是 **harness 自检**用的固定样本，不是学习材料。
//! 它是上一个样本的对照：如果 `expect_errors` 对这个文件也"通过"，
//! 说明断言器根本没在检查编译结果。

pub fn compiles_fine() -> usize {
    let owned = String::from("x");
    owned.len()
}
