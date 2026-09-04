//! EXPECT: E0382
//! CLAIM: 所有权转移后读取原绑定，被借用检查器在编译期拒绝（E0382 "use of moved value"）。
//!
//! C-01：所有权转移之后，原绑定不可再用。
//!
//! 关注点：错误发生在**编译期**，且指向的是"值已被移出"这一簿记事实，
//! 而非任何运行期的内存状态 —— 那块栈空间其实原封未动。

pub struct Boxed {
    pub data: Vec<u8>,
}

pub fn take(_b: Boxed) {}

pub fn demo() -> usize {
    let b = Boxed { data: vec![1, 2, 3] };
    take(b);
    b.data.len() // b 的所有权已转移给 take
}
