//! EXPECT: E0277
//! CLAIM: `RawSlot` 内含 `*mut u8`，裸指针显式 `!Send`，故 `RawSlot: !Send`。

pub struct RawSlot {
    ptr: *mut u8,
    len: usize,
}

fn assert_send<T: Send>() {}

pub fn check() {
    assert_send::<RawSlot>();
}
