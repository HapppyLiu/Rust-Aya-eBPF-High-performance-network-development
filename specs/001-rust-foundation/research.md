# Phase 0 Research: Rust Foundation

**Feature**: 001-rust-foundation | **Date**: 2026-09-04 | **Plan**: [plan.md](./plan.md)

本文件解决 Spec 显式推迟到 Plan 阶段的全部决策（FR-016、FR-019、FR-020，以及 Assumptions 中
"具体工具链版本号、UB 检测工具、实验组织形式与所用 crate 由 Plan 阶段确定"）。

所有决策均在本机实际验证通过，验证命令与输出摘要记录在每条决策的 **Verification** 中。

---

## R-01: Rust 工具链版本锁定

**Decision**：以 **stable 1.98.0（commit-hash `88d9e12ae178fab0fb5cc050a94da85685d449ea`,
commit-date 2026-08-18, LLVM 22.1.8）** 作为本 Feature 唯一的**构建与验收工具链**，通过仓库根目录
`rust-toolchain.toml` 锁定。Edition 固定为 **2024**。

同时锁定 **nightly 1.100.0-nightly（commit-hash `17fd5b8a37b6667b6cc137f3cc35f09759768a3b`,
commit-date 2026-08-28, LLVM 23.1.0）** 作为**分析工具链**，其唯一用途是运行 stable 上不存在的
诊断能力：Miri、`-Z unpretty=mir`、`-Z sanitizer`。分析工具链 **MUST NOT** 用于产出任何稳定断言，
也 MUST NOT 用于编译交付实验的构建产物。

**Rationale**：
- FR-020 要求全程锁定单一工具链版本，其立法意图是避免"编译器诊断/中间表示输出变化导致既有验收记录
  失效"。把 stable 1.98.0 定为唯一验收工具链完整满足该意图：所有稳定断言、所有编译器诊断预测
  （US1、US4 的核心验收方式）都只在这一个版本上判定。
- 但 FR-019 强制要求以 UB 检测工具输出作为 unsafe 实验的判定依据，而 Miri **仅作为 nightly 组件
  发布**，stable 无法安装（已验证）。二者不可能同时用单一 stable 版本满足。
- 选择"stable 为准 + nightly 仅作分析"而非"统一用 nightly"，是因为 nightly 会静默允许 unstable
  特性。本 Feature 的目标是建立**稳定 Rust 语义**的心智模型，学习者在 nightly 上误用 unstable
  特性而不自知，会污染整个知识基线。
- 该偏离已在 plan.md 的 Complexity Tracking 中显式登记（Constitution Governance 要求）。

**Alternatives considered**：
- *统一 pin nightly，单一版本*：字面满足 FR-020，但引入 unstable 特性静默可用的风险；且 nightly
  的诊断措辞变动频率高于 stable，反而削弱 FR-020 的立法意图。已拒绝。
- *放弃 Miri，改用 Valgrind/ASan 作为唯一 UB 检测*：本机无 Valgrind；且 ASan/Valgrind 只能发现
  **表现出来**的内存错误，无法发现 Rust 语义层 UB（如别名违规、provenance 违规、未初始化读取）。
  US5 的核心恰恰是这类"程序照常运行但已是 UB"的场景。已拒绝。
- *不锁定，跟随 rustup update*：直接违反 FR-020。已拒绝。

**Verification**：
```
$ rustc -Vv
rustc 1.98.0 (88d9e12ae 2026-08-18)   commit-hash: 88d9e12ae178fab0fb5cc050a94da85685d449ea
$ rustup run nightly rustc -Vv
rustc 1.100.0-nightly (17fd5b8a3 2026-08-28)  commit-hash: 17fd5b8a37b6667b6cc137f3cc35f09759768a3b
$ rustup component list --toolchain stable | grep miri   # → 无输出（stable 不提供）
```

**Operational rule**：本 Feature 进行期间 MUST NOT 执行 `rustup update`。若不可避免地升级，
按 FR-020 重新验证受影响实验的验收记录。

---

## R-02: UB 检测工具选型（FR-019）

**Decision**：采用**分层 UB 检测**，按实验类型选择判定工具：

