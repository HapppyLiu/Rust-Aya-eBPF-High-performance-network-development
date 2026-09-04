//! `rf-harness` —— Rust Foundation 学习工程的共享验证设施。
//!
//! 契约：`specs/001-rust-foundation/contracts/harness-api.md`
//!
//! 本 crate 存在的理由是让实验契约可被**机械执行**，而不是靠自律：
//!
//! - [`compile_fail`] —— 断言编译失败的**错误码**（rustc 的稳定契约），而非诊断措辞（不稳定）。
//! - [`counting_alloc`] —— 用**确定性**的分配次数替代计时 benchmark。
//! - [`miri`] —— 结构化读取 Miri 的 UB 判定；未运行时**拒绝**给出"无 UB"的答案。
//! - [`env`] —— 环境记录采集，与 `tools/env-record.sh` 输出格式一致。
//!
//! # 非目标
//!
//! 本 crate MUST NOT 包含任何**学习目标本身**的代码（链表、环形缓冲、解析器等），
//! 那些属于各模块 crate 的 `src/`。加入这里的 API 必须是"验证设施"，
//! 而不是"被学习的对象"。
//!
//! 同样 MUST NOT 提供对 `unsafe` 的封装糖 —— 学习者需要直接面对 `unsafe`，
//! 封装会掩盖 Constitution VI 要求陈述的那些不变量。

pub mod compile_fail;
pub mod counting_alloc;
pub mod env;
pub mod miri;
