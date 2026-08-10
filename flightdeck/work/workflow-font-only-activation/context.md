# 稳定上下文

## 产品语义

- Adapter 是否属于当前产品目录由 Runtime Bundle 与 Desktop Environment 决定。
- 有效工作流只请求实际需要的能力；字体专用目标可以只请求 `FontSubstitute`。
- 请求能力集合不能反向用作 Adapter 是否存在或是否属于产品目录的证据。

## 约束

- 观察器等已退役 Adapter 仍须由工作流解析阶段以 `UnknownAdapter` 拒绝。
- 回归必须经过 `DesktopApplication::enable_workflow`，不能只测字符串或前端文案。
- 本机工作流与 Runtime 证据只留在 `local-test/`。
