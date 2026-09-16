# Rust + Aya/eBPF 高性能网络开发学习工程

学习产物按类型放在仓库根目录，再按 Feature 分子目录。`specs/` 只放规格文档。

```text
specs/        001-rust-foundation/   Spec / Plan / Tasks
learning/     001-rust-foundation/   Answer Track（概念 + 源码引用）
learner/      001-rust-foundation/   Learner Track（问题，不含答案）
feynman/      001-rust-foundation/   Feynman 教学
experiments/  001-rust-foundation/   可执行实验
acceptance/   001-rust-foundation/   验收
contracts/    001-rust-foundation/   产物契约
```

当前 Feature：[001-rust-foundation](specs/001-rust-foundation/)（系统级 Rust 地基）。
验证入口：[quickstart.md](specs/001-rust-foundation/quickstart.md)

```bash
cargo test --workspace
cd experiments/001-rust-foundation/m7-nostd && cargo build
```

不要执行 `rustup update`。本 Feature 锁定 stable 1.98.0。
