//! EXPECT: E0277
//! CLAIM: `Shared` 内含 `Rc<u32>`，`Rc` 显式 `!Send`，故 `Shared: !Send`。

use std::rc::Rc;

pub struct Shared {
    handle: Rc<u32>,
}

fn assert_send<T: Send>() {}

pub fn check() {
    assert_send::<Shared>();
}
