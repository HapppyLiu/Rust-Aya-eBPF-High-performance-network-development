# Module 2 —— OBSERVATIONS

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

### C-05 / c05_layout  [NON-ASSERTION]

命令：`cargo run -p m2-types --example c05_layout`

输出：

```text
=== 1. repr(C) 结构体：大小、对齐、字段偏移 ===
  PacketHeader { version: 1, kind: 6, length: 64 }
  size=8 align=4 offset(length)=4

=== 2. 同样的有效字段，换一个顺序：填充把尺寸拉开 ===
  LooseHeader size=6 align=2 offset(length)=2 offset(kind)=4
  —— 有效载荷都是 1+2+1 字节；多出来的是对齐空位，不是数据

=== 3. enum 状态机：判别式与变体载荷共享一块空间 ===
  ConnState size=8 align=4  (最大载荷 u32 的 size=4)
    Idle  tag=0
    Connecting { attempt: 3 }  tag=1
    Established { peer: 167772161 }  tag=2
    Closed { reason: 54 }  tag=3
  WireKind（无载荷）size=1  —— 只有判别式时，大小就是判别式本身

=== 4. Option 的宽度：有的 T 能省掉判别式，有的不能 ===
  size(Option<&u8>)=8  size(&u8)=8  同宽? true
  size(Option<u8>)=2  size(u8)=1  同宽? false
```

解释：
  为什么会这样：`repr(C)` 按声明顺序+对齐排字段，所以 `length` 落在偏移 4。
  `LooseHeader` 把 `u16` 插在两个 `u8` 之间，`u16` 必须落在 2 对齐上，前面空 1 字节，
  结构对齐为 2，末尾再补 1，总长 6。`ConnState` 的 8 = 判别式 1 + 填充 3 + 载荷 4；
  `WireKind` 没有载荷，只剩判别式。`Option<&u8>` 走文档化的 NPO，空指针编码 `None`；
  `u8` 值域已满，必须另加一个字节。
  这不能证明什么：这里的 `size=8` 是本机 `x86_64`、`u32` 对齐为 4 时的数字。
  跨架构可推广的是**关系**（`Option<&T>` 与 `&T` 同宽、`ConnState` 严格大于最大载荷），
  不是具体的 8。`Established` 打印出的 `167772161` 只是 `0x0a000001` 的十进制，
  不参与任何断言。

架构相关性：可跨架构推广（关系与 `repr(C)` 算法）；具体字节数仅适用于
`u32` 对齐为 4 的目标。本模块断言写的是关系 + `repr(C)` 的确定布局。

---

### C-06 / c06_trait  [NON-ASSERTION]

命令：`cargo run -p m2-types --example c06_trait`

输出：

```text
=== 1. 同一 trait，两条分发路径 ===
  static  ping=1 http=80
  dynamic ping=1 http=80

=== 2. 引用宽度：具体类型 vs trait 对象 ===
  size(&Ping)=8  size(&dyn Score)=16  size(usize)=8
  —— 多出来的那个 usize 是 vtable 指针，不是数据本身

=== 3. Display / Deref：语法糖落到哪份实现 ===
  Display => xdp-pass
  Deref   => len=8 starts_with(xdp)? true
```

解释：
  为什么会这样：静态与动态两条路径选中的是同一份 `impl Score for Ping/Http`，
  所以返回值相同。`&dyn Score` 在本 ABI 下是「数据指针 + vtable 指针」，
  宽度为 `2 * usize`。`Display` / `Deref` 是方法查找的入口，不创造第三种分发。
  这不能证明什么：返回值相等**不能**推出调用代价相等。
  `16` 这个数字等于 `2 * 8`，在 32 位目标上会变成 `2 * 4`；
  可断言的是倍数关系，不是 16。

架构相关性：可跨架构推广（胖指针 = 两个指针宽；分发结果与路径无关）。
`size=16` 仅适用于 64 位。

---

### C-07 / c07_generic  [NON-ASSERTION]

命令：`cargo run -p m2-types --example c07_generic`

输出：

