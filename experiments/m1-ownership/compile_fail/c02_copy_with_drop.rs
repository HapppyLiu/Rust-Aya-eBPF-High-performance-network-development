//! EXPECT: E0184
//! CLAIM: 同时 derive `Copy` 与实现 `Drop` 被语言禁止（E0184），二者互斥由编译器强制。
//!
//! C-02：`Copy` 与 `Drop` 互斥。
//!
//! 关注点：这是语言层面强制的，不是约定。若一个值既能被随意按位复制、
//! 又带销毁行为，则 N 份副本各自销毁一次同一份资源。
//! 标准库把理由写在了 `core/src/marker.rs` 的 `Copy` 文档里。

#[derive(Clone, Copy)]
pub struct Handle {
    pub fd: i32,
}

impl Drop for Handle {
    fn drop(&mut self) {}
}
