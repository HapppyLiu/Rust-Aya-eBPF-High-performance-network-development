# Module 4 —— OBSERVATIONS

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

> 基线：[../../acceptance/environment-baseline.md](../../acceptance/environment-baseline.md)
> 本块字段与基线一致，无差异。

---

## 记录块

### C-12 / c12_send_sync  [NON-ASSERTION]

命令：`cargo run -p m4-concurrency --example c12_send_sync`

输出：

```text
=== 1. 移动 Cell：Send 成立，原线程不再持有 ===
  other thread saw 7

=== 2. 共享 Mutex<Cell>：锁把可移动升级成可共享 ===
  after scoped add = 13

=== 3. 共享裸 Cell 被拒绝（见 compile_fail/c12_cell_across_threads.rs） ===
  那是 Sync 一侧；本 example 不演示编译失败。
```

解释：
  为什么会这样：`Cell<u32>: Send`，把整个格子移到另一线程后原线程碰不到它，
  所以打印 7。`Mutex<Cell<u32>>: Sync`，因为锁的 `Sync` 条件是 `T: Send`，
  两个 scoped 线程可以轮流碰到同一个 `Cell`，加法发生一次，打印 13。
  这不能证明什么：打印 7 和 13 不能证明 `Cell` 自己变成了 `Sync`，
  也不能证明所有带内部可变性的类型都能共享。共享裸 `Cell` 根本不会出现在
  这段输出里 —— 那是编译失败，见下一块。

架构相关性：可跨架构推广。`Send`/`Sync` 是语言规则，与 endian / 字长无关。

---

### C-13 / c13_concurrency  [NON-ASSERTION]

命令：`cargo run -p m4-concurrency --example c13_concurrency`

输出：

```text
=== 1. scope + Mutex：总和确定，完成顺序不确定 ===
  sum = 100

=== 2. 各线程推入自己的编号（排列是 NON-ASSERTION） ===
  order (NON-ASSERTION) = [0, 1, 2, 3]
  id-set sum = 6
```

解释：
  为什么会这样：四个线程经同一把锁各加 25，互斥保证每次 `+=` 完整发生，
  所以总和是 100。编号集合是 `{0,1,2,3}`，元素和是 6。
  这一次打印的排列恰好是升序 —— 那是这一次调度的运气。
  这不能证明什么：`[0, 1, 2, 3]` **不能**证明互斥锁保证了完成顺序。
  换一次运行、换一台机器、换 Miri 的 seed，排列都可以变。
  稳定断言只钉总和与集合，见 `scoped_mutex_sum_is_order_independent`。

架构相关性：可跨架构推广。互斥与总和是语言 / 库语义；
排列本身被标成 NON-ASSERTION，不拿它做跨架构结论。

---

### C-14 / c14_atomic  [NON-ASSERTION]

命令：`cargo run -p m4-concurrency --example c14_atomic`

输出：

```text
=== 1. SeqCst 累加：终值确定 ===
  seqcst 4×25 = 100

=== 2. Release/Acquire 握手：看见旗标就看见载荷 ===
  acquire seen = 42

=== 3. 两边 Relaxed：语言允许旧载荷，本机可能仍打印 42 ===
  relaxed seen (NON-ASSERTION) = 42
  若这里是 42，只能说明这一次没观察到弱序窗口，不能推广到 aarch64。
```

解释：
  为什么会这样：`SeqCst` `fetch_add` 每次加 1 且有全序，4×25 必须是 100。
  Release 存储旗标对 Acquire 加载同步，看见 1 就蕴含看见之前的 `data = 42`。
  两边都改成 `Relaxed` 之后，语言规则允许"旗标已立、载荷仍是 0"。
  本机打印仍是 42，是因为 x86_64 的 TSO 对商店几乎按程序序提交，
  把这个窗口掩盖掉了。
  这不能证明什么：本机看见 42 **不能**证明 Relaxed 握手在 aarch64 / POWER
  上仍然正确，也不能证明"弱序等于强序"。它只证明这一次没观察到窗口。
  因此 Relaxed 的返回值不得进入 `#[test]`。

架构相关性：`SeqCst` 终值与 Release/Acquire 握手可跨架构推广。
Relaxed 这一次看见 42 **仅适用于 x86_64**，原因：TSO 掩盖了弱序窗口；
在弱序架构上同一源码可以看见 0。

---

## 编译器诊断抄录（compile_fail 样本）

### c12_cell_across_threads  [NON-ASSERTION]

期望错误码（样本首行 `//! EXPECT:` 声明）：E0277
实际错误码（`rf_harness` 提取）：E0277

