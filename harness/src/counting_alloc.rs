//! 确定性分配计数（R-07 / harness-api.md §counting_alloc）。
//!
//! # 为什么是分配次数而不是耗时
//!
//! Constitution IX 要求"所有'高性能'结论 MUST 通过实际测量验证"，而本 Feature 的
//! Technical Context 明确不追求性能。二者的正确交集不是"随便测个时间"，而是
//! **不产生需要 benchmark 的性能主张**，把"代价"这个话题限制在确定性可测量量上。
//!
//! 分配次数正好满足：它由程序结构决定，不受调度、缓存、频率影响，
//! 因此**可以**写进 `#[test]` 稳定断言（FR-003）。耗时不行 —— 它只能是 NON-ASSERTION。
//!
//! 这直接服务于 US3 AS1："预测迭代器链的求值顺序与实际发生的分配次数"。
//!
//! # 教学价值（C-24）
//!
//! 学习者必须亲手实现 `GlobalAlloc` 才能得到计数，从而具体回答
//! "`no_std` 下堆分配能力由谁提供"—— 那正是 US7 AS2 的问题。

//! # 为什么计数器是 thread-local 而不是进程全局
//!
//! 本模块存在的**唯一理由**是提供确定性可断言量（FR-003）。
//! 进程全局计数器做不到这一点：`cargo test` 默认并行跑测试，
//! 另一个线程的分配会被算进你的统计里，同一份代码时而通过时而失败。
//!
//! 因此计数器按线程隔离，[`measure`] 只统计**调用线程**的分配活动。
//! 这个语义边界是刻意的，其代价写在 [`measure`] 的文档里。

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

/// 单个线程的计数快照。
///
/// `live` / `peak` 用有符号类型：线程可以释放**别的**线程分配的内存，
/// 使本线程的净存活量为负。用 `u64` 会在此处下溢成天文数字。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Counters {
    allocs: usize,
    deallocs: usize,
    reallocs: usize,
    bytes_allocated: u64,
    live: i64,
    peak: i64,
}

impl Counters {
    const ZERO: Self = Self {
        allocs: 0,
        deallocs: 0,
        reallocs: 0,
        bytes_allocated: 0,
        live: 0,
        peak: 0,
    };
}

thread_local! {
    /// `const` 初始化 + `Copy` 且无 `Drop` 的类型 —— 这两点合起来保证访问 TLS
    /// **不会分配**，否则在 `GlobalAlloc` 内部触碰它就会无限递归。
    static COUNTERS: Cell<Counters> = const { Cell::new(Counters::ZERO) };
}

/// 读改写本线程计数器。
///
/// 用 `try_with`：线程退出、TLS 已析构时访问会失败，此时**静默丢弃**这次计数。
/// 那些分配不属于任何 `measure` 区间，漏掉它们不影响任何断言。
fn update(f: impl FnOnce(&mut Counters)) {
    let _ = COUNTERS.try_with(|cell| {
        let mut c = cell.get();
        f(&mut c);
        cell.set(c);
    });
}

fn read() -> Counters {
    COUNTERS.try_with(Cell::get).unwrap_or(Counters::ZERO)
}

/// 包装 [`System`] 并统计分配活动的全局分配器。
///
/// 在实验 crate 中启用：
///
/// ```ignore
/// #[global_allocator]
/// static A: CountingAllocator = CountingAllocator::new();
/// ```
///
/// # 全局性
///
/// 一旦注册，它对**整个 crate**（包括测试框架自身的分配）生效。
/// 使用它的 crate MUST 在 `OBSERVATIONS.md` 中说明这一点。
///
/// 注意"分配器是全局的"与"计数是按线程的"并不矛盾：
/// 所有线程的分配都经过本分配器，但每次分配只累加**发起该分配的那个线程**的计数器。
pub struct CountingAllocator;

