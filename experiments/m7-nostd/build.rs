//! C-22 构建入口：拒绝在非裸机 target 上配置本 crate。
//!
//! `build.rs` 跑在 **host** 上，但 `TARGET` 是即将交叉编译的三元组。
//! 这是 CHK041 允许的编译期断言载体：失败以非零退出码表现，不依赖 `#[test]`。

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target != "x86_64-unknown-none" {
        panic!(
            "C-22: m7-nostd MUST be built for x86_64-unknown-none (got {target:?}). \
             Use `cd experiments/m7-nostd && cargo build` — this crate is excluded from the root workspace (R-03)."
        );
    }
    println!("cargo:rerun-if-env-changed=TARGET");
}