```text
=== 1. 一份源码，三种具体类型 ===
  max_of<u8>  = 9
  max_of<u16> = 9
  max_of<u32> = 9

=== 2. 单态化的布局后果：Wrapper<T> 的大小随 T 变 ===
  size(Wrapper<u8>)=1  size(Wrapper<u64>)=8

=== 3. Iterator + PartialOrd bound ===
  count_gt([1, 5, 3, 9], 4) = 2

=== 4. 同一套比较：泛型 vs trait 对象 ===
  max_cmp        = 11
  max_cmp_dyn    = 11
```

解释：
  为什么会这样：三个 `max_of` 调用在编译期被复制成三份函数，各自按自己的宽度做比较，
  所以都能得到 9。`Wrapper<T>` 是 `repr(transparent)`，大小等于 `T`。
  `count_gt` 的过滤发生在 `Item: PartialOrd` 上。`max_cmp` 与 `max_cmp_dyn`
  跑的是同一套 `as_i64` 比较，所以都返回 11。
  这不能证明什么：打印出三个 9，不能单独证明"生成了三份机器码" ——
  那要看下面的 `print-mono-items` 观察。行为等价也不能推出代价等价。

架构相关性：可跨架构推广（单态化按具体类型复制；`Wrapper<T>` 大小等于 `T`）。

---

## 编译器诊断抄录（compile_fail 样本）

### c07_missing_bound  [NON-ASSERTION]

期望错误码（样本首行 `//! EXPECT:` 声明）：E0277
实际错误码（`rf_harness` 提取）：E0277

```text
error[E0277]: can't compare `T` with `T`
  --> experiments/m2-types/compile_fail/c07_missing_bound.rs:12:5
   |
12 |     needs_ord(a, b)
   |     ^^^^^^^^^ no implementation for `T < T` and `T > T`
   |
note: required by a bound in `needs_ord`
  --> experiments/m2-types/compile_fail/c07_missing_bound.rs:7:17
   |
 7 | fn needs_ord<T: PartialOrd>(a: T, b: T) -> bool {
   |                 ^^^^^^^^^^ required by this bound in `needs_ord`
help: consider restricting type parameter `T` with trait `PartialOrd`
```

解释：
  为什么会这样：`max_unbound<T>` 没有 `PartialOrd` bound，却调用了要求该 bound 的
  `needs_ord`。编译器在**候选选择**阶段失败，单态化尚未开始。
  插入符指向调用点，`note` 指向 bound 的定义处 —— 诊断把"谁要求、谁没给"画全了。
  这不能证明什么：E0277 是"缺 bound"的错误码，不是"运算符找不到"。
  若把 `a > b` 直接写在无 bound 的 `T` 上，同一条规则会先以 **E0369** 出现。
  样本刻意走函数调用路径，为的是让错误码对准任务要求的 E0277。

架构相关性：可跨架构推广。错误码是 rustc 的稳定契约，与指令集无关。

---

## UB 判定记录

本模块无 `unsafe` 代码。按 FR-019，`ub_verdict` 记 **`n/a`**，MUST NOT 记 `clean`。

| 实验 | 事前预测（`PREDICT-UB`） | 工具与命令 | 实际类别 | `ub_verdict` | 命中? |
|------|----------------------|-----------|---------|-------------|-------|
| c05_layout | n/a（无 unsafe） | 未运行 | — | n/a | — |
| c06_trait | n/a（无 unsafe） | 未运行 | — | n/a | — |
| c07_generic | n/a（无 unsafe） | 未运行 | — | n/a | — |

---

## IR 观察

### C-06：静态分发是直接调用，动态分发带 vtable 实参  [NON-ASSERTION]

命令：`tools/emit-llvm-ir.sh m2-types --example c06_trait`

摘录（`target/ir/m2-types-c06_trait.ll`，example 的 `main`）：

```text
; call m2_types::c06::score_static::<m2_types::c06::Ping>
%2 = call i32 @_RINv…score_static…Ping…(ptr %_3)

; call m2_types::c06::score_static::<m2_types::c06::Http>
%3 = call i32 @_RINv…score_static…Http…(ptr %_4)

; call m2_types::c06::score_dynamic
%8 = call i32 @_RNv…score_dynamic(ptr %_3, ptr align 8 @vtable.1)

; call m2_types::c06::score_dynamic
%9 = call i32 @_RNv…score_dynamic(ptr %_4, ptr align 8 @vtable.2)
```

