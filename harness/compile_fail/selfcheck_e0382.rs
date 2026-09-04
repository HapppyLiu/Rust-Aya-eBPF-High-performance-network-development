//! EXPECT: E0382
//! CLAIM: 值被移动后再使用会被借用检查器拒绝。
//!
//! 本文件是 **harness 自检**用的固定样本，不是学习材料。
//! 它的作用是给 `compile_fail::expect_errors` 一个"一定会失败、且失败原因确定"的输入，
//! 用来验证断言器本身不说谎。学习用的 compile_fail 样本在各 `experiments/mN-*/` 下。

pub fn moved_value_is_rejected() {
    let owned = String::from("x");
    let _moved = owned;
    let _use_after_move = owned.len();
}
