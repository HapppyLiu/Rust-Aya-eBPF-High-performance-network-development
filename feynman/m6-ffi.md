# Feynman: Module 6 — FFI 与 Rust/C/Linux 交界面

**Capabilities covered**: C-21

---

## 1. 用自己的话解释

面向"没学过 Rust 但懂 C"的同事。

你已经知道：两份目标文件要交换结构体，靠的不是字段名字长得像，而是两边的编译器
按同一份 ABI 把字段排成同一张图。C 那边这份图写在 ABI 手册里。
Rust 这边默认**不**承诺跟那份图走 —— 字段可以重排、填充可以变。
那叫语言自己的内部表示。要跟 C 对接，得明确说"请按 C 的规则排"，
然后**两边都量** `sizeof`、对齐、每个字段的偏移。只在一边贴了标签、从没问过
对面的编译器，那不叫证明，那叫假设。

函数也是两边的事。Rust 调 C：声明"这个符号按 C 的方式进来"，调用仍是你要保证
指针还活着的那种调用。C 调 Rust：Rust 把符号按 C 的名字导出，C 再声明再调用。
两个方向是两条契约，一条过了不能顶替另一条。

内存还是谁 `malloc` 谁 `free`。跨语言之后这句话仍然成立，只是两半写在不同文件里。
两边都 free 是重复释放；谁都不 free 是泄漏。进程返回 0 盖不住泄漏。

内核把失败写成一个整数。Rust 喜欢用带标签的错误类型包一层。包可以加一句人话，
但不许把那个整数弄丢。人话会随系统语言变；整数不会。
还得马上读 —— 下一次 C 库调用就会把这个整数冲掉。

最后：本模块的检查器看见的是机器上的越界和释放错误，不是 Rust 语言里所有
"没意义的操作"。检查器没说话，只能说它没看见那些内存错误，不能说"所以没有问题"。

---

## 2. 最小示例

### C-21 FFI —— 按 C 规则排的头 + 一次跨边界调用

```rust
#[repr(C)]
struct PacketHdr { kind: u8, id: u32, len: u16 }

unsafe extern "C" { fn m6_c_add(a: i32, b: i32) -> i32; }

assert_eq!(size_of::<PacketHdr>(), /* 与 C sizeof 比对，不口算 */);
assert_eq!(unsafe { m6_c_add(6, 7) }, 13);
```

完整观察：
[`examples/c21_ffi_layout.rs`](../experiments/m6-ffi/examples/c21_ffi_layout.rs)
[`examples/c21_ffi.rs`](../experiments/m6-ffi/examples/c21_ffi.rs)
[`examples/c21_ffi_ownership.rs`](../experiments/m6-ffi/examples/c21_ffi_ownership.rs)
[`examples/c21_errno.rs`](../experiments/m6-ffi/examples/c21_errno.rs)

---

## 3. 底层机制

每条论断带本模块的断言名或源码符号。

1. **默认布局不是 ABI。** `repr(Rust)` 不承诺字段顺序。要对齐 C，需要 `#[repr(C)]`
   **加上** 对 C 编译器排出数字的测量。
   依据：`tests/c21_ffi_layout.rs::repr_c_layout_matches_c_compiler`；
   OBSERVATIONS「C-21 / c21_ffi_layout」（本机 12/4/0/4/8，方法可推广，数字随 ABI 变）。

2. **`c_int` / `c_char` 跟目标走。** 本机 `c_int = i32`、`c_char = i8`，
   这是 `primitives.rs` 的分发结果，不是"Rust 认为 int 永远是 32 位"。
   依据：`source-refs.md` `core/src/ffi/mod.rs:36-38`、
   `primitives.rs:21/28/131-133/184`。

3. **`CStr` 的长度靠 NUL。** `from_ptr` 调 `strlen` 再拼切片。
   依据：`source-refs.md` `core/src/ffi/c_str.rs:254`；
   std 侧只是再导出（`std/src/ffi/c_str.rs:10`）。

4. **两个调用方向各自成立。**
   依据：`rust_to_c_add_returns_sum`（6+7=13）与
   `c_to_rust_mul_returns_product`（6×7=42）；
   结构体指针：`rust_to_c_struct_pointer_sum` / `c_to_rust_struct_pointer_sum`。

5. **所有权约定是配对，不是气氛。** C `malloc`，Rust `libc::free`。
   依据：`c_alloc_rust_free_roundtrips_literal`；
   拆掉时的形态见 OBSERVATIONS「约定不一致的故障形态」（书面，未执行故意 UB）。

