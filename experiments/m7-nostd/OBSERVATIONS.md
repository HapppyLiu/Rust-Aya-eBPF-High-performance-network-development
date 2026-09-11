# Module 7 —— OBSERVATIONS

本文件里的一切都是 **NON-ASSERTION**：它的差异不参与一致性判定（§C8.1）。
m7 的稳定断言载体是：`cargo build` 退出码、`const`/`compile_error!`、
`tools/check-nostd-artifact.sh`、`tools/m7-probe-errors.sh`、以及 host 侧
`#[test]`（`harness/tests/c24_bump_alloc.rs`）。

## 环境记录

| 字段 | 值 |
|------|-----|
| `rustc_stable` | 1.98.0 (88d9e12ae 2026-08-18) |
| `rustc_nightly` | 1.100.0-nightly (17fd5b8a3 2026-08-28) |
| `edition` | 2024 |
| `kernel` | 6.6.114.1-microsoft-standard-WSL2 |
| `arch` | x86_64 |
| `target` | x86_64-unknown-none |
| `command` | （按各记录块内的命令为准） |

> 基线：[../../acceptance/environment-baseline.md](../../acceptance/environment-baseline.md)
> 本块与基线的差异：`target` 为 `x86_64-unknown-none`（US7 强制，CHK018）。
> 工具链版本与基线一致，不是 FR-020 偏离。

---

## 记录块

### C-22 / 默认构建  [NON-ASSERTION]

命令：`cd experiments/m7-nostd && cargo build`

输出：`Finished dev profile ...`，退出码 0。

解释：
  为什么会这样：`.cargo/config.toml` 钉死 `x86_64-unknown-none`；`panic = "abort"`
  去掉对展开运行时的需求；`#[panic_handler]` 与 bump `#[global_allocator]` 补齐语言项；
  `extern crate alloc` 让 `Vec` 的类型出现。构建成功 = 这些入口都在。
  这不能证明什么：不能证明产物可以在本机执行，也不能证明 bump 在任意受限环境（含 eBPF）
  都能分配。裸机二进制没有操作系统可返回。

架构相关性：仅适用于 x86_64-unknown-none（以及同样无 std、默认 abort 的裸机三元组）。
指针宽度为 8 的 `const` 断言是 x86_64 的事实；方法（补齐语言项才能链接）可跨架构推广。

---

### C-22 / 产物静态检查  [NON-ASSERTION]

命令：`tools/check-nostd-artifact.sh`

输出（摘要）：

```text
PASS: 存在入口符号 _start
PASS: 无 __libc_start_main
PASS: 无 eh_personality
PASS: ELF64 / Machine = X86-64 / Type 为 EXEC 或 DYN（PIE）
退出码 0
```

`nm` 另可见 `rust_begin_unwind`（abort 路径的桩，**不是** `eh_personality`）。
`readelf -h`：Type = DYN（PIE），Entry = `_start` 的地址。

解释：
  为什么会这样：没有 libc，就不会有 `__libc_start_main`；panic=abort 不拉展开人格。
  rustc 仍会生成 `rust_begin_unwind` 作为 abort 的入口名字，这与 unwind 人格不是一回事。
  这不能证明什么：没有 libc 启动符号不能推出"这是 Linux 用户态程序"，也不能推出
  "所有 no_std 产物都是 PIE"。脚本断言的是**缺席/出席关系**，不是具体虚拟地址。

架构相关性：ELF64 + X86-64 头是本 target 的事实。无 `__libc_start_main`、无
`eh_personality` 这一对关系，在同样 abort 的裸机 target 上可推广。

---

### C-22 / 探测 1a 移除 panic_handler  [NON-ASSERTION]

命令：`tools/m7-probe-errors.sh 1a`
（`cargo build --features probe-no-panic-handler`）

输出：

```text
error: `#[panic_handler]` function required, but not found
```

错误码：无 `EXXXX`（这是语言项诊断，不是 E0xxx）。
归属：**panic / runtime**。计入 SC-006 分母第 1 条。

解释：
  为什么会这样：core 只声明 `panic_impl`，函数体必须由消费方用 `#[panic_handler]` 提供。
  拿掉它，rustc 在链接语言项时失败，不是 `libcore` 没编进 sysroot。
  这不能证明什么：不能写成"core 不能 panic"。core 里到处有 `panic!`；缺的是处理入口。

