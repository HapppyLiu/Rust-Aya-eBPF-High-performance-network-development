# Contract: linux-harness (`lf-harness`) API

**Feature**: 002-linux-foundation

本 crate 位于 `linux-harness/`，package 名 `lf-harness`。零外部依赖。
MUST NOT 包含学习目标代码（不得实现“迷你 TCP 栈”或“自己的 VFS”）。

---

## `host`

```rust
pub enum HostKind { UbuntuVm, Wsl2, Docker, Other }
pub fn host_kind() -> HostKind; // 读 LF_HOST_KIND，默认按 /proc/sys/kernel/osrelease 启发式
pub fn is_acceptance_host() -> bool; // UbuntuVm
```

验收断言（`accepted`）只应在 `is_acceptance_host() == true` 时把工具缺失当失败。

---

## `tool_probe`

```rust
pub fn have(cmd: &str) -> bool;
pub fn path(cmd: &str) -> Option<PathBuf>;
pub fn require(cmd: &str); // 验收宿主上缺失则 panic；偏离宿主返回
```

---

## `strace`

```rust
pub fn run(program: &Path, args: &[&str], extra: &[&str]) -> std::io::Result<String>;
pub fn syscall_names(stderr_text: &str) -> BTreeSet<String>;
```

`run` 以 `strace -f ... -- program` 执行。`syscall_names` 只提取名称。MUST NOT 解析 PID 列。

---

## `proc`

辅助读取 `/proc/self/status`、`maps`、`stat` 中的**命名字段**。
调用方只对方向或存在性断言。

---

## `env`

环境记录 markdown，与 `tools/env-record.sh` 兼容，并追加：

```text
- host_kind: ubuntu-vm
- kernel_series: v6.8
- tools: strace=yes perf=yes gdb=yes bpftrace=yes ip=yes ss=yes lsns=yes tcpdump=yes
- privilege: root
```

可委托 `rf_harness::env` 再拼接 Linux 字段。

---

## 非目标

- 不封装 `mmap` / `socket` / `epoll` 的“安全 API”——学习者必须直接面对 syscall。
- 不安装缺失工具。
- 不把 skip 转换成 Ok(true)。
- 不在 harness 里创建长期 netns。