| 实验类型 | 判定工具 | 命令 |
|---------|---------|------|
| 纯 Rust unsafe（US5 主体、US4 并发） | **Miri**（主判定） | `cargo +nightly miri run --example <name>` / `cargo +nightly miri test` |
| 并发数据竞争 | Miri + `-Zmiri-many-seeds` | `MIRIFLAGS="-Zmiri-many-seeds" cargo +nightly miri test` |
| 别名模型对照 | Miri Stacked / Tree Borrows | `MIRIFLAGS="-Zmiri-tree-borrows"` |
| 跨 FFI 边界（US6） | **AddressSanitizer**（Miri 不可用时的替代判定） | `RUSTFLAGS="-Zsanitizer=address" cargo +nightly run -Zbuild-std --target x86_64-unknown-linux-gnu` |
| `no_std` 裸机产物（US7） | 编译期 + 静态检查（`nm` / `readelf` / `objdump`） | 见 quickstart.md |

**判定规则**（写入实验契约）：Miri 报告 `error: Undefined Behavior` 即判**实验失败**（若实验意图是
"演示 UB"，则该报告是**预期结果**，必须捕获其错误类别文本作为稳定断言）。程序正常退出、输出符合预期
MUST NOT 被记为"无 UB"。

**Rationale**：
- Miri 是唯一能检测 Rust **语义层** UB 的工具（别名/Stacked Borrows、provenance、未对齐、
  未初始化读取、越界），且错误信息直接指向源码行，教学价值最高。
- Miri 无法执行真实的 C 函数调用（`-Zmiri-native-lib` 为实验特性且需要额外配置），因此 US6 的
  FFI 实验必须有替代判定手段；ASan 覆盖了 FFI 边界上最典型的故障（越界、use-after-free、
  double-free、所有权移交约定不一致）。
- 分层而非单一工具，是因为没有任何单一工具覆盖 US5（语义 UB）+ US6（跨语言）+ US7（裸机）三种环境。

**Alternatives considered**：
- *仅用 Miri*：US6 的 FFI 实验将无判定依据，直接违反 FR-019。已拒绝。
- *仅用 ASan/UBSan*：无法检测别名与 provenance 违规，US5 AS2（未对齐访问为何是 UB 而不只是
  "某些架构会崩溃"）在 x86_64 上根本无法被 ASan 观测到——x86_64 硬件容忍未对齐访问，程序会正常输出。
  已拒绝。
- *Valgrind/Memcheck*：本机未安装；且与 ASan 覆盖面重叠，不额外引入。已拒绝。

**Verification**（本机实测，这一条同时构成 US5 的教学范例）：
```
$ cargo run                       # 未对齐写 + 重叠可变引用
2                                 # ← 程序正常退出，输出"合理"
$ cargo +nightly miri run
error: Undefined Behavior: memory access failed: attempting to access 8 bytes,
       but got alloc159+0x1 which is only 7 bytes from the end of the allocation
 --> src/main.rs:6:9
$ RUSTFLAGS="-Zsanitizer=address" cargo +nightly run -Zbuild-std --target x86_64-unknown-linux-gnu
==78659==ABORTING        # ← ASan 亦可用
```
这一对照（普通运行输出 `2`，Miri 判定 UB）正是 Spec Edge Case "unsafe 实验触发 UB 但程序表面正常
运行"的实证，直接支撑 FR-019 的立法理由。

---

## R-03: `no_std` 实验的目标平台（US7 / C-22–C-24）

**Decision**：使用 **`x86_64-unknown-none`** 作为 `no_std` 实验的编译目标，配合
`panic = "abort"`。该 target 已在 stable 与 nightly 上安装完成。`no_std` 实验组织为
**独立于主 workspace 的单独 crate**（根 `Cargo.toml` 用 `exclude` 排除），并携带自己的
`.cargo/config.toml` 固定 target。

**Rationale**：
- `x86_64-unknown-none` 是官方 Tier 2 裸机 target：无 std、无 OS services、无 unwinding，
  与 eBPF 执行环境在"缺失哪一类运行时能力"这一点上高度同构，正是 US7 要建立的认知。
