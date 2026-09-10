//! C-21 FFI —— 布局、双向调用、跨边界所有权、errno 封装。

use core::ffi::c_char;
use core::mem::{align_of, offset_of, size_of};
use std::ffi::CStr;
use std::fmt;
use std::io;

use libc::c_int;

/// 与 `c/roundtrip.c` 的 `struct PacketHdr` 字段顺序、类型一一对应。
///
/// `#[repr(C)]` 是契约的 **Rust 半边**。另一半是 C 编译器排出的
/// `sizeof` / `_Alignof` / `offsetof`，由 `c_layout()` 读回后比对。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketHdr {
    pub kind: u8,
    pub id: u32,
    pub len: u16,
}

/// 双侧布局量。具体数字只在 example 里打印；测试只断言两侧相等。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutReport {
    pub size: usize,
    pub align: usize,
    pub off_kind: usize,
    pub off_id: usize,
    pub off_len: usize,
}

/// C `malloc` 出来、约定由 Rust `free` 的那串字节。与 `roundtrip.c` 字面量对齐。
pub const OWNED_MSG: &str = "m6-owned-by-rust";

/// `open` / `close` 失败时保留的原始内核错误码。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ErrnoError {
    pub code: i32,
}

impl fmt::Display for ErrnoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({})",
            io::Error::from_raw_os_error(self.code),
            self.code
        )
    }
}

impl std::error::Error for ErrnoError {}

unsafe extern "C" {
    fn m6_c_hdr_size() -> usize;
    fn m6_c_hdr_align() -> usize;
    fn m6_c_hdr_off_kind() -> usize;
    fn m6_c_hdr_off_id() -> usize;
    fn m6_c_hdr_off_len() -> usize;
    fn m6_c_add(a: i32, b: i32) -> i32;
    fn m6_c_call_rust_mul(a: i32, b: i32) -> i32;
    fn m6_c_hdr_sum(h: *const PacketHdr) -> i32;
    fn m6_c_fill_hdr(out: *mut PacketHdr);
    fn m6_c_call_rust_hdr_sum() -> i32;
    fn m6_c_alloc_message() -> *mut c_char;
}

/// C 要调的乘法。小整数测试走这里，用 wrapping 避免 debug 溢出 panic。
#[unsafe(no_mangle)]
pub extern "C" fn m6_rust_mul(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b)
}

/// C 填好的 `PacketHdr` 再交回 Rust 求和，验证 C→Rust 结构体指针。
///
/// 必须是 `unsafe fn`：公开 API 解引用了生指针参数（clippy `not_unsafe_ptr_arg_deref`）。
///
/// # Safety
///
/// - 有效性：`h` 为 null（本函数返回 0）或指向一块至少 `size_of::<PacketHdr>()`、
///   已按 `repr(C)` / C `struct PacketHdr` 初始化的内存。
/// - 对齐：指向的对象满足 `_Alignof(struct PacketHdr)`。
/// - 别名：本次读取期间没有并行的可变访问。
/// - provenance：指针来自一次合法的 `PacketHdr` 对象，未越出该对象。
/// - 生命周期：指向的对象活过本次调用；本函数不把指针存出去。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn m6_rust_hdr_sum(h: *const PacketHdr) -> i32 {
    if h.is_null() {
        return 0;
    }
    // SAFETY:
    // - 有效性：非空；调用方（C 的 `m6_c_call_rust_hdr_sum`）传入指向活着的
    //   `struct PacketHdr` 的指针，大小为 `size_of::<PacketHdr>()`。
    // - 对齐：C 侧该结构体按 `_Alignof(struct PacketHdr)` 放置，与 `repr(C)` 一致。
    // - 别名：本读期间 C 没有并行写；Rust 侧没有同时存在的 `&mut PacketHdr`。
    // - provenance：指针来自 C 栈上这一次 `struct PacketHdr h` 的地址，未做越界偏移。
    // - 生命周期：C 的局部变量活过本次调用；本函数不把指针存出去。
    let hdr = unsafe { &*h };
    i32::from(hdr.kind) + hdr.id as i32 + i32::from(hdr.len)
}

