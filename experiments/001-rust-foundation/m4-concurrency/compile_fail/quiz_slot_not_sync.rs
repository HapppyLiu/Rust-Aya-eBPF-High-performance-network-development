//! EXPECT: E0277
//! CLAIM: `Slot` 内含 `Cell<u32>`，`Cell` 显式 `!Sync`，故 `Slot: !Sync`。

use std::cell::Cell;

pub struct Slot {
    value: Cell<u32>,
}

fn assert_sync<T: Sync>() {}

pub fn check() {
    assert_sync::<Slot>();
}
