#!/usr/bin/env bash
# US7 / SC-006 三步递进：每步单独构建，核对计入分母的那一条错误是否出现。
#
# 不比对诊断全文（experiment-contract §C2.2 / §C4：断言错误码或稳定子串）。
# 分母固定为 6，见 spec.md SC-006。
#
# 用法（仓库根目录）：
#   tools/m7-probe-errors.sh           # 跑全部 6 条
#   tools/m7-probe-errors.sh 1a        # 只跑一条
#   tools/m7-probe-errors.sh 1a 3b
#
# 退出码：0 全部核对通过（cargo 在探测路径上 MUST 失败且命中期望子串）/ 非 0 未通过。
set -euo pipefail

usage() { sed -n '2,${/^#/!q;s/^# \{0,1\}//;p}' "$0"; exit "${1:-0}"; }
[[ "${1:-}" == "-h" || "${1:-}" == "--help" ]] && usage 0

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
M7="$ROOT/experiments/m7-nostd"
export CARGO_TARGET_DIR="$M7/target"

# id | 归属（文档用，脚本不断言归属）| cargo 参数描述 | 期望子串
run_probe() {
    local id="$1"
    local log rc
    log="$(mktemp)"
    rc=0
    case "$id" in
        1a)
            ( cd "$M7" && cargo build --features probe-no-panic-handler --offline ) \
                >"$log" 2>&1 || rc=$?
            expect="#[panic_handler]"
            expect2="not found"
            ;;
        1b)
            # 保留 panic_handler，强制 unwind：1.98.0 上不会索要 eh_personality。
            ( cd "$M7" && RUSTFLAGS="${RUSTFLAGS:-} -C panic=unwind" cargo build --offline ) \
                >"$log" 2>&1 || rc=$?
            expect="unwinding panics are not supported without std"
            expect2=""
            ;;
        2a)
            ( cd "$M7" && cargo build --features probe-std-type --offline ) \
                >"$log" 2>&1 || rc=$?
            expect="cannot find \`fs\` in \`core\`"
            expect2="E0433"
            ;;
        2b)
            ( cd "$M7" && cargo build --features probe-std-crate --offline ) \
                >"$log" 2>&1 || rc=$?
            expect="can't find crate for \`std\`"
            expect2="E0463"
            ;;
        3a)
            ( cd "$M7" && cargo build --features probe-vec-no-alloc-crate --offline ) \
                >"$log" 2>&1 || rc=$?
            expect="cannot find module or crate \`alloc\`"
            expect2="E0433"
            ;;
        3b)
            ( cd "$M7" && cargo build --features probe-vec-no-global-alloc --offline ) \
                >"$log" 2>&1 || rc=$?
            expect="no global memory allocator found"
            expect2=""
            ;;
        *)
            echo "unknown probe: $id" >&2
            rm -f "$log"
            return 2
            ;;
    esac

    echo "==> probe $id"
    if [[ "$rc" -eq 0 ]]; then
        echo "  FAIL: cargo build 竟然成功了（探测路径必须失败）" >&2
        tail -n 20 "$log" >&2
        rm -f "$log"
        return 1
    fi
    if ! grep -Fq "$expect" "$log"; then
        echo "  FAIL: 未命中期望子串: $expect" >&2
        tail -n 40 "$log" >&2
        rm -f "$log"
        return 1
    fi
    if [[ -n "$expect2" ]] && ! grep -Fq "$expect2" "$log"; then
        echo "  FAIL: 未命中第二子串: $expect2" >&2
        tail -n 40 "$log" >&2
        rm -f "$log"
        return 1
    fi
    echo "  PASS: cargo 失败且命中稳定子串（exit=$rc）"
    rm -f "$log"
    return 0
}

ids=("$@")
if [[ "${#ids[@]}" -eq 0 ]]; then
    ids=(1a 1b 2a 2b 3a 3b)
fi

fail=0
for id in "${ids[@]}"; do
    if ! run_probe "$id"; then
        fail=1
    fi
done

if [[ "$fail" -eq 0 ]]; then
    echo
    echo "==> 退出码 0：SC-006 六条探测均按清单复现。"
    exit 0
fi
echo
echo "==> 退出码 1：至少一条探测未按清单复现。" >&2
exit 1
