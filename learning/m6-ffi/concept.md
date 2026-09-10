# Module 6: FFI 与 Rust/C/Linux 交界面

**Story**: US6（P2） | **Capabilities**: C-21 | **Prerequisite**: m5（accepted）

> 本文件属于 **Answer Track**。对应的 Learner Track 是
> [`learner/m6-ffi/guide.md`](../../learner/m6-ffi/guide.md)。
> 如果你还没做过那边的预测表，先去做 —— 这份文件读过之后就不能再自测了。

## 这个模块回答什么问题

1. 为什么 Rust 结构体的默认布局不能拿去跟 C 结构体对字段？
2. 需要何种约束，两侧的 `size_of` / `align_of` / `offset_of` 才变成可核对的契约？
   只写了 `#[repr(C)]`、从未问过 C 编译器，算不算已经证明一致？
3. 一次跨边界的 `malloc`，谁负责 `free`？约定拆掉时会以泄漏还是重复释放出现？
4. 把 Linux `errno` 封装成 Rust 错误类型之后，原来的那个整数还在吗？封装层加进了哪些假设？

---

## 概念

### C-21 FFI

- **一句话定义**：FFI 是两套编译器按**同一份 ABI 契约**交换函数与内存的边界。
  契约没写进语言的那一半（默认布局、谁释放、何时读 errno）不能靠"这次碰巧能跑"补上。

- **底层机制**：

  **为何默认布局不可依赖（US6 AS1）**。
  `repr(Rust)` 是编译器的内部表示：字段顺序、填充、甚至是否重排，都**不是**对 C 的承诺。
  同名字段、同样的 Rust 类型，换一次 rustc 或加一个私有字段，图就可以变。
  所以不能把"我在 Rust 里按 C 的顺序声明了字段"当成 ABI。

  **何种约束才能保证两侧一致**。三件事合取，缺一不可：

  1. Rust 侧 `#[repr(C)]`，按目标平台的 C ABI 排字段（本机是 System V AMD64）；
  2. C 侧用相同的字段顺序与等宽类型（`uint8_t`/`uint32_t`/`uint16_t` 对 `u8`/`u32`/`u16`）；
  3. **两侧都量一遍**：Rust 用 `size_of` / `align_of` / `offset_of!`，C 用
     `sizeof` / `_Alignof` / `offsetof`，由导出函数把数字带回 Rust 再比对。

  第 3 步是 CHK043 的核心：声明 `#[repr(C)]` 只是契约的半边。
  `repr_c_layout_matches_c_compiler` 核对的是两侧编译器排出的图，不是源码里出现了那个属性。
  本机观察：`PacketHdr` size=12、align=4、off=(kind:0, id:4, len:8)，双侧相同。
  `kind` 后面有 3 字节填充，因为 `id: u32` 要对齐到 4 —— 这是 C ABI，不是 rustc 的口味。

  **双向调用（CHK042）**。
  Rust→C：`unsafe extern "C" { fn m6_c_add(...) }`，调用点仍是 `unsafe`。
  C→Rust：`#[unsafe(no_mangle)] pub extern "C" fn m6_rust_mul`，C 再包一层
  `m6_c_call_rust_mul` 把积带回。两个方向各有断言
  （`rust_to_c_add_returns_sum` / `c_to_rust_mul_returns_product`），
  不能合成一句"FFI 能跑"。结构体按**指针**传递，避开按值传递的寄存器分类细节。

  **跨边界所有权（US6 AS2 / CHK044）**。
  约定写死：**C `malloc`，Rust `libc::free`**。`take_c_message` 读成 `CStr` 拷进
  `String` 之后立刻 `free`。`c_alloc_rust_free_roundtrips_literal` 断言内容等于
  `OWNED_MSG`。约定不一致的故障形态（可复核，不作为稳定断言去跑）：

  | 约定被拆成 | 故障形态 |
  |-----------|---------|
  | 两边都 `free` | double-free / use-after-free；ASan 能看见 |
  | 谁都不 `free` | 泄漏；ASan+LSan 在检测开启时能看见，普通退出码 0 看不见 |
  | Rust `Vec::from_raw_parts` 去丢一块 `malloc` 的内存 | 分配器不配对，释放路径是 UB |

  **errno 封装（US6 AS3）**。
  `libc::open` 失败返回 -1，真正的码在线程局部的 `errno`。
  `open_path` **立刻**走 `io::Error::last_os_error().raw_os_error()`，收进
  `ErrnoError { code }`。`open_missing_preserves_enoent` 断言 `code == ENOENT`；
  `raw_os_error_roundtrips_through_io_error` 断言这个整数能圆回来。
  `Display` 全文（"No such file or directory"）是 NON-ASSERTION，随 locale 变。

  封装层引入的假设（写进产物，避免假装透明）：
  - Linux 上 `errno` 是线程局部的，但**不是** Rust 的 `Result` —— 下一次 libc 调用就会冲掉它；
  - 因此必须在失败返回之后、任何其它 FFI 之前捕获；
  - 路径是 `CStr`（内部不许有 NUL）；本实验不处理非 UTF-8 路径的显示问题；
  - `std::io::Error` 在 Linux 上保留 `raw_os_error`，这是 std 的承诺，不是内核的。

  **`c_int` / `c_char` / `CStr`**。
  `core/src/ffi/mod.rs` 再导出 `primitives.rs` 里的别名：它们跟**当前目标的 C ABI**走，
  不是"永远等于 `i32` / Rust `char`"。本机 `c_int = i32`，`c_char = i8`（x86_64 Linux
  上 C `char` 默认有符号）。Rust 的 `char` 是 Unicode 标量，和 C `char` 不是一类东西。
  `CStr` 在 `std/src/ffi/c_str.rs` 从 `core` 再导出；真正的 `from_ptr` 在
  `core/src/ffi/c_str.rs`，长度靠扫到 NUL，不在旁边另存一个 `usize`。

  **UB 判定用 ASan，不用 Miri（R-02 / CHK045）**。
  Miri 不能执行真实 C 调用。ASan 看见的是机器层内存错误（越界、UAF、double-free），
  **看不见**别名、provenance、未对齐这类 Rust 语义 UB。
  因此 `tools/run-asan.sh m6-ffi` 退出码 0 的含义是
  **"ASan 未在本次运行中观测到内存错误"**，MUST NOT 写成"该 FFI 代码无 UB"。
  判定强度弱于 US5 的 Miri。该假设抄在 OBSERVATIONS 判定说明里。