impl CountingAllocator {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for CountingAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// 记录一次净增长并维护峰值。
///
/// 计数器按线程隔离，所以这里不需要 CAS 循环 —— 没有并发写者。
fn record_growth(c: &mut Counters, bytes: u64) {
    c.bytes_allocated += bytes;
    c.live += bytes as i64;
    c.peak = c.peak.max(c.live);
}

// SAFETY:
// - 有效性：本实现不自行管理内存，全部请求原样转发给 `System`，
//   由它保证返回的指针要么为空、要么指向满足 `layout` 的可用分配。
// - 对齐：`layout` 未被修改即转发，`System` 保证返回指针满足 `layout.align()`。
// - 别名：计数器是独立的 thread-local `Cell`，与被分配的内存不重叠；
//   计数操作不产生指向用户内存的引用，因此不引入别名。
//   `Cell` 的读改写不重入（`const` 初始化的无 `Drop` TLS 不会分配），
//   因此不会在分配路径上递归回到本实现。
// - provenance：`ptr` 由 `System::alloc`/`realloc` 产生并原样传回 `System::dealloc`，
//   provenance 全程未被截断或伪造；本实现不做任何指针运算。
// - 生命周期：不适用 —— 本类型不持有任何被分配内存的引用，
//   分配的存活期完全由调用方与 `System` 决定。
// - 额外约束（`GlobalAlloc` 要求）：调用方保证 `dealloc`/`realloc` 收到的 `ptr`
//   来自本分配器同一 `layout` 的分配；该义务由 `GlobalAlloc` 契约转嫁给调用方，
//   本实现只需保证不破坏它，而转发实现天然满足。
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: `layout` 由调用方按 `GlobalAlloc` 契约提供（size/align 合法），原样转发。
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            update(|c| {
                c.allocs += 1;
                record_growth(c, layout.size() as u64);
            });
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        update(|c| {
            c.deallocs += 1;
            c.live -= layout.size() as i64;
        });
        // SAFETY: 调用方保证 `ptr` 来自本分配器、且 `layout` 与当初分配时一致（`GlobalAlloc` 契约）。
        unsafe { System.dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: 调用方保证 `ptr`/`layout` 匹配且 `new_size` 合法（`GlobalAlloc` 契约）。
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            update(|c| {
                c.reallocs += 1;
                let old = layout.size() as u64;
                let new = new_size as u64;
                if new > old {
                    record_growth(c, new - old);
                } else {
                    c.live -= (old - new) as i64;
                }
            });
        }
        new_ptr
    }
}

/// 某段代码执行期间的分配活动快照。每个字段都是**确定性**的 → 可直接写进 `#[test]` 断言。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocStats {
    /// 成功的 `alloc` 次数。
    pub allocs: usize,
    /// `dealloc` 次数。
    pub deallocs: usize,
    /// 成功的 `realloc` 次数。
    pub reallocs: usize,
    /// 累计**请求**的字节数（不含分配器内部开销）。
    pub bytes_allocated: u64,
    /// 期间的峰值净分配字节数。
    pub peak_bytes: u64,
}

/// 测量闭包执行期间的分配活动，返回闭包结果与统计。
///
/// # 统计范围：仅**调用线程**
///
/// 计数器按线程隔离，因此结果不受 `cargo test` 并行度影响 ——
/// 这正是 `AllocStats` 各字段能进 `#[test]` 断言的前提（FR-003）。
///
/// 代价是两条边界，二者都是刻意选择：
///
/// - 闭包内**新起线程**的分配**不计入**。要测它们，在那个线程内部各自调用 `measure`。
/// - 闭包若释放了别的线程分配的内存，`deallocs` 会计入而 `allocs` 不会。
///   `peak_bytes` 对此做了 `saturating` 处理，不会因净存活量为负而回绕。
///
/// 换来的是"同一份代码每次跑出同一个数"。若改用进程全局计数器，
/// 断言会随并行调度时而通过时而失败 —— 那样的量不配写进 `#[test]`。
///
/// # 字节数的含义
///
/// `bytes_allocated` 与 `peak_bytes` 反映**请求的**字节数，不含分配器内部开销
/// （头部、对齐填充、size class 向上取整）。这是刻意的：把 libc 的实现细节引进断言，
/// 断言就会在换一个 malloc 实现时失效，而那与被学习的语言机制无关。
pub fn measure<R>(f: impl FnOnce() -> R) -> (R, AllocStats) {
    let before = read();
    // 峰值从本次测量的起点重新起算：它是"这段代码用了多少"，而非线程历史最高水位。
    update(|c| c.peak = c.live);
    let base_live = before.live;

    let result = f();

    let after = read();

    let stats = AllocStats {
        allocs: after.allocs - before.allocs,
        deallocs: after.deallocs - before.deallocs,
        reallocs: after.reallocs - before.reallocs,
        bytes_allocated: after.bytes_allocated - before.bytes_allocated,
        peak_bytes: u64::try_from(after.peak - base_live).unwrap_or(0),
    };
    (result, stats)
}
