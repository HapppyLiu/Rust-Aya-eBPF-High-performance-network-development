//! T024 题集的 12 个自定义类型（与 `acceptance/send-sync-quiz.md` 定义一致）。
//!
//! 只供正向 `assert_send` / `assert_sync`。负向判定在 `compile_fail/quiz_*.rs`，
//! 那些样本由 `rustc` 单独编译，不能依赖本 crate。

use core::cell::{Cell, RefCell, UnsafeCell};
use core::marker::PhantomData;
use core::sync::atomic::AtomicUsize;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};

pub struct Tick {
    pub count: u64,
}

pub struct Slot {
    pub value: Cell<u32>,
}

pub struct Shared {
    pub handle: Rc<u32>,
}

pub struct RawSlot {
    pub ptr: *mut u8,
    pub len: usize,
}

pub struct Guarded {
    pub inner: Mutex<Cell<u32>>,
}

pub struct SharedMut {
    pub inner: Arc<RefCell<u32>>,
}

pub struct Held<'a> {
    pub guard: MutexGuard<'a, u32>,
}

pub struct Marked {
    pub id: u32,
    pub _marker: PhantomData<*const u8>,
}

pub struct Ticker {
    pub hits: AtomicUsize,
}

pub struct Callback {
    pub f: Box<dyn Fn() + Send>,
}

pub struct RawCell {
    pub inner: UnsafeCell<u32>,
}

pub struct Peek<'a> {
    pub view: &'a Cell<u32>,
}
