# Desktop Acquisition Command

Status: Complete

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

- [x] Inactive 软件可从 Pool 发起 Point Acquisition，Command 只返回有界产品结果。
- [x] Workflow/Capture active 时行为明确且不会停止、替换或污染现有 Session。
- [x] 未知软件/目标/Adapter、重复提交、取消、目标退出和 App 退出具有稳定清理合同。
- [x] Command payload、日志和前端类型不包含 PID、grant、制品路径或 Worker 字符串。
- [x] 相关 Runtime/Shell 合同、Clippy、fmt、architecture checks、TypeScript 与代码自审通过；本 Slice
  没有 GUI surface，Playwright 不适用。依照仓库验证纪律未运行不必要的全仓测试。

## Next

Command 完成后再按现有桌面视觉系统设计交互入口，决定快捷键、取点状态、外部译文呈现和错误反馈；
GUI 变更必须通过仓库 Playwright CLI 验证。

## Progress

- Quick Probe 完成后进入本 Slice。当前先审计 `DesktopRuntimePool` 的 Session ownership 与既有
  `DesktopAcquisitionSession::acquire_point` Interface，固定共存和取消边界；开发循环只运行 Runtime
  Desktop 与 Desktop Shell 的相关合同，不默认运行全仓测试。
- `DesktopRuntimePool::acquire_point` 每次从 factory 建立独立、短期 Runtime，不插入 `sessions`，因此
  inactive 不残留状态，Workflow/Capture active 也不会复用、停止或替换既有 Controller Session。
- Shell 新增 `desktop_acquire_point` 与显式 cancel command：Backend 编译当前 Runtime spec，
  `spawn_blocking` 执行有界请求；独立注册表固定 cancellation ID 去重、取消、完成回收与 App state
  drop 时的全量取消。
- Command 结果只序列化有界原文、anchor、granularity、provenance 和 confidence；错误只返回稳定的
  本地化 code。前端已增加纯类型/调用封装，Desktop API 升至 21，未增加热键、鼠标或 UI 状态。
- Slice 自审确认：一次性 Runtime 只共享 Pool factory/nonce ownership，不读写持久 Session maps；命令
  持有应用锁期间 cancellation 仍走独立 state，可取消正在执行或等待锁的请求。Runtime acquisition
  合同 10/10、Shell acquisition 合同 7/7、两 crate Clippy、fmt、Architecture Gate 与 `vue-tsc`
  通过，`git diff --check` 无错误。
- 剩余真实验证归入下一产品 Slice：当前还没有用户取点入口或 target 选择状态，因此不在本 Slice
  伪造 Playwright/实机 UIA command smoke；下一步用真实产品入口贯通标准 Win32、WPF/WinUI 目标。