架构相关性：可跨架构推广。任何 `no_std` 且无 std panic 运行时的 target 都要 handler。

---

### C-22 / 探测 1b 强制 unwind  [NON-ASSERTION]

命令：`tools/m7-probe-errors.sh 1b`
（`RUSTFLAGS='-C panic=unwind' cargo build`，保留 handler）

输出：

```text
error: unwinding panics are not supported without std
```

错误码：无 `EXXXX`。
归属：**runtime**。计入 SC-006 分母第 2 条。

解释：
  为什么会这样：展开需要 personality 函数与 unwinder，这些由 std / panic_unwind 供给。
  本 target 默认 abort；强制 unwind 时 1.98.0 **不会**再单独报 `eh_personality` 缺失，
  而是直接拒绝"没有 std 还要 unwind"。spec SC-006 已按此修订，分母未改。
  这不能证明什么：不能证明"加上 eh_personality 空函数就能在裸机 unwind"。
  本诊断停在"不支持"，没有走到链接人格符号那一步。

架构相关性：诊断文本绑定 rustc 1.98.0 + 此 target。换 nightly / 自定义 JSON target
  可能重新索要 `eh_personality`。归属（展开运行时不属于 core）可推广。

---

### C-23 / 三层求和  [NON-ASSERTION]

命令：默认 `cargo build`（core 版 + alloc 版一并链接）；std 版只在探测 2a/2b。

输出：默认构建成功，因此 `sum_core` 与 `sum_alloc` 都链上。产物 `nm` 可见
`alloc::vec::Vec` 的 `forget`/`drop_glue` 符号。

解释：
  为什么会这样：core 版只碰切片；alloc 版需要 `Vec` 的类型（alloc crate）加上
  本 crate 的 bump 分配器。两者都不需要 std，也不需要文件/线程。
  这不能证明什么：不能证明 `Vec` 实现写在 std 里，也不能证明任意 no_std 环境都有堆。

架构相关性：可跨架构推广。分层是 crate 图的事实，不是 x86_64 特有。

---

### C-23 / 探测 2a core 中无 fs  [NON-ASSERTION]

命令：`tools/m7-probe-errors.sh 2a`

输出：

```text
error[E0433]: cannot find `fs` in `core`
  --> src/c23_core_alloc_std.rs:37:19
   |
37 |     let _f: core::fs::File;
```

错误码：`E0433`。
归属：**OS services**。计入 SC-006 分母第 3 条。

解释：
  为什么会这样：故意走 `core::fs`，避免被"找不到 crate std"抢走这条错误。
  `File` 打开的是内核对象，core 作为平台无关层根本没有 `fs` 模块。
  这不能证明什么：不能证明"core 以后会加 File"。缺的是操作系统，不是模块漏写。

架构相关性：可跨架构推广。`core::fs` 在任何目标上都不是模块。

---

### C-23 / 探测 2b 去找 std crate  [NON-ASSERTION]

命令：`tools/m7-probe-errors.sh 2b`

输出（完整错误集合，CHK020）：

```text
error[E0463]: can't find crate for `std`
  = note: the `x86_64-unknown-none` target may not support the standard library
  = note: `std` is required by `m7_nostd` because it does not declare `#![no_std]`
error: cannot resolve a prelude import
error[E0463]: can't find crate for `std`   (src/main.rs 对 std::env::args 的使用)
error: `#[panic_handler]` function required, but not found
```

错误码集合：`E0463`（两次）+ 无码的 prelude / panic_handler 诊断。
**计入分母的条目是 E0463**（未找到 crate std）。其余为级联。
归属：**OS services**（该 target 没有、也不能有完整 std，因为没有内核去实现它）。

解释：
  为什么会这样：去掉 `#![no_std]` 后 rustc 按默认注入 `std`。sysroot 里
  `x86_64-unknown-none` 只有 core/alloc/compiler_builtins，没有 std。
  没有 std 也就没有默认 panic handler，于是语言项诊断跟着出现。
  这不能证明什么：不能把四条全算进分母。rustc 可能只报首错或追加级联；
  SC-006 用固定清单，本条只计"crate std 不存在"。