- 它在 **stable** 上可用，无需 `-Zbuild-std`，因此 US7 的验收仍落在 R-01 指定的 stable 工具链上。
- 排除出主 workspace，是因为 `#![no_main]` + 自定义 `#[panic_handler]` 的 crate 无法用 host
  target 构建，会使 `cargo test --workspace` 失败；隔离后主 workspace 的一键验证保持干净，
  且 US7 实验获得真正的"独立可运行"属性。

**Alternatives considered**：
- *自定义 JSON target spec*：教学价值低（学的是 target spec 语法而非运行时边界），且需要
  `-Zbuild-std`（nightly）。已拒绝。
- *在 host target 上仅加 `#![no_std]` 的 lib*：无法暴露 panic handler 与 `eh_personality`
  这类"OS services 缺席"的真实约束，US7 AS3 无法验收。作为**第一步递进**保留，但不作为终点。
- *bpfel-unknown-none*：属于 Feature 002+ 的范围，FR-017 禁止在本 Feature 编写 eBPF 产物。已拒绝。

**Verification**：
```
$ rustup target add x86_64-unknown-none            # stable + nightly 均已安装
$ cargo build --target x86_64-unknown-none         # #![no_std] #![no_main] + panic_handler
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
```

---

## R-04: 编译器行为观察手段（FR-004）

**Decision**：按"由浅入深、够用即止"的阶梯选择观察手段，SHOULD 仅在低阶手段不足以判定时才升级：

| 阶梯 | 手段 | 工具链 | 用途 |
|-----|-----|--------|-----|
| 1 | 编译器诊断（错误码 + 措辞） | stable | US1/US4 的预测-验证循环 |
| 2 | `size_of` / `align_of` / `offset_of` 运行期断言 | stable | 布局（C-05、C-18、C-21） |
| 3 | `cargo rustc -- --emit=mir` | stable | 移动/drop 插入点、单态化实例 |
| 4 | `cargo rustc -- --emit=llvm-ir` | stable | 静态 vs 动态分发、内联、边界检查消除 |
| 5 | `-Z unpretty=mir` / `-Z print-mono-items` | nightly（分析用） | 阶梯 3–4 不足时 |
| 6 | `objdump -d` / `nm` / `readelf` | binutils 2.42 | `no_std` 产物的符号与节区检查 |

**Rationale**：阶梯 1–4 全部在 stable 上可用（已验证 `--emit=mir` 与 `--emit=llvm-ir` 在
stable 1.98.0 上正常产出），使绝大多数 FR-004 验证不依赖分析工具链。FR-004 本身写着
"SHOULD 仅在概念性解释不足以判定时才引入中间表示分析"，阶梯化正是该条款的执行形式。

**Alternatives considered**：
- *一律 dump LLVM IR*：违反 FR-004 的 SHOULD 条款，且 30KB 级 IR 会淹没教学重点。已拒绝。
- *依赖 Compiler Explorer 等在线工具*：违反 Constitution X（可复现性需本地记录环境）。
  MAY 作为交叉参考入口，MUST NOT 作为记录依据。

**Verification**：
```
$ rustc --emit=mir t.rs --out-dir .      → t.mir (3684 bytes)
$ rustc --emit=llvm-ir t.rs --out-dir .  → t.ll  (30805 bytes)
```

---

## R-05: "稳定断言 vs 非断言输出"的落地机制（FR-003 / Clarification 3）

**Decision**：用**物理隔离**而非约定来强制区分：

- **稳定断言**只存在于 `#[test]` 函数中（`tests/cNN_*.rs` 与 `#[cfg(test)]`）。
  硬性规则：`#[test]` 断言中 **MUST NOT** 出现指针地址值、时间测量值、线程调度顺序、
  哈希遍历顺序、`{:p}` 格式化结果。
- **非断言输出**只存在于 `examples/cNN_*.rs` 的 `println!` 中，运行结果抄录到模块的
  `OBSERVATIONS.md`，并标注 `NON-ASSERTION`。
- 一致性判定 = `cargo test --workspace` 全绿；`OBSERVATIONS.md` 的差异不参与判定。

