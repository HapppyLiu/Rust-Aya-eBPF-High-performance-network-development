//! EXPECT: E0507
//! CLAIM: 从 `&T` 中移出非 `Copy` 字段被拒绝（E0507），因为借用方无权让所有者的位置失效。
//!
//! C-02：不能从借用中移出值。
//!
//! 关注点：移出会让原位置留下**无效状态**，而原位置并不归本函数所有 ——
//! 借用方无权把所有者的数据搬走。
//! 这正是 `mem::replace` / `mem::take` 存在的理由：它们在搬走的同时放回一个合法值。

pub struct Config {
    pub name: String,
}

pub fn steal(c: &Config) -> String {
    c.name
}
