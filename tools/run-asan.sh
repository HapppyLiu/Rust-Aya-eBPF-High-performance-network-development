#!/usr/bin/env bash
# FFI 场景的 UB 判定入口（R-02 / experiment-contract §C5.4）。
#
# ⚠️ 判定强度声明（spec.md Assumptions，T004 登记）：
#    ASan 的覆盖面**窄于** Miri。它检测机器层可观测的内存错误（越界、UAF、double-free），
#    **不能**检测别名违规、provenance 违规、未对齐访问等 Rust 语义层 UB。
#    因此本脚本无报告时，结论 MUST 表述为「ASan 未在本次运行中观测到内存错误」，
#    MUST NOT 表述为「该 FFI 代码无 UB」。
#
# 之所以用 ASan 而非 Miri：Miri 无法执行真实的 C 函数调用。
#
# 用法：
#   tools/run-asan.sh <package> [-- <extra cargo args>]
# 例：
#   tools/run-asan.sh m6-ffi
set -euo pipefail

usage() { sed -n '2,${/^#/!q;s/^# \{0,1\}//;p}' "$0"; exit "${1:-0}"; }
[[ $# -lt 1 || "${1:-}" == "-h" || "${1:-}" == "--help" ]] && usage 0

PKG="$1"; shift

if ! rustup run nightly rustc --version >/dev/null 2>&1; then
    echo "ERROR: 未找到 pinned nightly（-Zsanitizer 需要 nightly）。" >&2
    echo "       ub_verdict MUST 记为 'n/a'，MUST NOT 记为 'clean'（FR-019）。" >&2
    exit 2
fi

echo "==> asan: package=${PKG}"
set +e
RUSTFLAGS="-Zsanitizer=address" \
RUSTDOCFLAGS="-Zsanitizer=address" \
cargo +nightly test -p "$PKG" \
    -Zbuild-std --target x86_64-unknown-linux-gnu "$@"
rc=$?
set -e

echo
case "$rc" in
    0) echo "==> 退出码 0：ASan 未在本次运行中观测到内存错误。"
       echo "    ub_verdict 候选 = clean（**仅在 ASan 覆盖面内**成立，见文件头的强度声明）。"
       echo "    该假设 MUST 抄录进 experiments/${PKG}/OBSERVATIONS.md 的判定说明。" ;;
    *) echo "==> 退出码 ${rc}：ASan 报告了内存错误。核对是否命中事前预测。" ;;
esac
exit "$rc"