```text
error[E0277]: `Cell<u32>` cannot be shared between threads safely
   --> compile_fail/c12_cell_across_threads.rs:12:21
    = help: the trait `Sync` is not implemented for `Cell<u32>`
    = note: required for `&Cell<u32>` to implement `Send`
note: required by a bound in `Scope::spawn`
    F: FnOnce() -> T + Send + 'scope
```

解释：
  为什么会这样：`Cell: !Sync`，因此 `&Cell: !Send`（`T: Sync ⟺ &T: Send`）。
  `scope.spawn` 仍要求闭包 `Send`。诊断直接说的是"不能共享"，不是"不能移动"。
  这不能证明什么：诊断措辞会随版本漂移，不能拿全文做相等比较。
  稳定断言只认错误码 E0277。它也不能证明 `thread::spawn`（`'static`）会报同一个码。

架构相关性：可跨架构推广。auto trait 是语言规则。

---

### c13_data_race  [NON-ASSERTION]

期望错误码（样本首行 `//! EXPECT:` 声明）：E0499
实际错误码（`rf_harness` 提取）：E0499

```text
error[E0499]: cannot borrow `hits` as mutable more than once at a time
    first mutable borrow occurs here  (第一个 spawn)
    second mutable borrow occurs here (第二个 spawn)
```

解释：
  为什么会这样：两个闭包都要对 `hits` 做 `+=`，各自需要 `&mut hits`，
  且借用活过整个 `'scope`。借用规则同一时刻至多一个可变借用。
  这就是安全 Rust 拦截数据竞争的位置 —— 别名规则，不是"线程 API 忘了加锁"。
  这不能证明什么：E0499 不能证明"加了锁就没有并发问题"。
  锁解决的是共享之后的互斥；本样本在共享之前就被借用规则拦住了。
  绕过它需要 `unsafe`，本模块不绕。

架构相关性：可跨架构推广。借用规则是语言规则。

---

### quiz_*.rs（题集负向）  [NON-ASSERTION]

9 个样本实际错误码均为 **E0277**（与 `//! EXPECT:` 一致），各由一个最弱字段触发：

| 样本 | 诊断对准的字段 |
|------|--------------|
| `quiz_slot_not_sync` | `Cell<u32>` cannot be shared |
| `quiz_shared_not_send` | `Rc<u32>` cannot be sent |
| `quiz_rawslot_not_send` | `*mut u8` cannot be sent |
| `quiz_sharedmut_not_send` | `RefCell<u32>` cannot be shared（`Arc` 要求 `T: Sync`） |
| `quiz_held_not_send` | `MutexGuard` cannot be sent |
| `quiz_marked_not_send` | `*const u8` cannot be sent |
| `quiz_callback_not_sync` | `dyn Fn() + Send` cannot be shared |
| `quiz_rawcell_not_sync` | `UnsafeCell<u32>` cannot be shared |
| `quiz_peek_not_send` | `Cell<u32>` cannot be shared（`&T: Send` 要求 `T: Sync`） |

解释：
  为什么会这样：auto trait 按最弱字段推导，或按容器写明的 bound 失败。
  这不能证明什么：诊断对准"字段类型"不能代替推导依据。
  题集要求写出"哪个字段、哪条规则"；只报 E0277 不算依据。

架构相关性：可跨架构推广。

---

## UB 判定记录

事前预测（`PREDICT-UB`）：本模块三个 example / 全部 `#[test]` 均为 **`clean`**。
无 expected-ub 对照实验（那是 US5 的成对实验）。

| 实验 | 事前预测（`PREDICT-UB`） | 工具与命令 | 实际类别 | `ub_verdict` | 命中? |
|------|----------------------|-----------|---------|-------------|-------|
| c12_send_sync | clean | `tools/run-miri.sh m4-concurrency --many-seeds` | 无 UB | **clean** | 是 |
| c13_concurrency | clean | 同上 | 无 UB | **clean** | 是 |
| c14_atomic | clean | 同上 | 无 UB | **clean** | 是 |

命令：`tools/run-miri.sh m4-concurrency --many-seeds`
别名模型：Stacked Borrows（默认）
seed 数：`-Zmiri-many-seeds` 默认 64 个 seed（日志出现 `Trying seed: 0` … `63`）
退出码：0（2026-09-08）
耗时：约 246 s

`compile_fail::expect_errors` 在 Miri 下跳过（harness 不支持子进程）。
编译失败是编译期事实，由普通 `cargo test` 覆盖，不影响 UB 判定。

未运行时本表 MUST 记 `n/a`，MUST NOT 记 `clean`（FR-019）。本次已运行。
