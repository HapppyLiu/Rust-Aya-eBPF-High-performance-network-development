//! EXPECT: E0040
//! CLAIM: 显式调用 `Drop::drop` 被编译器拒绝（E0040），因为编译器已在作用域末尾安排了一次销毁。
//!
//! C-01：`Drop::drop` 不能被手动调用。
//!
//! 关注点：禁止的理由不是风格，而是 double free —— 编译器已经在作用域末尾
//! 安排了一次销毁，手动再调一次就是第二次。
//! 正确做法是 `drop(value)`（`mem::drop`，取得所有权后让它自然销毁）。

pub struct Resource;

impl Drop for Resource {
    fn drop(&mut self) {}
}

pub fn demo() {
    let r = Resource;
    r.drop();
}
