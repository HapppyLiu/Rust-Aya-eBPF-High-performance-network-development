//! EXPECT: E0277
//! CLAIM: `&T: Send` 要求 `T: Sync`，不是 `T: Send`。
//! `Cell<u32>` 不是 `Sync`，故 `Peek: !Send`（`T: Sync ⟺ &T: Send`）。

use std::cell::Cell;

pub struct Peek<'a> {
    view: &'a Cell<u32>,
}

fn assert_send<T: Send>() {}

pub fn check() {
    assert_send::<Peek<'static>>();
}
