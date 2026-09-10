# Module 6 —— OBSERVATIONS

本文件里的一切都是 **NON-ASSERTION**：它的差异不参与一致性判定（§C8.1）。
稳定断言在 `tests/`，两者物理隔离（R-05）。

## 环境记录

| 字段 | 值 |
|------|-----|
| `rustc_stable` | 1.98.0 (88d9e12ae 2026-08-18) |
| `rustc_nightly` | 1.100.0-nightly (17fd5b8a3 2026-08-28) |
| `edition` | 2024 |
| `kernel` | 6.6.114.1-microsoft-standard-WSL2 |
| `arch` | x86_64 |
| `target` | x86_64-unknown-linux-gnu |
| `command` | （按各记录块内的命令为准） |
| `crates.libc` | 0.2.189（Cargo.lock） |
| `crates.cc` | 1.4.4（Cargo.lock） |
| `cc`（C 编译器） | cc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0 |

> 基线：[../../acceptance/environment-baseline.md](../../acceptance/environment-baseline.md)
> 本块字段与基线一致；额外的 `crates` / `cc` 是 data-model §9 对 m6 的强制项，不是基线偏离。

---

## 记录块

### C-21 / ASan  [NON-ASSERTION]

命令：`tools/run-asan.sh m6-ffi`

输出（摘要）：全部 13 个 integration test 通过；脚本打印
「退出码 0：ASan 未在本次运行中观测到内存错误。」

`ub_verdict`（ASan 覆盖面内）：**clean**
事前预期：clean

**判定强度声明（T004 / CHK045 / spec Assumptions，原文抄录）**

ASan 的覆盖面**窄于** Miri。它检测机器层可观测的内存错误（越界、UAF、double-free），
**不能**检测别名违规、provenance 违规、未对齐访问等 Rust 语义层 UB。
因此本脚本无报告时，结论 MUST 表述为「ASan 未在本次运行中观测到内存错误」，
MUST NOT 表述为「该 FFI 代码无 UB」。

C-21 因 Miri 无法执行真实 C 调用而改用 ASan（Plan R-02）。
「ASan 无报告」不等于「无 UB」。C-21 的验收强度**弱于** C-15…C-20。

架构相关性：可跨架构推广。ASan 的覆盖面声明是工具能力，不是本机硬件特性；
换架构仍不得把"无报告"读成"无 UB"。

---

### C-21 / c21_ffi_layout  [NON-ASSERTION]

命令：`cargo run -p m6-ffi --example c21_ffi_layout`

输出：

```text
=== PacketHdr layout (Rust vs C) ===
  size   rust=12 c=12
  align  rust=4 c=4
  off.kind rust=0 c=0
  off.id   rust=4 c=4
  off.len  rust=8 c=8
  match = true
```

解释：
  为什么会这样：`#[repr(C)]` 让 rustc 按 System V AMD64 排字段：
  `u8 kind` 占 1，随后 3 字节填充，`u32 id` 从偏移 4 开始，`u16 len` 从 8 开始，
  末尾再填 2 字节使 size 成为 align=4 的倍数，得到 12。
  C 侧 `sizeof` / `_Alignof` / `offsetof` 给出同一组数字。
  这不能证明什么：这组 **12/4/0/4/8** 是本 ABI 的数字，不能当成"所有架构都是 12"。
  能推广的是方法：两侧都量，数字相等才叫一致。只看见源码里写了 `repr(C)` 不算证明。

架构相关性：仅适用于 x86_64（以及与本机相同的 System V AMD64 / LP64 数据模型）。
`uint32_t` 对齐 4、尾部填充到 12，在 ILP32 或不同 ABI 上 size/align 会变；
稳定断言因此只比两侧是否相等，不断言具体字节数。

---

### C-21 / c21_ffi  [NON-ASSERTION]

命令：`cargo run -p m6-ffi --example c21_ffi`

输出：

```text
=== Rust → C ===
  c_add(6, 7) = 13
  c_hdr_sum(1,2,3) = 6
=== C → Rust ===
  rust_mul via C (6, 7) = 42
  rust_hdr_sum via C (1,2,3) = 6
=== C filled hdr ===
  kind=7 id=16909060 len=42
```

