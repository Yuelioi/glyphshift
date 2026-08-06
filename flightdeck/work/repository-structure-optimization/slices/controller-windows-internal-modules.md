# Controller Windows 内部 module

Status: Complete

## Goal

保持 ControllerPlugin wire 行为、进程家族 token 稳定性、路径授权、Runtime hash 校验、Windows 提权与
远程 Runtime 调用不变，把公开 executable 能力、Controller 状态机、底层进程平台查询和内嵌测试分离。

## Baseline

- `src/lib.rs` 1284 行，其中生产 1040 行、内嵌测试 244 行；`remote.rs` 已独立拥有注入/远程调用。
- 前 238 行是公开 Windows executable/提权能力；239-774 行是 Controller 状态与 Plugin interface；
  775-1040 行是 hash、路径、PE 与系统进程查询。
- 8 项单元测试覆盖进程家族稳定 token/PID 复用、根选择、PE、提权查询/启动/参数引用。

## Plan

- [x] 提取 executable/elevation 公开能力，底层平台查询保持 crate 内可见。
- [x] 提取 Controller 状态机与 Plugin interface，保持 target/runtime 所有权和 wire 错误。
- [x] 提取 platform：hash、路径规范化、ancestor、PE 与系统进程查询。
- [x] 按 controller/platform/elevation seam 拆分 8 项内嵌测试。
- [x] 运行单元/合同/下游回归、Clippy、格式、架构和 diff。
- [x] Standards/Spec 自审公开 API、token/PID-reuse、路径/hash、PE 与提权语义。

## Result

- 根 `lib.rs` 从 1284 行降为 17 行，只保留私有 module、Windows `remote` seam 和稳定 re-export。
- `executable.rs` 227 行：公开 executable inspection、foreground、elevation query/launch 和参数引用。
- `controller.rs` 556 行：进程家族授权、稳定 token、Runtime ownership 与完整 ControllerPlugin interface。
- `platform.rs` 270 行：SHA-256、路径规范化/授权祖先、PE 与系统进程事实查询。
- 8 项内嵌测试拆为共享 fixture 69 行、Controller 130 行、Platform 18 行、Elevation 38 行。

验证通过：8/8 单元、4/4 Target Process Hook、6/6 确定性 Inventory；1 项授权实机 Inventory 正常
ignored；唯一直接下游 Desktop Shell 44/44；package Clippy `-D warnings`、Rust 格式、架构检查与
`git diff --check` 全部通过。

Standards 自审确认 `remote.rs` 的 unsafe/注入职责未移动，所有新文件低于 1200 行，跨 module 仅开放
crate 内事实/fixture 接口。Spec 自审将 Executable、Controller、Platform 三段规范化后与 `HEAD` 逐段
比较一致，公开类型/函数多重集一致；token/PID reuse、根选择、路径/hash、PE、提权 mask/quoting 与
runtime/capture wire 行为由 18 项确定性测试覆盖。

## Decisions

- `remote.rs` 保持独立部署/unsafe seam，不与系统进程枚举混合。
- Controller 拥有授权目标、token、runtime library 与 publication；platform 只返回事实，不保存会话状态。
- 不扩展公开 API；跨 module 辅助仅使用 `pub(super)`。