#[must_use]
pub fn rust_layout() -> LayoutReport {
    LayoutReport {
        size: size_of::<PacketHdr>(),
        align: align_of::<PacketHdr>(),
        off_kind: offset_of!(PacketHdr, kind),
        off_id: offset_of!(PacketHdr, id),
        off_len: offset_of!(PacketHdr, len),
    }
}

#[must_use]
pub fn c_layout() -> LayoutReport {
    // SAFETY: 无指针参数；C 只返回 `sizeof` 常量。
    // 有效性/对齐/别名/provenance/生命周期均不适用：没有解引用、没有分配。
    let size = unsafe { m6_c_hdr_size() };
    // SAFETY: 同上（`_Alignof` 查询，无内存操作）。
    let align = unsafe { m6_c_hdr_align() };
    // SAFETY: 同上（`offsetof(kind)` 查询，无内存操作）。
    let off_kind = unsafe { m6_c_hdr_off_kind() };
    // SAFETY: 同上（`offsetof(id)` 查询，无内存操作）。
    let off_id = unsafe { m6_c_hdr_off_id() };
    // SAFETY: 同上（`offsetof(len)` 查询，无内存操作）。
    let off_len = unsafe { m6_c_hdr_off_len() };
    LayoutReport {
        size,
        align,
        off_kind,
        off_id,
        off_len,
    }
}

/// Rust→C：把一对加数交给 C。
#[must_use]
pub fn rust_to_c_add(a: i32, b: i32) -> i32 {
    // SAFETY:
    // - 有效性：`m6_c_add` 只吃两个 `int32_t`，无指针。
    // - 对齐：不适用，原因是没有内存操作。
    // - 别名：不适用，原因是没有共享对象。
    // - provenance：不适用，原因是没有指针。
    // - 生命周期：不适用，原因是按值传递整数。
    unsafe { m6_c_add(a, b) }
}

/// C→Rust：C 包装去调 `m6_rust_mul`，再把积带回 Rust。
#[must_use]
pub fn c_to_rust_mul(a: i32, b: i32) -> i32 {
    // SAFETY: 同 `rust_to_c_add`；整数按值往返，无指针。
    // 有效性：C 包装只转发参数。对齐/别名/provenance/生命周期：不适用（无内存操作）。
    unsafe { m6_c_call_rust_mul(a, b) }
}

/// Rust→C：按指针把 `PacketHdr` 交给 C 求和。
#[must_use]
pub fn rust_to_c_hdr_sum(hdr: &PacketHdr) -> i32 {
    let p = hdr as *const PacketHdr;
    // SAFETY:
    // - 有效性：`p` 指向仍然活着的 `PacketHdr`，大小匹配。
    // - 对齐：`hdr` 是 Rust 侧 `repr(C)` 值，满足 `align_of::<PacketHdr>()`。
    // - 别名：共享引用，C 只读；本调用期间没有 `&mut`。
    // - provenance：来自 `hdr` 这一次借用，未做偏移。
    // - 生命周期：借用活过本次 C 调用；C 不保存指针。
    unsafe { m6_c_hdr_sum(p) }
}

/// C 填字段，Rust 读回来。验证 C 写入的布局 Rust 能按同名字段读到。
#[must_use]
pub fn c_filled_hdr() -> PacketHdr {
    let mut hdr = PacketHdr {
        kind: 0,
        id: 0,
        len: 0,
    };
    let p = &raw mut hdr;
    // SAFETY:
    // - 有效性：`p` 指向本函数的 `hdr`，C 只写 `sizeof(struct PacketHdr)`。
    // - 对齐：局部 `PacketHdr` 按 `repr(C)` 对齐放置。
    // - 别名：`p` 派生期间没有同时存在的引用；随后的字段读发生在 C 返回之后。
    // - provenance：来自 `hdr` 这一次分配，未做偏移。
    // - 生命周期：`hdr` 活过 C 调用；指针不逃出本函数。
    unsafe { m6_c_fill_hdr(p) };
    hdr
}

