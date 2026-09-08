//! EXPECT: E0277
//! CLAIM: `Cell<u32>` 显式 `!Sync`，因此 `&Cell<u32>` 不是 `Send`。
//! `thread::scope` 的闭包仍要求 `Send`。共享内部可变性被类型系统拒绝。
//! 这是 US4 AS1：不能的是共享，不是移动（移动见 `move_cell_to_thread`）。

use std::cell::Cell;
use std::thread;

pub fn share_cell() {
    let cell = Cell::new(0u32);
    thread::scope(|scope| {
        scope.spawn(|| {
            cell.set(1);
        });
        scope.spawn(|| {
            cell.set(2);
        });
    });
}