架构相关性：仅适用于没有预编译 std 的 target。host `x86_64-unknown-linux-gnu`
  上去掉 `no_std` 会成功链接 std，这条探测将不再失败。

---

### C-24 / Vec 链接成功  [NON-ASSERTION]

命令：`cd experiments/m7-nostd && cargo build`

输出：成功。`c24_panic_alloc::vec_push_demo` 把两个 `u8` `push` 进 `Vec` 后 `forget`。

解释：
  为什么会这样：`extern crate alloc` 给出 `Vec` 类型；`#[global_allocator] static ALLOC: BumpAlloc`
  给出 `__rust_alloc` 的实现。分配能力由 **BumpAlloc** 提供。
  这不能证明什么：不能证明 alloc crate 自带堆，也不能证明同一份 bump 能搬进 eBPF。
  代码从未在裸机上**执行**，只是链接了 `push`。

架构相关性：可跨架构推广（"谁提供 GlobalAlloc 谁就提供堆"）。arena 大小与对齐填充随实现变。

---

### C-24 / 探测 3a 未引入 alloc crate  [NON-ASSERTION]

命令：`tools/m7-probe-errors.sh 3a`

输出：

```text
error[E0433]: cannot find module or crate `alloc` in this scope
  = help: add `extern crate alloc` to use the `alloc` crate
```

（同一行类型位置与表达式位置各一条，共 2 条 E0433。）
错误码：`E0433`。
归属：**alloc**。计入 SC-006 分母第 5 条。

解释：
  为什么会这样：`no_std` 不会自动 `extern crate alloc`。名字 `alloc::vec::Vec` 在作用域里
  不存在。这是缺 **alloc 这一层 crate**，还没有走到分配器。
  这不能证明什么：不能写成"Vec 在 std 里所以缺 std"。help 文本已经指向 `extern crate alloc`。

架构相关性：可跨架构推广。`no_std` + 未声明 alloc 在任何 target 上都是 E0433。

---

### C-24 / 探测 3b 无全局分配器  [NON-ASSERTION]

命令：`tools/m7-probe-errors.sh 3b`

输出：

```text
error: no global memory allocator found but one is required; link to std or add `#[global_allocator]` to a static item that implements the GlobalAlloc trait
```

错误码：无 `EXXXX`。
归属：**allocator**。计入 SC-006 分母第 6 条。
1.98.0 **没有**报 `alloc_error_handler` 缺失（OOM 默认 panic）。spec 已修订。

解释：
  为什么会这样：`Vec::push` 要调用 `__rust_alloc`。没有 `#[global_allocator]`、又没有 std
  的默认 malloc 跳板，语言项空缺。alloc crate 的算法在，堆的入口不在。
  这不能证明什么：不能证明"再写一个 `#[alloc_error_handler]` 就能过"。本工具链要的是
  全局分配器。也不能证明提供 bump 之后执行期一定不会 OOM。

架构相关性：可跨架构推广。纯 no_std 无全局分配器时诊断同类；具体措辞随 rustc 版本变。

---

### C-24 / host 侧 Miri  [NON-ASSERTION]

命令：`cargo +nightly miri test -p rf-harness --test c24_bump_alloc`

输出：3 passed（`bump_alloc_aligned_roundtrip` / `two_allocs_are_disjoint` /
`exhausted_arena_returns_null`）。

`ub_verdict`（Miri，host 侧 allocator）：**clean**
事前预期：clean

解释：
  为什么会这样：测试直接调用 `BumpAlloc::{alloc,dealloc}`，不经过裸机 `_start`。
  Miri 解释了 CAS bump、对齐、耗尽返回 null、以及 dealloc 空操作。未报告 UB。
  这不能证明什么：不能证明裸机产物在真实 CPU 上无 UB（那份二进制从未被执行，也未经 Miri）。
  范围仅限 host 上被测的这三个函数。未运行本命令时 MUST NOT 把 ub_verdict 写成 clean（FR-019）。

架构相关性：可跨架构推广。Miri 判定的是语言语义，不是 x86_64 总线行为。
