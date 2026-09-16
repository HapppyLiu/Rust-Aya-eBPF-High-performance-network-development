//! 静态数组 bump 分配器（C-24）。
//!
//! 只使用 `core`：本文件同时被
//! - 裸机 crate `m7-nostd` 注册为 `#[global_allocator]`；
//! - host 侧 `harness/tests/c24_bump_alloc.rs` 经 `#[path]` 引入，交给 Miri 检查 unsafe 实现。
//!
//! 不提供逐对象释放：`dealloc` 是空操作。耗尽后 `alloc` 返回空指针。

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

/// 教学用 arena。容量只为证明"分配入口由谁提供"，不是通用堆。
pub const ARENA_SIZE: usize = 64 * 1024;

#[repr(C, align(16))]
struct Arena {
    buf: UnsafeCell<[u8; ARENA_SIZE]>,
    offset: AtomicUsize,
}

/// 从 [`Arena`] 里切互不重叠区间的 bump 分配器。
pub struct BumpAlloc {
    arena: Arena,
}

// SAFETY:
// - 有效性：不适用类型本身 —— `BumpAlloc` 不因 `Sync` 而解引用用户指针。
// - 对齐：不适用。
// - 别名：`offset` 用原子 CAS 把缓冲划分成互不重叠的已分配区间；
//   每个 `alloc` 返回的指针只覆盖自己的 `[aligned, aligned+size)`。
//   调用方按 `GlobalAlloc` 契约不会对同一区间制造别名冲突。
//   `UnsafeCell` 仅用于获得 `*mut u8` 基址，从不把整块缓冲转成 `&mut [u8]`。
// - provenance：基址来自 `self.arena.buf` 这一次分配（静态存储），
//   `add` 不把指针铸成整数再铸回来。
// - 生命周期：`BumpAlloc` 与 arena 同寿；已分配指针的寿命由调用方与
//   `GlobalAlloc` 契约约束，不因 `Sync` 延长或缩短。
unsafe impl Sync for BumpAlloc {}

impl BumpAlloc {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            arena: Arena {
                buf: UnsafeCell::new([0; ARENA_SIZE]),
                offset: AtomicUsize::new(0),
            },
        }
    }

    /// 已 bump 出去的字节数（含对齐填充）。用于 host 侧断言，不是稳定的跨架构 ABI。
    #[must_use]
    #[allow(dead_code)] // 裸机产物不读它；host 侧 `c24_bump_alloc` 测试会读。
    pub fn bump_offset(&self) -> usize {
        self.arena.offset.load(Ordering::Relaxed)
    }
}

// SAFETY:
// - 有效性：返回的指针要么为空，要么指向 `buf` 内一段 `[aligned, aligned+size)`，
//   该区间尚未交给其他调用方（CAS 成功才提交 `new_offset`）。
//   `layout.size()` 非零由 `GlobalAlloc` 调用方保证。
// - 对齐：`aligned` 向上取整到 `layout.align()` 的倍数；`Arena` 本身 align 16，
//   故基址满足常见对齐；更大的 `layout.align()` 仍由 bump 公式保证。
// - 别名：成功的 CAS 使各分配区间不相交；本实现不把重叠区间交给两个调用方。
// - provenance：`buf.get()` 得到指向静态数组的指针，`add(aligned)` 不越出
//   `ARENA_SIZE`（提交前已检查 `new_offset <= ARENA_SIZE`）。
// - 生命周期：指针在 `BumpAlloc` 存活期内有效。`dealloc` 不回收，因此不存在
//   "已释放再返回同一地址"的别名窗口；耗尽后返回空指针，由调用方走 OOM 路径。
unsafe impl GlobalAlloc for BumpAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let mut offset = self.arena.offset.load(Ordering::Relaxed);
        loop {
            let aligned = offset.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
            let new_offset = match aligned.checked_add(size) {
                Some(n) => n,
                None => return ptr::null_mut(),
            };
            if new_offset > ARENA_SIZE {
                return ptr::null_mut();
            }
            match self.arena.offset.compare_exchange_weak(
                offset,
                new_offset,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    let base = self.arena.buf.get().cast::<u8>();
                    // SAFETY: `aligned < new_offset <= ARENA_SIZE`，加法落在同一块静态数组内。
                    return unsafe { base.add(aligned) };
                }
                Err(actual) => offset = actual,
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // bump：不回收。泄漏是本实现的显式契约，不是疏忽。
    }
}
