//! C-12 Send / Sync —— 被 `c12_send_sync` 的 example 与 test 复用的最小设施。
//!
//! `Send`：值可以**移动**到另一个线程。
//! `Sync`：`&T` 是 `Send`，即多个线程可以**同时持有引用**。
//! 含内部可变性 ≠ 不能跨线程；不能的常常是共享，不是移动。

use std::cell::Cell;
use std::sync::Mutex;

/// 编译期探针。能通过类型检查就说明 `T` 满足对应 auto trait。
pub fn assert_send<T: Send>() {}
pub fn assert_sync<T: Sync>() {}

/// 把 `Cell` **移动**到另一个线程。`Cell<u32>: Send`，移动之后原线程碰不到它。
#[must_use]
pub fn move_cell_to_thread(v: u32) -> u32 {
    let cell = Cell::new(v);
    std::thread::spawn(move || cell.get()).join().unwrap()
}

/// 经 `Mutex` 共享一个 `Cell`。`Mutex<Cell<u32>>: Sync`，因为锁把 `Send` 提升成了共享能力。
#[must_use]
pub fn share_mutex_cell(start: u32, add: u32) -> u32 {
    let slot = Mutex::new(Cell::new(start));
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let guard = slot.lock().unwrap();
            guard.set(guard.get() + add);
        });
        scope.spawn(|| {
            let _ = slot.lock().unwrap().get();
        });
    });
    slot.lock().unwrap().get()
}