/// C 在自己的栈上构造 `PacketHdr`，再调 `m6_rust_hdr_sum`。
#[must_use]
pub fn c_to_rust_hdr_sum() -> i32 {
    // SAFETY: 无指针从 Rust 传入。C 侧构造局部结构体再回调。
    // 有效性：由 C 保证传入非空、指向自己的栈对象。
    // 对齐/别名/provenance/生命周期：见 `m6_rust_hdr_sum` 的 SAFETY（被调方）。
    unsafe { m6_c_call_rust_hdr_sum() }
}

/// C `malloc` → Rust 读成 `CStr` → Rust `free`。
///
/// 约定：**C 分配，Rust 释放**。Rust 不得再让 C `free` 一次，也不得漏掉 `free`。
#[must_use]
pub fn take_c_message() -> String {
    // SAFETY:
    // - 有效性：成功时指向 `malloc` 出来、已写入 NUL 结尾字节的一块；失败为 null。
    // - 对齐：`malloc` 满足任意标量对齐；`c_char` 对齐为 1。
    // - 别名：这块内存此刻只有返回值这一份指针；C 不再持有。
    // - provenance：来自这一次 `malloc`，未做越界偏移。
    // - 生命周期：所有权按约定移交给 Rust，直到下面的 `free`。
    let p = unsafe { m6_c_alloc_message() };
    assert!(!p.is_null(), "C malloc returned null");
    // SAFETY:
    // - 有效性：`p` 非空且指向 NUL 结尾的 C 字符串（`roundtrip.c` 写入 `OWNED_MSG` + NUL）。
    // - 对齐：`c_char` 对齐 1。
    // - 别名：只读；没有并行的可变指针。
    // - provenance：仍是上面那次 `malloc`。
    // - 生命周期：借用只活在本语句，随后立刻拷进 `String`，再 `free`。
    let owned = unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned();
    // SAFETY:
    // - 有效性：`p` 仍是尚未释放的 `malloc` 指针。
    // - 对齐：`free` 接受任意 `malloc` 指针。
    // - 别名：`owned` 已拷走内容；此后不再通过 `p` 读。
    // - provenance：与 `malloc` 配对；`libc::free` 对应 C 的 `free`。
    // - 生命周期：释放后 `p` 作废，本函数不再使用它。
    unsafe { libc::free(p.cast()) };
    owned
}

/// 打开路径。失败时**立刻**捕获 `raw_os_error`，避免中间再调 libc 冲掉 errno。
pub fn open_path(path: &CStr, flags: c_int) -> Result<c_int, ErrnoError> {
    // SAFETY:
    // - 有效性：`path` 是合法 `CStr`，指针指向 NUL 结尾字节。
    // - 对齐：`c_char` 对齐 1。
    // - 别名：内核只读路径；本调用期间没有通过别的指针改这些字节。
    // - provenance：来自 `CStr` 的内部缓冲。
    // - 生命周期：`path` 的借用活过 `open`。
    let fd = unsafe { libc::open(path.as_ptr(), flags) };
    if fd < 0 {
        return Err(capture_errno());
    }
    Ok(fd)
}

pub fn close_fd(fd: c_int) -> Result<(), ErrnoError> {
    // SAFETY:
    // - 有效性：`fd` 是调用方提供的描述符；非法值由内核返回错误，不是 Rust UB。
    // - 对齐 / 别名 / provenance / 生命周期：不适用，原因是没有用户空间指针。
    let rc = unsafe { libc::close(fd) };
    if rc != 0 {
        return Err(capture_errno());
    }
    Ok(())
}

fn capture_errno() -> ErrnoError {
    let code = io::Error::last_os_error()
        .raw_os_error()
        .expect("Linux libc errno always has a raw OS code");
    ErrnoError { code }
}
