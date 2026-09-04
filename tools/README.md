# tools/ —— 脚本

统一封装实验中反复使用的命令，使 OBSERVATIONS 中记录的命令与实际执行的命令一致。

| 脚本 | 用途 | 契约依据 |
|-----|------|---------|
| `env-record.sh` | 生成环境记录块（data-model §9 全部字段） | FR-010 / §C7.1 |
| `emit-mir.sh` | `cargo rustc -- --emit=mir`，输出到 `target/ir/` | R-04 阶梯 3 |
| `emit-llvm-ir.sh` | `cargo rustc -- --emit=llvm-ir`，输出到 `target/ir/` | R-04 阶梯 4 |
| `run-miri.sh` | 统一 `MIRIFLAGS` 的 UB 判定入口 | R-02 / FR-019 |
| `run-asan.sh` | FFI 场景的 UB 判定入口（Miri 不支持真实 C 调用） | R-02 / §C5.4 |
| `check-nostd-artifact.sh` | `no_std` 产物的符号与节区静态检查（退出码判据） | R-04 阶梯 6 / §D2 |

**约定**：所有脚本从仓库根目录执行，接受 `-h` 打印用法，用退出码表达判定结果
（0 = 通过 / 非 0 = 未通过），使它们可以直接作为 `acceptance/criteria/` 的退出码判据。

`env-record.sh` 的输出格式 MUST 与 `rf_harness::env::EnvironmentRecord::to_markdown()`
逐字一致 —— 两者是同一份环境记录的 shell 与 Rust 实现，`no_std` 构建等不便启动 cargo
的场合用前者。
