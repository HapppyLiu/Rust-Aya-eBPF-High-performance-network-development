# Module N 源码引用

<!--
模板：learning-artifact-contract §B。复制为 learning/mN-<module>/source-refs.md 后填写。

路径根：$(rustc --print sysroot)/lib/rustlib/src/rust/library/
本机为：/root/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/

行号在 pinned 工具链（1.98.0）下固定，因此**可以**被记录并复核（规则 B1）。
定位命令示例：
    SRC="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
    grep -n 'pub trait Drop' "$SRC/core/src/ops/drop.rs"
-->

**Story**: USn | **Capabilities**: C-xx … C-yy

| C-ID | 路径（相对 `library/`） | 符号 | 行 | kind | 这段源码回答了什么 |
|------|----------------------|------|----|------|------------------|
| C-xx | `core/src/...` | `SomeTrait` | 68 | library | <这段源码回答了 concept.md 里的哪个问题> |
| C-yy | — Rust Reference §"..." | — | — | reference-fallback | <为什么无库代码对应> |

## 规则提醒

- **规则 B3**：MUST 有"这段源码回答了什么"一列。只记路径不记问题的引用等同于孤立笔记
  （Constitution XIII）—— 它无法被审查，也无法在下次回看时告诉你当初为什么要看它。
- **规则 B2**：`kind = reference-fallback` **仅**允许用于确实无库代码对应的能力。
  已知的三项：C-03（borrowck 属编译器内建）、C-04（elision 规则）、C-15（UB 定义）。
  其余能力若用 fallback，MUST 在下方说明理由。

## reference-fallback 理由说明

（若本模块用到了非"已知三项"之外的 fallback，在此逐条说明为何无库代码对应。）
