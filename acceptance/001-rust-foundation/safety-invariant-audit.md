# SAFETY 五要素审查（T089 / FR-008 / SC-010）

**Module**: m5-unsafe | **日期**: 2026-09-08
**机械兜底**：`cargo clippy -p m5-unsafe --all-targets -- -D warnings` 退出码 0
（`undocumented_unsafe_blocks` + `multiple_unsafe_ops_per_block` = deny）

人工审查范围：`experiments/m5-unsafe/` 下全部 `unsafe` 块
（含 `examples/`、`src/`、`tests/`）。`learning/` / `feynman/` 本模块没有可编译示例代码。

判定：每一块逐项看有效性 / 对齐 / 别名 / provenance / 生命周期。
"同上"仅当紧邻上一块、五要素未变时合格。故意 UB 的块必须写明**哪一项故意不成立**。

| 位置 | 操作 | 五要素 | 判定 |
|------|------|--------|------|
| `src/c15.rs` `get_in_bounds` | `get_unchecked` | 先 `get` 证明下标；`u8` 对齐 1；只读别名；来自 slice；返回拷贝 | pass |
| `examples/c15_unsafe_ub.rs` | `get_unchecked(3)` | 有效性/provenance **故意不成立**；其余写明不是本实验要打破的 | pass |
| `src/c16.rs` 写 / 读 | `ptr::write` / `read` | 栈上活着的 `i32`；对齐由局部变量保证；无引用；无偏移 | pass |
| `examples/c16_raw_ptr_ub.rs` `from_raw` | 拼回 `Box` | 此时分配仍在，唯一所有者 | pass |
| `examples/c16_raw_ptr_ub.rs` `ptr::read` | 释放后读 | 有效性/寿命/provenance **故意不成立** | pass |
| `src/c17.rs` 各 `add`/`offset`/`read` | 分配内一步 | `index < 3` 或写死的 `1`；元素对齐；只读 | pass |
| `examples/c17_ptr_arith_ub.rs` | `add(8)` | provenance **故意不成立**；不解引用故别名不适用 | pass |
| `src/c18.rs` `read_aligned` | `ptr::read` | 独立 `u64`；取模关系由测试钉住 | pass |
| `src/c18.rs` `read_unaligned_u16` | `read_unaligned` | 对齐 **不适用（方法卸掉）**；两字节仍在分配内 | pass |
| `examples/c18_alignment_ub.rs` `add(1)` | 造指针 | `u8` 一步仍在 8 字节内 | pass |
| `examples/c18_alignment_ub.rs` write/read | 未对齐 `u64` | 有效性与对齐 **故意不成立**（7 字节剩额 / 未对齐） | pass |
| `src/c19.rs` 三次 `*p` | 顺序写读 | 无同时活着的引用；`get` 只交指针 | pass |
| `examples/c19_aliasing_ub.rs` 第一份 `&mut` | 造引用 | 此时只有一份 | pass |
| `examples/c19_aliasing_ub.rs` 第二份 `&mut` | 叠 `&mut` | 别名 **故意不成立** | pass |
| `src/c20.rs` `parse_u16_be_unchecked` 四块 | `add` / deref | 调用方保证 `off+2<=len`；`u8` 对齐 1 | pass |
| `src/c20.rs` `as_u16_slice` | `from_raw_parts` | 公开 API 已拒绝奇数/未对齐 | pass |
| `examples/c20_mem_safety_ub.rs` | `set_len` | 有效性 **故意不成立**（未初始化） | pass |
| `examples/c20_bounds_check.rs` | 调 `unsafe fn` | 刚 `checked` 返回 `Some`；五要素写全 | pass |
| `tests/c20_mem_safety.rs` | 同上 | 同上 | pass |

**覆盖率**：上表全部 pass。空洞说明（"Rust 要求 unsafe"）零出现。

**不适用项摘要**：
- `u8` 访问：对齐不适用，原因是 `align_of::<u8>() == 1`。
- `read_unaligned`：对齐不适用，原因是该方法的契约卸掉对齐。
- `add` 而不解引用：别名不适用，原因是没有创建引用、没有访问。
- 故意 UB 块：被打破的那一项写"故意不成立"，其余仍逐项表态。
