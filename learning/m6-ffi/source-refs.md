# Module 6 源码引用

**Story**: US6 | **Capabilities**: C-21 | **依据**: FR-005 / SC-003 /
[learning-artifact-contract §B](../../specs/001-rust-foundation/contracts/learning-artifact-contract.md)

路径根：`$(rustc --print sysroot)/lib/rustlib/src/rust/library/`
本机为 `/root/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/`

行号在 pinned 工具链 **1.98.0**（`88d9e12ae 2026-08-18`）下固定，因此可被记录并复核（规则 B1）。

---

## 引用表

| C-ID | 路径（相对 `library/`） | 符号 | 行 | kind | 这段源码回答了什么 |
|------|----------------------|------|----|------|------------------|
| C-21 | `core/src/ffi/mod.rs` | `pub use self::primitives::{c_char, …, c_int, …}` | 36–38 | library | C ABI 整数别名从哪再导出；FFI 代码应写 `c_int` 而不是赌 `i32` |
| C-21 | `core/src/ffi/primitives.rs` | `type_alias! { "c_char.md", c_char = … }` | 21 | library | `c_char` 是按目标选择的别名，不是 Rust `char` |
| C-21 | `core/src/ffi/primitives.rs` | `type_alias! { "c_int.md", c_int = … }` | 28 | library | `c_int` 跟当前目标的 C `int` 走 |
| C-21 | `core/src/ffi/primitives.rs` | `c_char_definition` 的默认分支 `type c_char = i8` | 131–133 | library | 本机（x86_64 Linux）C `char` 有符号；ARM 等目标走上面的 `u8` 分支 |
| C-21 | `core/src/ffi/primitives.rs` | `c_int_definition` 默认分支 `type c_int = i32` | 184 | library | 本机 `c_int = i32`；avr/msp430 才是 `i16` |
| C-21 | `std/src/ffi/c_str.rs` | `pub use core::ffi::c_str::CStr` | 10 | library | std 的 `CStr` 是 core 的再导出，实现不在 std |
| C-21 | `core/src/ffi/c_str.rs` | `pub struct CStr` 文档 | 18–23 | library | `&CStr` 是 NUL 结尾字节的借用视图，长度不另存 |
| C-21 | `core/src/ffi/c_str.rs` | `pub const unsafe fn from_ptr` | 254–266 | library | 从 `*const c_char` 做成 `CStr`：调用方保证有效、NUL 结尾；体内 `strlen` 再 `from_raw_parts` |

### 复核命令

```bash
SRC="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
sed -n '36,38p' "$SRC/core/src/ffi/mod.rs"
sed -n '21,28p;131,133p;177,185p' "$SRC/core/src/ffi/primitives.rs"
sed -n '1,14p' "$SRC/std/src/ffi/c_str.rs"
sed -n '18,23p;254,266p' "$SRC/core/src/ffi/c_str.rs"
```

---

## 读这些源码时最值得注意的三件事

### 1. `c_int` / `c_char` 是 ABI 别名，不是"Rust 里更好记的 i32 / char"

`mod.rs:36-38` 只是再导出。真正按目标分发的是 `primitives.rs` 的 `cfg_select!`。
本机走到 `c_char = i8`、`c_int = i32`。换到把 C `char` 定为 unsigned 的架构，
同一个名字会变成 `u8`。Rust 的 `char`（Unicode 标量）从未出现在这张表里。

### 2. std 的 `CStr` 文件几乎是空的

`std/src/ffi/c_str.rs:10` 一行 `pub use`。任务要求读这个文件，它回答的问题是
"用户代码写 `std::ffi::CStr` 时，实现落在哪一层"。答案：core。
Safety 段要去 `core/src/ffi/c_str.rs:254` 的 `from_ptr`。

### 3. `from_ptr` 用 `strlen` 算长度，不相信你另传一个 usize

`:257` 调 `strlen`，`:265` 再 `from_raw_parts(..., len + 1)`（含 NUL）。
所以传入的指针必须真的有 NUL，且 NUL 在 `isize::MAX` 之内 —— 这是调用方的义务，
和 C 程序员对 `strlen` 的义务是同一件事。

---

## reference-fallback 理由说明

C-21 全部是 `kind = library`，未使用 fallback。
`c_int` / `c_char` 的定义在 `primitives.rs` 而不在 `mod.rs` 的函数体里，
这不是"无库代码对应"，只是模块拆分；因此仍记 library，并同时引用 `mod.rs` 的再导出行。
