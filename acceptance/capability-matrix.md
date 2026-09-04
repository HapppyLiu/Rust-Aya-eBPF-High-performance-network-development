# Capability Matrix —— 追踪链单一事实源

**Feature**: 001-rust-foundation | **依据**: FR-001 / FR-013 / SC-003 / SC-011 |
**契约**: [learning-artifact-contract §E](../specs/001-rust-foundation/contracts/learning-artifact-contract.md)

本表是 **Capability 状态的唯一权威**。`acceptance/criteria/cNN.md` 里的 `Status`
记的是那条 AC 自身的判定结果，两者含义不同，不要互相覆盖（§C1a）。

FR-013 要求的七段链条在本表中逐列体现：

```text
Spec        →  Plan          →  Task  →  Learning Material  →  Experiment  →  Source Code  →  Acceptance Criteria
C-ID/Story     Module/UB Tool    Task     （learning/mN/）       Experiment      SourceRef       Criterion
```

## 状态机（data-model §1）

```text
planned → in-progress → experiment-passed → accepted
               ▲                              │
               └────────── regressed ◀────────┘
```

- `planned → in-progress`：概念材料与源码引用已建立
- `in-progress → experiment-passed`：全部稳定断言通过，且（`UB Tool ≠ n/a` 时）UB 判定符合**事前预期**
- `experiment-passed → accepted`：AC 判定通过 **且**所属模块 Feynman 五项检验全部通过
- `accepted → regressed`：重跑时稳定断言未复现，或工具链被迫升级致记录失效（FR-020）

**只有 `accepted` 计入进度。** 模块 Feynman fail 时，该模块能力停在 `experiment-passed`（§C1a）。

## 矩阵

| C-ID | Capability | Module | Story | Task | Experiment | SourceRef | Criterion | UB Tool | Status |
|------|-----------|--------|-------|------|-----------|-----------|-----------|---------|--------|
| C-01 | Ownership | m1 | US1 | T029 | `c01_ownership` | `core/src/ops/drop.rs:209` `Drop`、`:206` `#[lang="drop"]` | [criteria/c01.md](criteria/c01.md) | n/a | **accepted** |
| C-02 | Move semantics | m1 | US1 | T030 | `c02_move` | `core/src/mem/mod.rs:953/886/189` `replace`/`take`/`forget` | [criteria/c02.md](criteria/c02.md) | n/a | **accepted** |
| C-03 | Borrowing | m1 | US1 | T031 | `c03_borrow` | `core/src/cell.rs:849` `RefCell`、`:945` `BorrowCounter` | [criteria/c03.md](criteria/c03.md) | n/a | **accepted** |
| C-04 | Lifetime | m1 | US1 | T032 | `c04_lifetime` | `core/src/marker.rs:811` `PhantomData`、`:805`（零大小保证） | [criteria/c04.md](criteria/c04.md) | n/a | **accepted** |
| C-05 | Struct / Enum | m2 | US2 | T041 | `c05_layout` | `core/src/option.rs`（niche）、`core/src/mem/mod.rs` `size_of` | [criteria/c05.md](criteria/c05.md) | n/a | planned |
| C-06 | Trait | m2 | US2 | T042 | `c06_trait` | `core/src/fmt/mod.rs` `Display`、`core/src/ops/deref.rs` `Deref` | [criteria/c06.md](criteria/c06.md) | n/a | planned |
| C-07 | Generic | m2 | US2 | T043 | `c07_generic` | `core/src/iter/traits/iterator.rs`、`core/src/cmp.rs` `PartialOrd` | [criteria/c07.md](criteria/c07.md) | n/a | planned |
| C-08 | Error handling | m3 | US3 | T052 | `c08_error` | `core/src/result.rs`、`core/src/convert/mod.rs` `From` | [criteria/c08.md](criteria/c08.md) | n/a | planned |
| C-09 | Iterator | m3 | US3 | T053 | `c09_iterator` | `core/src/iter/traits/iterator.rs`、`core/src/iter/adapters/map.rs` | [criteria/c09.md](criteria/c09.md) | n/a | planned |
| C-10 | Closure | m3 | US3 | T054 | `c10_closure` | `core/src/ops/function.rs` `Fn`/`FnMut`/`FnOnce` | [criteria/c10.md](criteria/c10.md) | n/a | planned |
| C-11 | Smart pointer | m3 | US3 | T055 | `c11_smart_ptr` | `alloc/src/boxed.rs`、`alloc/src/rc.rs`、`alloc/src/sync.rs` `Arc` | [criteria/c11.md](criteria/c11.md) | n/a | planned |
| C-12 | Send / Sync | m4 | US4 | T064, T067 | `c12_send_sync` | `core/src/marker.rs` `Send`/`Sync` | [criteria/c12.md](criteria/c12.md) | miri | planned |
| C-13 | Concurrency | m4 | US4 | T065 | `c13_concurrency` | `std/src/thread/mod.rs`、`std/src/sync/mutex.rs` | [criteria/c13.md](criteria/c13.md) | miri（many-seeds） | planned |
| C-14 | Atomic | m4 | US4 | T066 | `c14_atomic` | `core/src/sync/atomic.rs` `Ordering`/`AtomicUsize` | [criteria/c14.md](criteria/c14.md) | miri（many-seeds） | planned |
| C-15 | Unsafe Rust | m5 | US5 | T077, T078 | `c15_unsafe` | `core/src/slice/mod.rs` `get_unchecked` | [criteria/c15.md](criteria/c15.md) | miri | planned |
| C-16 | Raw pointer | m5 | US5 | T079, T080 | `c16_raw_ptr` | `core/src/ptr/mod.rs`、`core/src/ptr/const_ptr.rs` `read`/`write` | [criteria/c16.md](criteria/c16.md) | miri | planned |
| C-17 | Pointer arithmetic | m5 | US5 | T081, T082 | `c17_ptr_arith` | `core/src/ptr/const_ptr.rs` `add`/`offset`/`wrapping_add` | [criteria/c17.md](criteria/c17.md) | miri | planned |
| C-18 | Alignment | m5 | US5 | T083, T084 | `c18_alignment` | `core/src/mem/mod.rs` `align_of`、`core/src/ptr/mod.rs` `read_unaligned` | [criteria/c18.md](criteria/c18.md) | miri | planned |
| C-19 | Aliasing | m5 | US5 | T085, T086 | `c19_aliasing` | `core/src/cell.rs` `UnsafeCell` | [criteria/c19.md](criteria/c19.md) | miri（SB + TB 对照） | planned |
| C-20 | Memory safety | m5 | US5 | T087, T088, T090 | `c20_mem_safety` | `core/src/slice/raw.rs` `from_raw_parts`、`alloc/src/vec/mod.rs` `set_len` | [criteria/c20.md](criteria/c20.md) | miri | planned |
| C-21 | FFI | m6 | US6 | T100, T101, T102, T103 | `c21_ffi` | `core/src/ffi/mod.rs` `c_int`/`c_char`、`std/src/ffi/c_str.rs` `CStr` | [criteria/c21.md](criteria/c21.md) | asan | planned |
| C-22 | no_std | m7 | US7 | T112 | `c22_nostd` | `core/src/lib.rs` `#![no_std]`、`std/src/lib.rs` | [criteria/c22.md](criteria/c22.md) | compile-time | planned |
| C-23 | core / alloc / std | m7 | US7 | T113 | `c23_core_alloc_std` | `alloc/src/lib.rs`、`std/src/lib.rs`（re-export） | [criteria/c23.md](criteria/c23.md) | compile-time | planned |
| C-24 | Panic and allocator fundamentals | m7 | US7 | T114, T117 | `c24_panic_alloc` | `core/src/panicking.rs`、`core/src/alloc/global.rs` `GlobalAlloc` | [criteria/c24.md](criteria/c24.md) | compile-time + miri（host 侧 allocator） | planned |