- **常见误解**：
  - **误解**："两边字段名一样、又写了 `repr(C)`，布局就一定一致"
    → **实际**：属性只约束 Rust 半边；没量过 C 编译器排出的数字，一致仍是假设。
    → **证据**：`repr_c_layout_matches_c_compiler`；OBSERVATIONS「C-21 / c21_ffi_layout」。
  - **误解**："跨边界的内存反正有 GC / 反正进程退出就回收，谁 free 无所谓"
    → **实际**：重复释放是 UB；不释放是泄漏。退出码 0 盖不住。
    → **证据**：`c_alloc_rust_free_roundtrips_literal`；OBSERVATIONS「约定不一致的故障形态」。
  - **误解**："封装成 `io::Error` 之后看 `Display` 就行，整数码不重要"
    → **实际**：诊断全文不稳定；没丢掉的是 `raw_os_error` 那个整数。
    → **证据**：`open_missing_preserves_enoent` / `raw_os_error_roundtrips_through_io_error`。
  - **误解**："ASan 没报告 = 没有 UB"（把 US5 的 Miri 强度复用过来）
    → **实际**：ASan 覆盖面窄于 Miri；无报告只说明它没看见内存错误。
    → **证据**：spec Assumptions / T004；OBSERVATIONS「C-21 / ASan」。

- **对应实验**：
  [`c21_ffi_layout`](../../experiments/m6-ffi/examples/c21_ffi_layout.rs)
  / [`c21_ffi`](../../experiments/m6-ffi/examples/c21_ffi.rs)
  / [`c21_ffi_ownership`](../../experiments/m6-ffi/examples/c21_ffi_ownership.rs)
  / [`c21_errno`](../../experiments/m6-ffi/examples/c21_errno.rs)
  / [tests](../../experiments/m6-ffi/tests/)

---

## 与后续学习的关联

FR-014 要求每项能力说明与后续 Linux / eBPF / Aya 的关联点，或显式标注为仅是基础。
本 Feature MUST NOT 编写 eBPF / Aya 程序（FR-017）。

| C-ID | 关联点 |
|------|-------|
| **C-21 FFI** | **直接对应 Aya 用户态与 Linux 的交界面**，不是"仅为理解基础"。 |
| | Aya 的用户态 loader、map 操作、syscall 包装，走的就是 `extern "C"` +
  C 结构体（`union bpf_attr`、各类 `*_attr`）+ 内核返回的负 errno。 |
| | 本模块三条落点，对应后续读那些绑定时应做的三件事： |
| | **(i) 布局**：先量两侧再相信字段偏移。`repr_c_layout_matches_c_compiler`
  是这个习惯的最小形态。bindgen 可以生成声明，但不能替代"我核对过 ABI"。 |
| | **(ii) 所有权**：map value、probe 读回来的缓冲，哪一侧 `free` / 哪一侧
  在 `drop` 时关 fd，必须写进约定。拆掉就是泄漏或 double-close。 |
| | **(iii) 错误码**：syscall 失败是整数。封装成 Rust `Error` 之后，
  `raw_os_error` 必须还在，否则用户态日志只剩一句空话，内核侧对不上。 |
| | 完整的 skb / NAPI / XDP / Aya 程序属 Feature 002+。本 Feature 只建立
  最小内核接触点（Constitution VIII：`open`/`close`/`errno`）。 |
