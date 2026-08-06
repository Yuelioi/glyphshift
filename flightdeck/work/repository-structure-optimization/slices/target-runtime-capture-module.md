# Target Runtime Capture module

Status: Complete

## Goal

保持 Target Runtime Native ABI、全局原子激活/更新/停止、publication identity 与 observation drain 不变，
只提取同时支持 File/Batch 两种所有权的 Capture 生命周期，并迁移对应内嵌测试。

## Baseline

- `src/lib.rs` 966 行，其中 63-164 行是 RuntimeCaptureConfiguration/RuntimeCapture，904-966 行为 1 项测试。
- 其余约 800 行共同维护单个全局 RuntimeState、Native Host 回调、Kernel、Adapter set 与导出 ABI；拆散会
  增加原子切换和 unsafe 边界的跨模块耦合。
- 公开合同另有 1 项确定性和 3 项授权 Native DLL 测试。

## Plan

- [x] 提取 Runtime Capture owner：File/Batch start、observe、pause、drain 与 finish。
- [x] 根保留 RuntimeState、Native ABI、activation/update/query/deactivate 和 publication 校验。
- [x] 迁移 1 项 Batch-only drain 内嵌测试。
- [x] 运行本 crate/直接下游、Clippy、格式、架构和 diff。
- [x] Standards/Spec 自审 ABI/export、原子状态、Capture owner 与 publication identity。

## Result

- 根 `lib.rs` 从 966 行降至 803 行，继续完整拥有 RuntimeState、Native Host/Kernel 与全部导出 ABI。
- `capture.rs` 111 行，独立拥有 File/Batch 配置、ingress/producer/sink、pause/drain/finish。
- 1 项内嵌测试迁为 60 行 `tests.rs`，仍从 activate/decide/query/control/deactivate 公开路径验证。

验证通过：1/1 内部、1/1 确定性公开合同；3 项授权 Native DLL 合同保持 ignored；package Clippy
`-D warnings`（含 Windows Host 编译）、Rust 格式、架构检查与 `git diff --check` 通过。本 workspace 没有
以 `glyphshift-target-runtime` 为 Cargo 依赖的直接下游；它通过构建产物被 Host 加载。

Standards 自审确认 Capture 是唯一新增 owner，Native ABI/unsafe/global Mutex 未被分散。Spec 自审将
Capture 去除 crate 内可见性和 rustfmt 签名换行后、以及根 Runtime 主体，分别与 `HEAD` 对应区段比较
一致；公开导出函数/类型多重集一致，publication identity、原子替换、观察 drain 与 deactivate 顺序由
原测试保持覆盖。

## Decisions

- Capture 是独立资源所有者；Native ABI 与 RuntimeState 是同一原子激活 seam，保持合并。
- 不按导出函数逐文件拆分，避免把同一全局 Mutex 状态隐藏在多个浅 facade 后。
