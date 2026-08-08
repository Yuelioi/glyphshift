# Desktop Acquisition Session

Status: Finished

## Outcome

Desktop Runtime 提供一次交互式取词 Session：从当前授权 Target 解析 opaque target 与短期 Controller
grant，按 Adapter ID 使用已验证 Bundle Host 完成一个 Point Acquisition request，并在成功、拒绝、
超时、取消或目标退出后确定回收 Worker。Shell 只提交产品级选择并接收有界结果。

## Scope

- 在 Desktop Runtime 内组合当前 Target 选择、Controller grant 与 `RuntimeBundle` Host factory。
- 固定一次请求一次 Worker 的生命周期、取消传播、目标失效和稳定错误映射。
- 先用确定性 Controller/Worker transport 合同覆盖 Point 请求；保持未来 Text Range / Region 扩展位。
- 为后续 Tauri command 留下窄 Interface，但本切片不让 command 接触 PID、创建时间、grant payload、
  executable 路径或任意 Worker 错误字符串。

## Non-goals

- 不注册全局快捷键，不实现取点交互、浮层、弹窗、剪贴板或设置页面。
- 不选择生产 OCR、在线 Translation Provider 或外部 Presenter。
- 不扩大 Dictionary、持续 Observation、Workflow 或 `TextReplace` 的职责。

## Verification

- [x] 正常 Point 请求只向所选授权 Target 签发短期 grant，并通过 Bundle Host 返回有界结果。
- [x] 未选择目标、未知 Adapter、权限拒绝、目标退出、超时、取消与 Worker 崩溃具有稳定产品错误。
- [x] 每个终态都回收 Worker；重复请求不复用过期 target/grant 或泄漏子进程。
- [x] Shell 可见 Interface 不包含 PID、平台 grant、制品路径或底层 Worker 字符串。
- [x] Workspace test、Clippy、fmt、architecture checks 与代码自审通过。

## Review

- `DesktopRuntime::acquire_point` 是当前唯一产品 Interface：调用者只提交 target ID、Adapter ID、虚拟
  桌面 Point 与取消句柄，并收到有界 `AcquisitionResult` 或 `DesktopAcquisitionError`。Controller token、
  PID、创建时间、grant payload、Worker 路径和任意底层字符串均留在 Runtime 内。
- 每次调用都重新执行 `authorize_worker_target`；确定性合同连续请求得到两个不同短期 grant，未知目标与
  预取消在授权和 Worker 之前失败。当前只允许 Discovered/Inactive Runtime，Active 状态明确返回
  `InvalidState`；与 Pool 中现有 Workflow/Capture 的共存策略留给下一切片，不隐式抢占已有 Session。
- Worker Binding Interface 不再要求调用者依赖 wire SDK 的 grant 类型，只接收 opaque platform/payload
  并在 Host 内验证。生产 Catalog 是唯一进程 Adapter，测试使用私有内存 Adapter，不扩大公开 Interface。
- Desktop Runtime 20 个常规合同、Acquisition Host 6 个进程合同、真实 UIA 5 个常规 Windows 合同通过；
  本地正式 Debug Bundle 合同实际启动受验证 UIA Worker，并从合成前台控件取得预期 Point 文本。
- 60 包 workspace 测试、workspace Clippy `-D warnings`、fmt、architecture checks 与 diff whitespace
  检查均通过；本机 Bundle 与证据只保存在 `local-test/`。

## Next

进入 [Desktop Acquisition Command](interactive-desktop-acquisition-command.md)：先把 Session 接到
`DesktopRuntimePool` 与窄 Tauri command，并明确它与现有 Workflow/Capture Session 的共存策略；之后
再决定快捷键、取点状态、外部呈现和错误反馈。若需要 Region/OCR，必须先有授权真实目标证明结构化
UIA 的覆盖缺口。
