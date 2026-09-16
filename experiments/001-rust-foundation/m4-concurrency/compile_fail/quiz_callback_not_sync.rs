//! EXPECT: E0277
//! CLAIM: trait object 的 auto trait 集合就是写在类型里的那些。
//! `dyn Fn() + Send` 只标了 `Send`，故 `Callback: !Sync`。

pub struct Callback {
    f: Box<dyn Fn() + Send>,
}

fn assert_sync<T: Sync>() {}

pub fn check() {
    assert_sync::<Callback>();
}
