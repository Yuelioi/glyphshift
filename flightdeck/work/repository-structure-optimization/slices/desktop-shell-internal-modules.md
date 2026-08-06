# 桌面 Tauri 壳内部 module

Status: Complete

## Goal

保持 crate 唯一公共 interface `run()`、所有 Tauri 命令名、Serde 字段、错误码和锁定语义不变，按功能
变化原因把桌面壳的 implementation 收进私有深 module，并让测试位于对应 interface 附近。

## Delivery

- [x] 提取 `font_catalog`：用加载与显式刷新两个入口隐藏缓存 schema、去重、文件读写和 Windows 注册表
  扫描；字体标签与缓存合同随 module 迁移。
- [x] 提取 Probe 纵向 module：请求/视图、`DesktopApplication` Probe 方法和 Tauri wrapper 共同归位，
  根 Builder 只引用命令路径。
- [x] 依次复核 Dictionary、Workflow 与 Software 功能面；共享 Runtime 展示映射只形成一个私有 module，
  不复制到各功能。
- [x] 将剩余根测试按被测 interface 迁移，避免形成另一个超过 1200 行的 `tests.rs`。
- [x] 完成桌面壳测试、Clippy `-D warnings`、格式、架构检查和受影响的 Playwright 命令合同回归。

## Current

`font_catalog`、`probe`、`dictionary`、`workflow` 与 `software` 已按功能变化原因形成私有
module；根 `lib.rs` 只保留共享应用状态、Runtime seam、启动/设置命令和 Builder 组合。快速捕获的
状态与 Windows 交互也由 Software module 隐藏，Builder 只调用初始化和事件入口。

内嵌测试已拆成 `tests/mod.rs` 的共享确定性夹具，以及 shell、Dictionary、Workflow、Probe、Software
五个场景文件。根文件从基线 4633 行降至 564 行；本 crate 最大文件为 762 行，未产生新的超长
`tests.rs`。

验证已通过：Desktop Shell 44/44、该 package 全 targets Clippy `-D warnings`、Rust 格式、架构检查、
`git diff --check` 和组件/命令合同 Playwright 6/6。45 个 Tauri 命令函数名与基线逐项一致，crate
公共 interface 仍只有 `run()`。

Standards 自审确认所有协作可见性不超过 `pub(super)`、机器专属证据未进入跟踪目录、所有 Rust 文件
低于 1200 行；并恢复了 Playwright 启动 Vite 时产生的本机依赖布局声明差异。Spec 自审确认命令名、
Serde/错误合同、锁定方式、Runtime 恢复顺序和产品行为均由既有测试保持。

## Decisions

- 采用“纵向功能 module + 小型平台 module”，拒绝把所有 DTO、命令与状态各放一个横向大文件。
- 私有协作最多使用 `pub(super)`；不为了迁移或测试扩大 crate 公共 interface。
- Tauri 命令 wrapper 与对应功能放在一起，但 Builder 仍集中列出全部固定命令，便于审计兼容面。
- 根测试按功能逐步迁移；不会先整体挪成单一超长 `tests.rs`。

## Next

进入 Desktop Backend interface 复核：先固定持久化映射、Dictionary、Workflow 与 Software catalog 的
调用关系，再建立不增加 facade 转发层的私有 module。

## Boundaries

- 不改变 Tauri 命令名、参数 JSON、返回 JSON、错误码、锁顺序和运行时恢复顺序。
- 不拆 `DesktopApplication` 成多个互相转发的状态对象；内部 module 通过同一状态实现功能。
- 不在本切片移动 crate 目录、改 package 名或调整 workspace 依赖。

## References

- [超长文件职责与拆分图](large-file-decomposition-map.md)
- [稳定上下文](../context.md)
- [阶段计划](../plan.md)
