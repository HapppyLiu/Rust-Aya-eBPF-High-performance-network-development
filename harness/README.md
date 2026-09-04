# harness/ —— `rf-harness` 验证设施

本 Feature 唯一的共享设施 crate。**零外部依赖**（只用 `std`）。

契约：[harness-api.md](../specs/001-rust-foundation/contracts/harness-api.md)

| 模块 | 提供什么 | 服务于 |
|-----|---------|-------|
| `compile_fail` | 编译失败的**错误码**断言器 | US1/US4 的"预测编译器诊断"验收循环（R-06） |
| `counting_alloc` | `CountingAllocator` + `measure`：确定性分配计数 | US3 AS1 的"预测分配次数"；替代计时 benchmark（R-07） |
| `miri` | Miri 判定结果的结构化读取 | FR-019 的 UB 判定 |
| `env` | 环境记录采集与 Markdown 渲染 | FR-010 |

## 设计约束

**加入这里的 API MUST 是"验证设施"，而不是"被学习的对象"。**
链表、环形缓冲、解析器一类**学习目标本身**的代码属于各模块 crate 的 `src/`，不属于这里。

明确不提供：计时/benchmark 设施（本 Feature 不产生需 benchmark 的性能主张）、
对 `unsafe` 的封装糖（学习者需要直接面对 unsafe，封装会掩盖 Constitution VI 要陈述的不变量）。

## 关键契约

- `compile_fail::expect_errors` 断言 `expected ⊆ actual`（子集语义），失败时 MUST 同时打印
  **期望**与**实际**错误码 —— 这正是 US1 AS1 的验收形式。
- `miri::MiriOutcome::reported_ub()` 在 Miri **未运行**时 **panic** 而非返回 `false`。
  这是 FR-019 在类型层面的强制：不允许"没跑工具"被静默当成"没有 UB"。
- `counting_alloc` 的分配器是全局的，但计数**按线程隔离**：
  `measure` 只统计调用线程的分配活动，因此无需与其他测试串行执行。
  边界是闭包内新起线程的分配不计入（见 harness-api §统计范围）。

## 自检

`tests/harness_selfcheck.rs` 验证设施本身的行为。设施不可靠则建立在它之上的**全部**验收失效，
因此这个文件先于任何学习模块存在。