同文件顶部的两张表：

```text
@vtable.1 = { …, ptr @<Ping as Score>::score, ptr @<Ping as Score>::label }
@vtable.2 = { …, ptr @<Http as Score>::score, ptr @<Http as Score>::label }
```

解释：
  为什么会这样：`score_static::<Ping>` 在单态化时已经选定 `Ping` 的 `score`，
  调用指令只有一个数据指针。`score_dynamic` 的签名在 IR 里是两个指针，
  第二个是对应类型的 vtable。`@vtable.1` / `@vtable.2` 把方法地址排成表，
  这就是"运行期才知道调谁"的全部运行时结构。
  这不能证明什么：debug IR 里的直接 `call` 仍可能被优化器内联；
  结构差异（1 个实参 vs 2 个实参、有无 vtable 全局量）才是分发方式的证据，
  不能用"有没有 `call` 指令"反推"有没有单态化"。

架构相关性：可跨架构推广。直接调用 vs 带 vtable 的间接调用是 rustc 的
分发模型，与指令集无关。vtable 里的指针宽度随 `usize` 变。

---

### C-06：`score_dynamic` 函数体是 load-from-vtable + 间接 call  [NON-ASSERTION]

命令：`tools/emit-llvm-ir.sh m2-types`（库 IR，`target/ir/m2-types.ll`）

摘录：

```text
define i32 @…score_dynamic(ptr %s.0, ptr align 8 %s.1) {
  ; s.0 = 数据指针, s.1 = vtable
  %1 = getelementptr inbounds i8, ptr %s.1, i64 24
  %2 = load ptr, ptr %1, align 8
  %_0 = call i32 %2(ptr %s.0)
  ret i32 %_0
}
```

解释：
  为什么会这样：vtable 前 24 字节是 rustc 放的元数据（drop / size / align），
  偏移 24 起才是 `Score::score` 的函数指针。`call i32 %2(...)` 的被调者是
  **寄存器里的指针**，不是一个具名函数 —— 这就是间接调用。
  对照 `score_static::<Ping>`：那里的被调者是具名的
  `@<Ping as Score>::score`。
  这不能证明什么：偏移 24 是本工具链、本 ABI 下 `Score` 这张表的布局，
  不是稳定契约，不能写成断言。能断言的是"动态路径的引用宽两个 `usize`"。

架构相关性：间接调用这一结构可跨架构推广；`i64 24` 这个偏移仅适用于
当前 rustc 为此 trait 生成的 vtable 布局，MUST NOT 当作稳定数字。

---

### C-07：`max_of` 对三个整数类型各有一份单态化实例  [NON-ASSERTION]

命令：`cargo +nightly rustc -p m2-types --example c07_generic -- -Z print-mono-items=yes`

摘录（只保留本模块写出的泛型函数）：

```text
MONO_ITEM fn m2_types::c07::count_gt::<[i32; 4]>
MONO_ITEM fn m2_types::c07::max_cmp::<i32>
MONO_ITEM fn m2_types::c07::max_of::<u16>
MONO_ITEM fn m2_types::c07::max_of::<u32>
MONO_ITEM fn m2_types::c07::max_of::<u8>
```

解释：
  为什么会这样：example 对 `u8` / `u16` / `u32` 各调用了一次 `max_of`，
  编译器按具体类型各复制一份。实例数是 **3**，不是 1。
  `count_gt::<[i32; 4]>` 与 `max_cmp::<i32>` 是另外两处单态化，
  同样是"一份源码、按调用点类型复制"。
  这不能证明什么：清单上还有大量标准库适配器实例（`Filter` / `Map` / `Sum`），
  那些不是你写的函数，不能算进"你的泛型被实例化了几次"。
  优化器稍后可能把三份 `max_of` 全部内联掉，最终 `.text` 里看不到三个符号；
  那不推翻"单态化发生过"，只说明内联是单态化之后的另一步。

架构相关性：可跨架构推广。单态化是 rustc 前端/中端的类型特化，与指令集无关。