解释：
  为什么会这样：`6+7` 在 C 里算完；`6*7` 在 Rust 的 `m6_rust_mul` 里算完，经 C 包装返回。
  `id=16909060` 是 `0x01020304` 的十进制，C `m6_c_fill_hdr` 写入的值被 Rust 按同名字段读回。
  这不能证明什么：整数往返成功不能证明任意结构体按值传递都安全。本实验传的是指针。
  两个方向是两条契约：少写 C→Rust 那条，SC-008 的"双向"就不成立。

架构相关性：可跨架构推广。`int32_t` 加法/乘法的值语义与 endian 无关；
结构体按指针传递时，字段值能读回依赖于布局实验已经钉住的那份 ABI 契约。

---

### C-21 / c21_ffi_ownership  [NON-ASSERTION]

命令：`cargo run -p m6-ffi --example c21_ffi_ownership`

输出：

```text
=== C alloc / Rust free ===
  got = "m6-owned-by-rust"
  matches convention literal = true
  owner-to-free = Rust (libc::free pairs with C malloc)
```

解释：
  为什么会这样：C `malloc` + `memcpy` 写出含 NUL 的字面量；Rust `CStr::from_ptr` 读到
  第一个 NUL，拷进 `String`，再 `libc::free`。分配器配对：同一条 libc。
  这不能证明什么：一次成功的配对不能证明"忘了 free 也没事"。退出码 0 看不见泄漏。

架构相关性：可跨架构推广。`malloc`/`free` 配对是 C 抽象机规则，与指令集无关。
字面量本身是本实验选的，不是 ABI。

---

### C-21 / 约定不一致的故障形态  [NON-ASSERTION]（CHK044，书面产物，未执行）

命令：不执行故意双释放 / 故意泄漏。下面是约定被拆掉时的**预期**故障，供复核，不是测试输出。

| 约定被拆成 | 预期故障 | 谁能看见 |
|-----------|---------|---------|
| C `malloc` 之后 C 再 `free`，Rust 仍 `free` | double-free / heap-use-after-free | ASan 报告；普通运行可能 abort 或看似正常 |
| C `malloc` 之后谁都不 `free` | 泄漏 | 普通退出码 0 **看不见**；ASan+LSan 在检测开启时能看见 |
| Rust 用 `Vec::from_raw_parts` drop 一块 `malloc` 的内存 | 分配器不配对，释放路径 UB | ASan 可能报告 invalid-free；Miri 本场景不可用 |

解释：
  为什么会这样：跨语言之后"谁分配谁释放"仍然成立，只是两半写在不同源文件里。
  这不能证明什么：本块没有跑那些路径，所以它**不是** UB 判定。它是 US6 AS2 要求的
  可复核说明。真要观测双释放，应另开对照实验并接受 ASan 非 0 退出码；本模块的
  Independent Test 要求 ASan 无报告，因此对照只停留在书面。

架构相关性：可跨架构推广。double-free / 泄漏的分类是分配器契约，不是 x86_64 特性。

---

### C-21 / c21_errno  [NON-ASSERTION]

命令：`cargo run -p m6-ffi --example c21_errno`

输出：

```text
=== open missing path ===
  raw code = 2
  ENOENT   = 2
  preserved = true
  display (NON-ASSERTION) = No such file or directory (os error 2) (2)
=== close(-1) ===
  raw code = 9
  EBADF    = 9
```

解释：
  为什么会这样：内核给 `ENOENT=2`、`EBADF=9`。封装层把整数收进 `ErrnoError.code`，
  `Display` 再请 `io::Error` 加一句人话。人话会随 locale 变；整数不会。
  这不能证明什么：本机英文环境的那句 "No such file or directory" 不能当稳定断言。
  封装层假设"失败后立刻读 errno、中间不插其它 libc 调用"。若先 `printf` 再读，码可能已经变了。

架构相关性：可跨架构推广。POSIX `ENOENT`/`EBADF` 的编号在 Linux 上稳定为 2 和 9；
稳定断言绑的是 `libc::ENOENT` 常量而不是字面量 `2`，换架构若常量不同，断言仍比的是
"封装层保留了 libc 认为的那一个码"。
`Display` 全文仅适用于本 locale，已标 NON-ASSERTION。
