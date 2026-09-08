# Module 5 源码引用

**Story**: US5 | **Capabilities**: C-15…C-20 | **依据**: FR-005 / SC-003 /
[learning-artifact-contract §B](../../specs/001-rust-foundation/contracts/learning-artifact-contract.md)

路径根：`$(rustc --print sysroot)/lib/rustlib/src/rust/library/`
本机为 `/root/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/`

行号在 pinned 工具链 **1.98.0**（`88d9e12ae 2026-08-18`）下固定，因此可被记录并复核（规则 B1）。

---

## 引用表

| C-ID | 路径（相对 `library/`） | 符号 | 行 | kind | 这段源码回答了什么 |
|------|----------------------|------|----|------|------------------|
| C-15 | `core/src/slice/mod.rs` | `get_unchecked` Safety 段 | 612–619 | library | 越界即使结果不用也是 UB |
| C-15 | `core/src/slice/mod.rs` | `pub const unsafe fn get_unchecked` | 640 | library | 不检查下标的入口；体内再 `unsafe` 转给 `SliceIndex` |
| C-15 | Rust Reference §"Behavior considered undefined" | UB 定义 | — | reference-fallback | 语言把哪些行为叫未定义；崩溃既不充分也不必要 |
| C-16 | `core/src/ptr/mod.rs` | `pub const unsafe fn read` | 1692 | library | typed 读：有效性/对齐/初始化在调用方 |
| C-16 | `core/src/ptr/mod.rs` | `pub const unsafe fn write` | 1916 | library | typed 写：不 drop 旧值 |
| C-16 | `core/src/ptr/mod.rs` | `pub macro addr_of` | 2729 | library | 取址不创建引用（软弃用，等价 `&raw const`） |
| C-16 | `core/src/ptr/const_ptr.rs` | `pub const unsafe fn read` | 1148 | library | 方法版与自由函数是同一组要求 |
| C-17 | `core/src/ptr/const_ptr.rs` | `pub const unsafe fn offset` | 354 | library | 有符号偏移；越出分配是 UB |
| C-17 | `core/src/ptr/const_ptr.rs` | `pub const unsafe fn add` | 838 | library | 无符号加法；越出分配立刻犯规 |
| C-17 | `core/src/ptr/const_ptr.rs` | `pub const fn wrapping_add` | 1035 | library | 本身永远安全；解引用仍受分配约束 |
| C-17 | `core/src/ptr/const_ptr.rs` | `wrapping_add` Safety："always safe" | 985–988 | library | 和 `add` 的分工写在这里 |
| C-18 | `core/src/mem/mod.rs` | `pub const fn align_of` | 540 | library | 返回类型的 ABI 对齐要求，不是某块内存的实测 |
| C-18 | `core/src/ptr/mod.rs` | `pub const unsafe fn read_unaligned` | 1810 | library | 卸掉对齐义务，有效性还在 |
| C-19 | `core/src/cell.rs` | `pub struct UnsafeCell` | 2323 | library | 内部可变性的根；`repr(transparent)` |
| C-19 | `core/src/cell.rs` | `impl !Sync for UnsafeCell` | 2328 | library | 默认禁止共享；要共享得自己 `unsafe impl` |
| C-19 | `core/src/cell.rs` | `pub const fn get` | 2443 | library | **安全**函数，只交出 `*mut T` |
| C-20 | `core/src/slice/raw.rs` | `pub const unsafe fn from_raw_parts` | 124 | library | 裸指针 + 长度拼切片；责任全在调用方 |
| C-20 | `alloc/src/vec/mod.rs` | `pub const unsafe fn set_len` | 2224 | library | 只改长度，不初始化新露出来的槽 |

### 复核命令

```bash
SRC="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
sed -n '612,648p' "$SRC/core/src/slice/mod.rs"
sed -n '1692,1722p;1810,1830p;1916,1920p;2729,2735p' "$SRC/core/src/ptr/mod.rs"
sed -n '354,357p;838,841p;985,1040p;1148,1152p' "$SRC/core/src/ptr/const_ptr.rs"
sed -n '540,542p' "$SRC/core/src/mem/mod.rs"
sed -n '2323,2328p;2443,2448p' "$SRC/core/src/cell.rs"
sed -n '124,139p' "$SRC/core/src/slice/raw.rs"
sed -n '2224,2232p' "$SRC/alloc/src/vec/mod.rs"
```

---

## 读这些源码时最值得注意的三件事

### 1. `get_unchecked` 的 UB 在调用那一刻，不在使用结果那一刻

`:614-619` 写明 `get_unchecked(len)` 即使立刻丢掉引用也已经犯规。
所以 C-15 对照侧不必"用"那个引用去撞页错误 —— 调用本身就是实验。

### 2. `wrapping_add` 和 `add` 的分工写在 Safety 第一句

`:987`：`wrapping_add` 本身永远安全。`:838` 的 `add` 没有这句话。
"算出越界指针算不算"这个问题，源码已经替你划开了。

### 3. `UnsafeCell::get` 不是 `unsafe fn`

`:2443` 是普通 `pub const fn`。内部可变性的根并不在"交出指针"这一步放炸弹，
炸弹在你叠两份 `&mut` 的时候。C-19 对照实验造的是后一步。

---

## reference-fallback 理由说明

| C-ID | 为何 fallback | 用了哪份参考 |
|------|--------------|-------------|
| C-15（UB 定义） | 已知三项之一（契约 §B2）。未定义行为是语言规则，没有一个 `fn` 能"实现"它 | Rust Reference §Behavior considered undefined |

其余 C-16…C-20 全部是 `kind = library`，未使用额外 fallback。