**行数校验**：24 行，C-01…C-24 无遗漏、无重复、无无归属项（FR-001 / 规则 E1）。

## 模块状态

| Module | Story | Priority | Capabilities | Prerequisite | Feynman | Status | FR-012 硬前置 |
|--------|-------|----------|-------------|--------------|---------|--------|--------------|
| m1 | US1 | P1 | C-01…C-04 | — | **passed** | **accepted** | **是** |
| m2 | US2 | P2 | C-05…C-07 | m1 | pending | pending | 否 |
| m3 | US3 | P2 | C-08…C-11 | m2 | pending | pending | 否 |
| m4 | US4 | P2 | C-12…C-14 | m3 + T024 题集冻结 | pending | pending | 否 |
| m5 | US5 | P1 | C-15…C-20 | m4 | pending | pending | **是** |
| m6 | US6 | P2 | C-21 | m5 | pending | pending | 否 |
| m7 | US7 | P1 | C-22…C-24 | m6 | pending | pending | **是** |
| m8 | US8 | P3 | 综合（C-01…C-24 全部） | m1–m7 全部通过 | pending | pending | 否 |

### FR-012 硬前置进度

| 硬前置模块 | 状态 | 达成日期 | 依据 |
|-----------|------|---------|------|
| **m1**（C-01…C-04） | ✅ **已满足** | 2026-09-04 | `cargo test -p m1-ownership` 33 项全绿；`criteria/c01…c04.md` 四项 `pass`；`feynman/m1-ownership.md` 五项检验全 `pass` |
| m5（C-15…C-20） | 未开始 | — | — |
| m7（C-22…C-24） | 未开始 | — | — |

> **FR-011a 提醒**：三个硬前置是 Feature 002 的**必要**条件，不是充分条件。
> US2/US3/US4/US6 任一未通过时 MUST NOT 启动 Feature 002 —— 硬前置模块本身的验收
> 依赖它们的前置链，前置未过则该验收不可信。
