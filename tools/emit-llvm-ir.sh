#!/usr/bin/env bash
# 导出 LLVM IR（R-04 阶梯 4）。产物落到 target/ir/。
#
# IR 文本一律为 NON-ASSERTION（experiment-contract §C3.3）。
# 典型用途：对照静态分发（直接调用）与动态分发（vtable 间接调用）的结构差异。
#
# 用法：
#   tools/emit-llvm-ir.sh <package> [--example <name>]
# 例：
#   tools/emit-llvm-ir.sh m2-types --example c06_trait
set -euo pipefail

usage() { sed -n '2,${/^#/!q;s/^# \{0,1\}//;p}' "$0"; exit "${1:-0}"; }
[[ $# -lt 1 || "${1:-}" == "-h" || "${1:-}" == "--help" ]] && usage 0

PKG="$1"; shift
OUT_DIR="target/ir"
mkdir -p "$OUT_DIR"

TARGET_ARGS=()
LABEL="$PKG"
if [[ "${1:-}" == "--example" ]]; then
    TARGET_ARGS=(--example "$2")
    LABEL="${PKG}-$2"
    shift 2
fi

echo "==> emitting LLVM IR for ${LABEL} -> ${OUT_DIR}/"
cargo rustc -p "$PKG" "${TARGET_ARGS[@]}" -- \
    --emit=llvm-ir -o "${OUT_DIR}/${LABEL}.ll"

echo "==> ${OUT_DIR}/${LABEL}.ll ($(wc -l <"${OUT_DIR}/${LABEL}.ll") lines)"
echo "    30KB 级 IR 会淹没教学重点；抄录进 OBSERVATIONS 时只取相关函数体，并标注 [NON-ASSERTION]。"
