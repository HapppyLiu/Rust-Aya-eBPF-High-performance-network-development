//! EXPECT: E0277
//! CLAIM: `Arc<T>` 的 `Send` 要求 `T: Send + Sync`。`RefCell<u32>` 不是 `Sync`，
//! 故 `SharedMut: !Send`。`Arc` 不提升能力。

use std::cell::RefCell;
use std::sync::Arc;

pub struct SharedMut {
    inner: Arc<RefCell<u32>>,
}

fn assert_send<T: Send>() {}

pub fn check() {
    assert_send::<SharedMut>();
}