**Rationale**：Clarification 明确"重跑仅比对稳定断言"。若把断言和观测混在同一个可执行体里，
学习者迟早会把一次偶然的地址值写进断言。用 `tests/` 与 `examples/` 两类 cargo 目标做物理隔离，
使这条规则可被机械检查，而不是靠自律。

**Alternatives considered**：
- *用 `trybuild` 做 stderr 全文比对*：`trybuild` 比对完整 stderr，等价于 Clarification 明确
  拒绝的"逐字节相同"，且对路径与措辞极度敏感。已拒绝，改用 R-06 的错误码断言。
- *快照测试（insta 等）*：同样是全文比对，且引入额外依赖。已拒绝。

---

## R-06: 编译失败实验的验证机制

**Decision**：自研极小的 `rf-harness::compile_fail` 断言器，**零外部依赖**。它调用 pinned stable
`rustc` 编译 `compile_fail/*.rs`，断言 stderr 中出现**指定的错误码**（如 `E0502`、`E0597`、
`E0277`），完整 stderr 作为非断言输出落盘。

**Rationale**：
- US1 的核心验收方式是"预测错误类型与出错位置，与编译器诊断一致"（AS1），US4 AS3 是"定位编译器
  拒绝的具体规则"。这两者需要的恰好是**错误码**级别的断言——错误码是 rustc 的稳定契约，措辞不是。
- 与 R-05 的"稳定断言"定义天然吻合：错误码稳定，诊断措辞与路径为非断言输出。
- 零依赖使实验保持"小、独立、可运行"（用户明确要求）。

**Alternatives considered**：
- *`trybuild` crate*：见 R-05，全文比对不符合 Clarification。已拒绝。
- *`#[doc = compile_fail]` doctest*：只能断言"编译失败"，无法断言**失败原因**，会让学习者用错误的
  理由通过验收。已拒绝。

---

## R-07: 运行时代价的可测量化（Constitution IX + US3 AS1）

**Decision**：本 Feature **不做计时类 benchmark**。所有"代价"结论改用**确定性可测量量**：

- **分配次数**：`rf-harness::CountingAllocator`（实现 `GlobalAlloc`，包装 `System`，用
  `AtomicUsize` 计数 alloc/dealloc/realloc 与字节数）。分配次数是确定性的 → 可作稳定断言。
  直接满足 US3 AS1"预测其求值顺序与实际发生的分配次数"。
- **内存布局**：`size_of` / `align_of` / `offset_of` 断言（C-05 判别式与 niche 优化、C-18、C-21）。
- **分发方式**：MIR/LLVM IR 中直接调用 vs 通过 vtable 间接调用的结构性观察（非断言）+
  单态化实例数量（`-Z print-mono-items`，非断言）+ `size_of::<&dyn Trait>() == 2 * size_of::<usize>()`
  这类可断言事实。

**Rationale**：
- Constitution IX 要求"所有'高性能'结论 MUST 通过实际测量验证"，同时"未测量的性能主张 MUST 标注为
  假设"。本 Feature 的 Technical Context 明确不追求性能，因此正确做法是**不产生需要 benchmark 的
  性能主张**，把话题限制在可确定性测量的结构性代价上。
- 计时结果在 WSL2 上抖动大，且天然属于非断言输出，写进验收只会制造假阳性/假阴性。
- `CountingAllocator` 同时是 C-24（allocator fundamentals）的活教材：学习者要实现一个
  `GlobalAlloc` 才能计数，等于亲手回答"分配能力由谁提供"。

**Alternatives considered**：
- *引入 criterion 做微基准*：与 Technical Context"本 Feature 不追求性能"冲突，且引入重依赖与
  长运行时间。已拒绝，推迟到后续 Feature。
- *用 `std::time::Instant` 手工计时*：结果不可作稳定断言，教学上还会诱导错误结论。仅允许作为
  `OBSERVATIONS.md` 中的 NON-ASSERTION 记录。

---

## R-08: 外部 crate 依赖策略（FR-016 的 Plan 侧决定）

