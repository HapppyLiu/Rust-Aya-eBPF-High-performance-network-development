# Module 7 源码引用

**Story**: US7 | **Capabilities**: C-22…C-24 | **依据**: FR-005 / SC-003 /
[learning-artifact-contract §B](../../specs/001-rust-foundation/contracts/learning-artifact-contract.md)

路径根：`$(rustc --print sysroot)/lib/rustlib/src/rust/library/`
本机为 `/root/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/`

行号在 pinned 工具链 **1.98.0**（`88d9e12ae 2026-08-18`）下固定，因此可被记录并复核（规则 B1）。

---

## 引用表

| C-ID | 路径（相对 `library/`） | 符号 | 行 | kind | 这段源码回答了什么 |
|------|----------------------|------|----|------|------------------|
| C-22 | `core/src/lib.rs` | crate 文档：core 不知道堆、不提供并发与 I/O | 12–14 | library | core 的边界：可移植原语，不是"缩小版 std" |
| C-22 | `core/src/lib.rs` | 消费方必须提供 panic handler | 37–39 | library | handler 是语言项，core 只规定签名、不提供函数体 |
| C-22 | `core/src/lib.rs` | `rust_eh_personality` / `eh_personality` | 41–44 | library | unwind 人格也是运行时约定；abort 策略下不会被调用 |
| C-22 | `core/src/lib.rs` | `#![no_core]` | 64 | library | core 不能写 `#![no_std]`：它比 std 更底，是 no_std crate 仍然依赖的那一层 |
| C-22 | `std/src/lib.rs` | `#![no_std]` + 注释 `Don't link to std. We are std.` | 236–237 | library | std **自己**也是 `no_std`。这直接推翻"`no_std` = 不能碰标准库" |
| C-23 | `alloc/src/lib.rs` | crate 文档：`no_std` crate 用 alloc 而不是 std | 6–9 | library | alloc 与 std 的分工：集合在这，OS 服务不在这 |
| C-23 | `alloc/src/lib.rs` | `#![no_std]` / `#![needs_allocator]` | 74–75 | library | alloc 自身不要 std，但**需要**一个分配器 |
| C-23 | `std/src/lib.rs` | `extern crate alloc as alloc_crate` | 464 | library | std 把 alloc 拉进来，不是自己实现 Vec |
| C-23 | `std/src/lib.rs` | `pub use alloc_crate::vec` | 607 | library | `std::vec::Vec` 是再导出，实现落在 alloc |
| C-24 | `core/src/panicking.rs` | core 不能定义 handler，只能调用 | 10–22 | library | `#[panic_handler]` 与 `panic_impl` 语言项的关系 |
| C-24 | `core/src/panicking.rs` | `#[lang = "panic_impl"]` | 67–69 | library | 所有 panic 漏斗进消费方提供的那个永不返回的函数 |
| C-24 | `core/src/alloc/global.rs` | `unsafe trait GlobalAlloc` / `alloc` | 144、178 | library | 分配入口的签名；失败返回空指针，size 0 是调用方 UB |
| C-24 | `alloc/src/alloc.rs` | `__rust_alloc` 由 `#[global_allocator]` 或 std 默认实现生成 | 12–22 | library | 没有全局分配器时，这些符号无人定义 |
| C-24 | `alloc/src/alloc.rs` | `handle_alloc_error`：无 std 时走 `panic!` | 530–532 | library | 1.98.0 不再强制 `#[alloc_error_handler]`；OOM 默认 panic |

### 复核命令

```bash
SRC="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
sed -n '12,14p;37,44p;64p' "$SRC/core/src/lib.rs"
sed -n '236,237p;464p;605,607p' "$SRC/std/src/lib.rs"
sed -n '1,9p;74,75p' "$SRC/alloc/src/lib.rs"
sed -n '10,22p;67,69p' "$SRC/core/src/panicking.rs"
sed -n '144,178p' "$SRC/core/src/alloc/global.rs"
sed -n '12,22p;523,532p' "$SRC/alloc/src/alloc.rs"
```

---

## 读这些源码时最值得注意的三件事

### 1. core 写的是 `no_core`，std 写的是 `no_std`

任务要求去 `core/src/lib.rs` 找 `#![no_std]`。1.98.0 上它不在那里 —— 换成了更底层的
`#![no_core]`。std 反倒在 `:237` 写着 `#![no_std]`，注释是"我们自己就是 std，别再链一份"。
这不是文档过时，这是分层：`no_std` 的 crate 仍然链接 core；core 不能再链接 core。

### 2. `needs_allocator` 不等于"我自带一块堆"

alloc 的集合会调用 `__rust_alloc`。谁实现它，谁才是堆。std 默认把实现接到 libc malloc；
裸机没有这份默认，必须 `#[global_allocator]`。

### 3. panic 处理是语言项，不是 core 里一段可以缺席的库函数

`panicking.rs` 用 `extern "Rust" { #[lang = "panic_impl"] fn panic_impl(...) -> !; }`
把责任推给消费方。拿掉 `#[panic_handler]` 时 rustc 报的是**语言项缺失**，
不是"core::panicking 没编进 sysroot"。

---

## reference-fallback 理由说明

C-22…C-24 全部是 `kind = library`，未使用 fallback。
core 顶部属性是 `no_core` 而非任务原文写的 `no_std`，这不是"无库代码对应"，
只是分层用词；因此仍记 library，并同时引用 std 的 `#![no_std]`。
