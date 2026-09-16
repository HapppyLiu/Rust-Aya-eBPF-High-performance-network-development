# Environment Baseline

**Feature**: 001-rust-foundation | **建立于**: 2026-09-04 | **依据**: FR-010 / FR-018 / Constitution X

本文件是后续**全部** `OBSERVATIONS.md` 环境块的基准。
各实验的环境块 MUST 与本基线一致；不一致时 MUST 说明差异来源，
并按 FR-020 判断受影响实验的验收记录是否需要重新验证。

## 关于 pinned 工具链的一条记录（FR-020）

`rust-toolchain.toml` 按 R-01 用**精确版本号** `1.98.0` 而非 `stable` 声明 channel。
首次在本仓库执行 `rustc` 时，rustup 因此安装了一个独立命名的
`1.98.0-x86_64-unknown-linux-gnu` 工具链（见下方 `rustup toolchain list`）。

这**不是** `rustup update`，FR-020 未被违反 —— 它禁止的是把工具链**升级**到新版本，
而这里是把已声明的 pinned 版本**安装**到位。校验：新装工具链的 `commit-hash`
`88d9e12ae178fab0fb5cc050a94da85685d449ea` 与 research.md R-01 记录的完全一致。

这样做的收益正是 FR-020 的立法意图：`stable` 这个名字会随时间指向新版本，
而 `1.98.0` 不会。此后即使 `stable` 前进，本仓库的构建与验收仍钉在同一个编译器上，
既有的错误码预测、诊断措辞与 IR 观察不会失效。

`rustc --print sysroot` 因此指向 `~/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu`，
这也是 `source-refs.md` 中源码路径的根（`$(rustc --print sysroot)/lib/rustlib/src/rust/library/`）。

## 环境记录

| 字段 | 值 |
|------|-----|
| `rustc_stable` | 1.98.0 (88d9e12ae 2026-08-18) |
| `rustc_nightly` | 1.100.0-nightly (17fd5b8a3 2026-08-28) |
| `edition` | 2024 |
| `kernel` | 6.6.114.1-microsoft-standard-WSL2 |
| `arch` | x86_64 |
| `target` | x86_64-unknown-linux-gnu |
| `command` | （基线：见下方逐条校验命令） |

## quickstart §0 环境基线校验

```text
$ rustc -Vv
rustc 1.98.0 (88d9e12ae 2026-08-18)
binary: rustc
commit-hash: 88d9e12ae178fab0fb5cc050a94da85685d449ea
commit-date: 2026-08-18
host: x86_64-unknown-linux-gnu
release: 1.98.0
LLVM version: 22.1.8

$ rustup run nightly rustc -Vv
rustc 1.100.0-nightly (17fd5b8a3 2026-08-28)
binary: rustc
commit-hash: 17fd5b8a37b6667b6cc137f3cc35f09759768a3b
commit-date: 2026-08-28
host: x86_64-unknown-linux-gnu
release: 1.100.0-nightly
LLVM version: 23.1.0

$ rustup toolchain list
stable-x86_64-unknown-linux-gnu (default)
nightly-x86_64-unknown-linux-gnu
1.98.0-x86_64-unknown-linux-gnu (active)

$ rustup target list --toolchain 1.98.0 --installed
x86_64-unknown-linux-gnu
x86_64-unknown-none

$ rustup component list --toolchain 1.98.0 --installed
cargo-x86_64-unknown-linux-gnu
clippy-x86_64-unknown-linux-gnu
rust-src
rust-std-x86_64-unknown-linux-gnu
rust-std-x86_64-unknown-none
rustc-x86_64-unknown-linux-gnu
rustfmt-x86_64-unknown-linux-gnu

$ rustup run nightly cargo miri --version
miri 0.1.0 (17fd5b8a37 2026-08-28)

$ uname -r && uname -m
6.6.114.1-microsoft-standard-WSL2
x86_64

$ cc --version | head -1
cc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0

$ nm --version | head -1
GNU nm (GNU Binutils for Ubuntu) 2.42

$ rustc --print sysroot
/root/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu
```