**Decision**：**默认零依赖**。全 Feature 仅允许两个外部 crate，且各自绑定唯一用途：

| Crate | 用途 | 作用域 |
|-------|-----|-------|
| `libc` | US6 FFI 中调用真实 Linux API（`errno`、`open`/`close` 等） | 仅 `experiments/m6-ffi` |
| `cc` | US6 中编译配套的 C 源文件（build-dependency） | 仅 `experiments/m6-ffi` 的 `build.rs` |

其余全部能力用 `core` / `alloc` / `std` 实现。crates.io 已验证可达。

**Rationale**：用户明确要求"不要为了学习 Rust 而构建没有价值的大型应用"，且"实验应尽可能小、独立"。
每引入一个依赖，都会把学习者的注意力从语言机制转移到库 API 上——这与 Constitution I
（First-Principles）直接冲突。`libc` 与 `cc` 是例外，因为 US6 的学习对象**就是**跨语言边界本身，
手写 `extern "C"` 声明与手工调用 gcc 只会增加噪音而不增加理解。

**Alternatives considered**：
- *完全零依赖，手写 syscall*：`libc` 的类型别名（`c_int`、`c_char`）本身就是 C-21 的教学内容
  （"布局一致性靠什么保证"），手写反而绕开该知识点。已拒绝。
- *引入 `bindgen`*：自动生成会掩盖 US6 AS1 要学的"为何默认布局不可依赖"。已拒绝——手写声明是
  本 Story 的学习目的。

**Verification**：`cargo add libc` → `Updating crates.io index / Locking 1 package` 成功。

---

## R-09: 项目结构（learning / experiments / feynman / acceptance 四分）

**Decision**：采用四顶层目录分离 + 单 cargo workspace，详见 plan.md 的 Project Structure。
实验按 **8 个学习模块**（对应 8 个 Story）划分 crate，每个 **capability（C-01…C-24）**在模块内
拥有独立的 example（可观察）与 test 文件（可断言）。

**Rationale**：
- Clarification 已裁定"Feynman 材料按 8 个模块产出；24 项能力各自保留独立的 Acceptance Criteria
  与实验"。crate 粒度取 8（对齐 Feynman 与模块验收），文件粒度取 24（对齐能力级验收），
  与该裁定一一对应。
- 24 个独立 crate 会让 workspace 构建与交叉引用变得笨重，却不带来额外独立性——example 与
  integration test 本身就是独立编译、独立运行的 cargo 目标（`cargo run --example c03_borrow`
  可单独执行），已满足"小、独立、可运行、可观察"。

**Alternatives considered**：
- *24 个 crate*：独立性无实质提升，构建开销与样板显著上升。已拒绝。
- *单一 crate 装下全部*：模块无法独立验证，违反用户"每个核心知识模块应能被独立验证"的要求。已拒绝。

---

## R-10: Send/Sync 判定题集的冻结机制（SC-007）

**Decision**：题集在 **US4 学习开始前**定稿于 `acceptance/send-sync-quiz.md`（≥10 个自定义类型），
参考答案与推导写入 `acceptance/send-sync-quiz.answers.md`，作答前 MUST NOT 打开答案文件。
判定题的客观校验由 `experiments/m4-concurrency/tests/c12_send_sync_quiz.rs` 完成：对每个类型用
`fn assert_send<T: Send>()` / `assert_sync<T: Sync>()` 做正向断言，用 `compile_fail/` 条目做
负向断言（期望 `E0277`）。

**Rationale**：SC-007 要求"题目 MUST 在学习开始前定稿，MUST NOT 在验收时另行挑选"。把题集落到
版本控制中的独立文件，并让编译器成为客观裁判，同时满足"定稿"与"判定客观"。学习者的书面推导依据
则由 Feynman 材料承载（SC-007 要求"能对每次判定给出推导依据而非结论"）。

---

## Unresolved

无。Technical Context 中不存在 NEEDS CLARIFICATION 项；Spec 推迟到 Plan 的四项决策
（工具链版本 FR-020、UB 工具 FR-019、实验组织形式、所用 crate）已分别由 R-01、R-02、R-09、R-08 解决。
