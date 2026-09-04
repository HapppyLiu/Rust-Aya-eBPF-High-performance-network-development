#!/usr/bin/env bash
# UB 判定入口（R-02 / FR-019）。统一 MIRIFLAGS，避免每次手敲导致标志不一致。
#
# ⚠️ 判定纪律（FR-019 / experiment-contract §C5.2）：
#    - 本脚本**未运行成功**时，ub_verdict MUST 记 `n/a`，MUST NOT 记 `clean`。
#    - "程序未崩溃" MUST NOT 作为无 UB 的证据。
#    - expected-ub 实验的类别预测 MUST 在跑本脚本**之前**提交（§C5.1a）。
#
# 用法：
#   tools/run-miri.sh <package> [--tree-borrows] [--many-seeds] [-- <extra cargo args>]
# 例：
#   tools/run-miri.sh m5-unsafe
#   tools/run-miri.sh m5-unsafe --tree-borrows
#   tools/run-miri.sh m4-concurrency --many-seeds
set -euo pipefail

usage() { sed -n '2,${/^#/!q;s/^# \{0,1\}//;p}' "$0"; exit "${1:-0}"; }
[[ $# -lt 1 || "${1:-}" == "-h" || "${1:-}" == "--help" ]] && usage 0

PKG="$1"; shift

FLAGS=()
MODEL="stacked-borrows (default)"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --tree-borrows) FLAGS+=(-Zmiri-tree-borrows); MODEL="tree-borrows"; shift ;;
        --many-seeds)   FLAGS+=(-Zmiri-many-seeds);   shift ;;
        --) shift; break ;;
        *) echo "unknown argument: $1" >&2; usage 1 ;;
    esac
done

if ! rustup run nightly cargo miri --version >/dev/null 2>&1; then
    echo "ERROR: pinned nightly 上未找到 miri。" >&2
    echo "       ub_verdict MUST 记为 'n/a'，MUST NOT 记为 'clean'（FR-019）。" >&2
    exit 2
fi

echo "==> miri: package=${PKG} aliasing-model=${MODEL} flags=${FLAGS[*]:-none}"
set +e
MIRIFLAGS="${FLAGS[*]:-}" cargo +nightly miri test -p "$PKG" "$@"
rc=$?
set -e

echo
case "$rc" in
    0) echo "==> 退出码 0：Miri 未报告 UB。ub_verdict 候选 = clean" ;;
    *) echo "==> 退出码 ${rc}：Miri 报告了问题。"
       echo "    若该实验意图为 UB 对照 → 核对错误类别是否命中事前 PREDICT-UB（§C5.1a）："
       echo "      命中   → ub_verdict = expected-ub（pass）"
       echo "      未命中 → ub_verdict = unexpected-ub（**fail**），MUST 保留原预测并书面复盘" ;;
esac
exit "$rc"
