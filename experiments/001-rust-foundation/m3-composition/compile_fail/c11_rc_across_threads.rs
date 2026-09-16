//! EXPECT: E0277
//! CLAIM: `Rc<T>` 显式 `!Send`。即使 `move` 拿走所有权，也不能送进 `thread::spawn`
//!（`spawn` 要求 `Send`）。这是智能指针合同的跨线程一侧；借用逃逸见 C-10。

use std::rc::Rc;

pub fn send_rc() {
    let r = Rc::new(1u32);
    std::thread::spawn(move || {
        let _v = *r;
    });
}
