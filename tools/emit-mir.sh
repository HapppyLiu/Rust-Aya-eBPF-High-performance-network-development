#!/usr/bin/env bash
# 导出 MIR（R-04 阶梯 3）。产物落到 target/ir/。
#
# MIR 文本一律为 NON-ASSERTION（experiment-contract §C3.3）。
# 需要断言的部分 MUST 先转化为确定性量（size_of / 分配计数 / 单态化实例数）再写进 tests/。
#
# 用法：
#   tools/emit-mir.sh <package> [--example <name>]
# 例：
#   tools/emit-mir.sh m1-ownership --example c01_ownership
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

DEST="${OUT_DIR}/${LABEL}.mir"
echo "==> emitting MIR for ${LABEL} -> ${DEST}"

# rustc 会在 `-o` 之外仍产出 link 产物，并给文件名追加 metadata hash
# （"output file name will be adapted for each output type"），因此先落到临时目录，
# 再把唯一的 .mir 规范化成稳定文件名。直接信任 `-o` 的路径会拿到不存在的文件。
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

cargo rustc -p "$PKG" "${TARGET_ARGS[@]}" -- \
    --emit=mir -o "${STAGE}/${LABEL}.mir" 2>&1 |
    grep -v -e 'multiple output types requested' -e 'ignoring --out-dir' -e '^warning: *$' || true

mapfile -t EMITTED < <(find "$STAGE" -name '*.mir' -type f)
if [[ ${#EMITTED[@]} -ne 1 ]]; then
    echo "!! 预期恰好一个 .mir 产物，实际 ${#EMITTED[@]} 个：${EMITTED[*]:-<none>}" >&2
    exit 1
fi

mkdir -p "$OUT_DIR"
mv "${EMITTED[0]}" "$DEST"

echo "==> ${DEST} ($(wc -l <"$DEST") lines)"
echo "    记入 OBSERVATIONS.md 时 MUST 标注 [NON-ASSERTION]。"
