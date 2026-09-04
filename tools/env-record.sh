#!/usr/bin/env bash
# 生成环境记录块（data-model.md §9 / FR-010）。
#
# 输出格式 MUST 与 rf_harness::env::EnvironmentRecord::to_markdown() 逐字一致。
# 修改本脚本时同步修改 harness/src/env.rs，反之亦然。
#
# 用法：
#   tools/env-record.sh                                  # 默认 host target
#   tools/env-record.sh --target x86_64-unknown-none     # 指定 target
#   tools/env-record.sh --command 'cargo test -p m1-ownership'
set -euo pipefail

TARGET="x86_64-unknown-linux-gnu"
COMMAND=""

usage() {
    sed -n '2,${/^#/!q;s/^# \{0,1\}//;p}' "$0"
    exit "${1:-0}"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target)  TARGET="$2"; shift 2 ;;
        --command) COMMAND="$2"; shift 2 ;;
        -h|--help) usage 0 ;;
        *) echo "unknown argument: $1" >&2; usage 1 ;;
    esac
done

# rustc -Vv 的 release 行与 commit-hash/commit-date 拼成 data-model §9 的示例格式。
rustc_version() {
    local out
    out="$("$@" -Vv 2>/dev/null)" || return 1
    local release hash date
    release="$(sed -n 's/^release: //p' <<<"$out")"
    hash="$(sed -n 's/^commit-hash: //p' <<<"$out")"
    date="$(sed -n 's/^commit-date: //p' <<<"$out")"
    printf '%s (%s %s)' "$release" "${hash:0:9}" "$date"
}

STABLE="$(rustc_version rustc || echo 'UNAVAILABLE')"
# nightly 是分析工具链，缺失时记 n/a 而非报错 —— 它不参与稳定断言（R-01）。
NIGHTLY="$(rustc_version rustup run nightly rustc 2>/dev/null || echo 'n/a')"

cat <<EOF
## 环境记录

| 字段 | 值 |
|------|-----|
| \`rustc_stable\` | $STABLE |
| \`rustc_nightly\` | $NIGHTLY |
| \`edition\` | 2024 |
| \`kernel\` | $(uname -r) |
| \`arch\` | $(uname -m) |
| \`target\` | $TARGET |
| \`command\` | ${COMMAND:-（按各记录块内的命令为准）} |
EOF
