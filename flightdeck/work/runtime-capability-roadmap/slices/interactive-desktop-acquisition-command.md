# Desktop Acquisition Command

Status: Planned

## Outcome

Desktop Shell 可通过一个窄 Tauri command 发起 Point Acquisition；`DesktopRuntimePool` 负责取得或创建
正确的短期 Runtime，调用已完成的 Session Interface，并返回不含平台身份的产品结果。命令不会抢占、
停止或复用现有 Workflow / Capture 的 Controller Session。

## Scope

- 在 `DesktopRuntimePool` 内增加一次性 Acquisition 路由，明确 inactive、Workflow active 与 Capture
  active 三种状态的共存策略。
- Command 只接收 software ID、target ID、Adapter ID、Point 和产品取消标识；Backend 负责编译当前
  Runtime spec，Shell 不接触 executable 路径、Controller token 或 grant。
- 用 `spawn_blocking` 或等价有界执行避免阻塞 Tauri async 线程，并固定重复提交、取消和 App 退出清理。
- 将 `DesktopAcquisitionError` 映射为稳定、可本地化的 command error code，不上送 Worker 字符串。

## Non-goals

- 不注册全局快捷键，不捕获鼠标，不实现浮层、剪贴板、Translation Provider 或 Presenter。
- 不增加 Text Range / Region / OCR command，不让 Dictionary 或 Workflow 承担 Point 状态。
- 不为并发方便复制 Controller token、grant 或可执行文件路径到 Shell state。

## Verification

- [ ] Inactive 软件可从 Pool 发起 Point Acquisition，Command 只返回有界产品结果。
- [ ] Workflow/Capture active 时行为明确且不会停止、替换或污染现有 Session。
- [ ] 未知软件/目标/Adapter、重复提交、取消、目标退出和 App 退出具有稳定清理合同。
- [ ] Command payload、日志和前端类型不包含 PID、grant、制品路径或 Worker 字符串。
- [ ] Workspace test、Clippy、fmt、architecture checks、Playwright CLI 与代码自审通过。

## Next

Command 完成后再按现有桌面视觉系统设计交互入口，决定快捷键、取点状态、外部译文呈现和错误反馈；
GUI 变更必须通过仓库 Playwright CLI 验证。
