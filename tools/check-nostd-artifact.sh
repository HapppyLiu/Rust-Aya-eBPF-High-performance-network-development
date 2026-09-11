#!/usr/bin/env bash
# no_std 产物的符号与节区静态检查（R-04 阶梯 6 / spec.md SC-002 第 3 项 / T116）。
#
# 断言载体是本脚本的退出码，不是运行期 #[test]（裸机产物不能在本机执行，CHK041）。
#
# 用法（仓库根目录）：
#   tools/check-nostd-artifact.sh
#
# 退出码：0 通过 / 非 0 未通过。
set -euo pipefail

usage() { sed -n '2,${/^#/!q;s/^# \{0,1\}//;p}' "$0"; exit "${1:-0}"; }
[[ "${1:-}" == "-h" || "${1:-}" == "--help" ]] && usage 0

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
M7="$ROOT/experiments/m7-nostd"
export CARGO_TARGET_DIR="$M7/target"
BIN="$CARGO_TARGET_DIR/x86_64-unknown-none/debug/m7-nostd"

echo "==> cargo build (m7-nostd, target pinned by .cargo/config.toml)"
( cd "$M7" && cargo build --offline )
echo

if [[ ! -x "$BIN" ]]; then
    echo "FAIL: 未找到产物 $BIN" >&2
    exit 1
fi

fail=0
nm_out="$(nm "$BIN")"
header="$(readelf -h "$BIN")"

has() { grep -Eq "$1" <<<"$2"; }

echo "==> nm 符号检查"
if has '(^| )[Tt] _start$' "$nm_out"; then
    echo "  PASS: 存在入口符号 _start"
else
    echo "  FAIL: 缺少 _start" >&2
    fail=1
fi
if has '__libc_start_main' "$nm_out"; then
    echo "  FAIL: 不应出现 __libc_start_main（那是 libc/OS 启动例程）" >&2
    fail=1
else
    echo "  PASS: 无 __libc_start_main"
fi
if has 'eh_personality' "$nm_out"; then
    echo "  FAIL: 不应出现 eh_personality（panic=abort 的裸机产物不该拉展开人格）" >&2
    fail=1
else
    echo "  PASS: 无 eh_personality"
fi

echo
echo "==> readelf -h 节区/文件头"
if grep -q 'Class:[[:space:]]*ELF64' <<<"$header"; then
    echo "  PASS: ELF64"
else
    echo "  FAIL: 不是 ELF64" >&2
    fail=1
fi
if grep -q 'Machine:[[:space:]]*Advanced Micro Devices X86-64' <<<"$header"; then
    echo "  PASS: Machine = X86-64"
else
    echo "  FAIL: Machine 不是 X86-64" >&2
    fail=1
fi
if grep -Eq 'Type:[[:space:]]*(EXEC|DYN)' <<<"$header"; then
    echo "  PASS: Type 为 EXEC 或 DYN（PIE）"
else
    echo "  FAIL: 不是可加载的 ELF 类型" >&2
    fail=1
fi

echo
if [[ "$fail" -eq 0 ]]; then
    echo "==> 退出码 0：m7-nostd 产物静态检查通过。"
    exit 0
fi
echo "==> 退出码 1：产物静态检查未通过。" >&2
exit 1
