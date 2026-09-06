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

DEST="${OUT_DIR}/${LABEL}.ll"
echo "==> emitting LLVM IR for ${LABEL} -> ${DEST}"

# rustc 会在 `-o` 之外仍产出 link 产物，并给文件名追加 metadata hash
# （"output file name will be adapted for each output type"），因此先落到临时目录，
# 再把唯一的 .ll 规范化成稳定文件名。直接信任 `-o` 的路径会拿到不存在的文件。
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

cargo rustc -p "$PKG" "${TARGET_ARGS[@]}" -- \
    --emit=llvm-ir -o "${STAGE}/${LABEL}.ll" 2>&1 |
    grep -v -e 'multiple output types requested' -e 'ignoring --out-dir' -e '^warning: *$' || true

mapfile -t EMITTED < <(find "$STAGE" -name '*.ll' -type f)
if [[ ${#EMITTED[@]} -ne 1 ]]; then
    echo "!! 预期恰好一个 .ll 产物，实际 ${#EMITTED[@]} 个：${EMITTED[*]:-<none>}" >&2
    exit 1
fi

mkdir -p "$OUT_DIR"
mv "${EMITTED[0]}" "$DEST"

echo "==> ${DEST} ($(wc -l <"$DEST") lines)"
echo "    30KB 级 IR 会淹没教学重点；抄录进 OBSERVATIONS 时只取相关函数体，并标注 [NON-ASSERTION]。"
