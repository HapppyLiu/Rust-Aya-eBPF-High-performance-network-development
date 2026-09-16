//! EXPECT: E0277
//! CLAIM: `PhantomData<T>` 在 auto trait 推导中与 `T` 一致。
//! `*const u8` 是 `!Send`，故 `Marked: !Send`（零大小字段照样参与推导）。

use core::marker::PhantomData;

pub struct Marked {
    id: u32,
    _marker: PhantomData<*const u8>,
}

fn assert_send<T: Send>() {}

pub fn check() {
    assert_send::<Marked>();
}