6. **errno 留下的是整数。** 失败后立刻 `raw_os_error`。
   依据：`open_missing_preserves_enoent`、`raw_os_error_roundtrips_through_io_error`、
   `close_invalid_fd_preserves_ebadf`。

7. **ASan 无报告 ≠ 无 UB。**
   依据：OBSERVATIONS「C-21 / ASan」抄录的 T004 / CHK045 声明；
   `tools/run-asan.sh m6-ffi` 退出码 0。

---

## 4. 常见误区

- **误解**：写了 `#[repr(C)]` 布局就一定跟 C 一样
  → **实际**：那只是 Rust 半边的约束；没量过 C 的 `sizeof`/`offsetof`，一致仍是假设
  → **证据**：`repr_c_layout_matches_c_compiler`；OBSERVATIONS「C-21 / c21_ffi_layout」

- **误解**：一边调用能跑，双向就算过了
  → **实际**：SC-008 要 Rust→C 与 C→Rust 各自的判据
  → **证据**：`rust_to_c_add_returns_sum` 与 `c_to_rust_mul_returns_product` 是两条测试

- **误解**：跨边界谁 free 都行，反正进程会退出
  → **实际**：双释放是内存错误；泄漏在退出码 0 里看不见
  → **证据**：`c_alloc_rust_free_roundtrips_literal`；OBSERVATIONS「约定不一致的故障形态」

- **误解**：看 `Display` 就知道错误还在
  → **实际**：诊断全文随 locale 变；没丢掉的是 `ErrnoError.code`
  → **证据**：`raw_os_error_roundtrips_through_io_error`；OBSERVATIONS「C-21 / c21_errno」

- **误解**：ASan 干净 = 这段 FFI 没有未定义行为（把 Miri 的强度搬过来）
  → **实际**：ASan 看不见别名 / provenance / 未对齐这类语义问题
  → **证据**：OBSERVATIONS「C-21 / ASan」

---

## 5. 验证性问题

1. 只在 Rust 侧加了 `#[repr(C)]`，从未调用 C 侧 `m6_c_hdr_size` 这类导出，
   布局实验还算通过吗？
   **答**：不算。缺的是 C 编译器那一列数字。
   指向：`tests/c21_ffi_layout.rs::repr_c_layout_matches_c_compiler`

2. `c_add` 过了、没有 `c_to_rust_mul` 那条断言，SC-008 的双向过了吗？
   **答**：没有。两个方向是合取。
   指向：`tests/c21_ffi.rs::c_to_rust_mul_returns_product`

3. 约定是 C 分配、Rust 释放。Rust 里再调一次 C 的 `free`，ASan 预期看见什么？
   本模块为什么不把这件事做成稳定断言？
   **答**：double-free / use-after-free。Independent Test 要求 ASan 无报告，
   故意 UB 只写在观察记录里。
   指向：OBSERVATIONS「约定不一致的故障形态」

4. `Display` 在中文环境和英文环境字面不同。该断言哪一层才算原始信息未丢失？
   **答**：`ErrnoError.code` 与 `io::Error::from_raw_os_error` 的圆回，不是全文。
   指向：`tests/c21_errno.rs::raw_os_error_roundtrips_through_io_error`

5. `c_char` 在本机是有符号还是无符号？是语言对所有目标的保证吗？去哪核对？
   **答**：本机有符号（`i8`）。不是全目标保证；ARM 等走 `u8` 分支。
   指向：`source-refs.md` `core/src/ffi/primitives.rs:131-133`

6. 用户态后面要跟内核结构体、syscall 缓冲区打交道。本模块哪一条实验对应
   "先量两侧再相信布局"？
   **答**：布局实验，加上 concept 里对 Aya 用户态/`bpf_attr` 的关联说明。
   指向：`repr_c_layout_matches_c_compiler`；`learning/m6-ffi/concept.md`「与后续学习的关联」

---

## 检验结果

| # | 检验项 | 合格标准（可判定） | 状态 |
|---|-------|-----------------|------|
| 1 | 自述概念 | 第 1 节存在，且不含未解释的 Rust 术语 | pass |
| 2 | 最小示例 | 每个 covered capability 各有一段 ≤15 行代码且链接到 `examples/` | pass |
| 3 | 底层机制 | 第 3 节每条论断都带断言名或源码符号 | pass |
| 4 | 常见误区 | 每个 covered capability ≥1 条，且三段式齐备 | pass |
| 5 | 回答问题 | ≥5 个问题，且每题的回答指向一条断言 / 一处源码引用 / 一个观测块 | pass |

**五项是合取。** 全部 pass → 本模块 `feynman_status = passed`，C-21 可进入 `accepted`。
