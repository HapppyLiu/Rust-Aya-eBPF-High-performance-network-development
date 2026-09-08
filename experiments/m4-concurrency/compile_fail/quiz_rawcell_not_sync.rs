//! EXPECT: E0277
//! CLAIM: `UnsafeCell<T>` 显式 `!Sync`，故 `RawCell: !Sync`。
//! 它仍可以是 `Send`（`T: Send` 时）—— 内部可变性的"去皮"形态。

use core::cell::UnsafeCell;

pub struct RawCell {
    inner: UnsafeCell<u32>,
}

fn assert_sync<T: Sync>() {}

pub fn check() {
    assert_sync::<RawCell>();
}
