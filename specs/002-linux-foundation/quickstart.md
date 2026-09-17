# Quickstart: Linux Foundation

本文件是**验证指南**，不是教程。完整学习从 `learner/002-linux-foundation/` 进入。
每个模块可单独验证；实验小、独立、可运行、可观察。

最终 `accepted` 只在 **Ubuntu x86_64 虚拟机** 上判定。WSL2 / Docker 最多打到
`experiment-passed`。

## 0. 验收 VM 准备

```bash
# Ubuntu VM，x86_64，完整 6.x 内核
sudo apt-get update
sudo apt-get install -y build-essential clang gdb strace \
  linux-tools-$(uname -r) bpftrace iproute2 tcpdump linux-headers-$(uname -r)

uname -r          # 记录主.次版本 → KERNEL_SERIES=vX.Y
rustc --version   # 期望 1.98.0（仓库 rust-toolchain.toml）

export LF_HOST_KIND=ubuntu-vm
./tools/linux-tool-probe.sh
./tools/env-record.sh --command 'baseline 002-linux-foundation'
```

把探测结果存为 `acceptance/002-linux-foundation/tool-probe.md`。
基准 VM 上 `strace` `gdb` `perf` `bpftrace` `ip` `ss` `tcpdump` MUST 为 yes。

内核源码：设置 `KERNEL_SRC` 指向与 `uname -r` 主.次同系列的 linux 树，或在
`source-refs.md` 使用对应 `vX.Y` 的 tree-url。

偏离宿主：

```bash
export LF_HOST_KIND=wsl2    # 或 docker
# 允许等价观察；不得把 C-04/C-06/C-26 标为 accepted
```

## 1. 模块独立验证（推荐）

从仓库根，一次只验证一个模块：

```bash
./tools/lf-verify-module.sh m1-cli-observe
./tools/lf-verify-module.sh m2-process-syscall
./tools/lf-verify-module.sh m3-memory
./tools/lf-verify-module.sh m4-vfs-io
./tools/lf-verify-module.sh m5-sched-ipc
./tools/lf-verify-module.sh m6-net
./tools/lf-verify-module.sh m7-capstone
```

每个命令 MUST 只构建 `lf-harness` + 该 crate，MUST NOT 要求其它模块的 netns 或后台进程。
全部退出码 0 是该模块 Track B 稳定断言通过的口径。

也可：

```bash
cargo test -p m1-cli-observe -- --test-threads=1
```

001 的 crate 仍在同一 workspace，互不替代。

## 2. 先预测后验证（双轨）

1. 只打开 `learner/002-linux-foundation/mN-*/guide.md`、`predictions.md`、`selfcheck.md`、`quiz.md`。
2. 填写预测并保存。
3. 运行该模块 `lf-verify-module.sh` 或对应 `cNN` test。
4. 卡住后再打开 `learning/` 与 `feynman/`。

## 3. 观测工具（每条都是独立最小实验）

```bash
# C-03 普通用户
strace -e write,openat,close -- cargo run -p m1-cli-observe --example c03_strace

# C-04 root（基准 VM）
sudo perf stat -e cycles,instructions -- sleep 0.05

# C-05 普通用户
gdb -batch -ex 'bt' -ex 'quit' --args true

# C-06 root；观察，不是 eBPF 课
sudo bpftrace -e 'BEGIN { printf("ok\n"); exit(); }'
```

## 4. 网络

```bash
# C-22…C-25：回环即可，不需要 netns
ip link show lo
ss -ltn
tcpdump -i lo -c 1 -n &  # 仅作观察，计数绝对值非断言

# C-26：仅此能力必须创建两个 ns；必须 root；必须删除
sudo ip netns add lf-c26-a
sudo ip netns add lf-c26-b
sudo ip netns exec lf-c26-a ip link
sudo ip netns del lf-c26-a
sudo ip netns del lf-c26-b
```

实现阶段的正式脚本是 `experiments/002-linux-foundation/m6-net/scripts/c26_netns.sh`，
MUST 自带 cleanup。不要依赖上面的手工残留。

## 5. 综合实验（US7）

```bash
./tools/lf-verify-module.sh m7-capstone
```

对照 `acceptance/002-linux-foundation/capability-matrix.md` 逐项定位体现位置。
capstone 自己 setup/cleanup。

## 6. 追溯审计

按 `contracts/002-linux-foundation/learning-artifact-contract.md` §E / §H 执行。
`comm` 两侧必须为空。

## 7. 本 Feature 明确不跑的东西

- `cargo aya` / `bpf-linker` / 自定义 `.bpf.c`
- XDP 加载、物理网卡卸载、io_uring、TCP 拥塞控制实验
- 任何“未测量但声称更快”的基准
- 把 WSL2 上的 skip 写成 `accepted`
