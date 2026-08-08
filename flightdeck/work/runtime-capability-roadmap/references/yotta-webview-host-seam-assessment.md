# Yotta WebView 宿主接入评估

Status: Complete

## 结论

Yotta 应替代外部软件成为 GlyphShift 首个真实 Web Desktop 验收目标。它是自有的 Wails 3 + WebView2
桌面工具，普通菜单、设置、表单与弹窗足够验证 DOM 翻译，同时宿主接口、权限和生命周期都可以由双方
共同收敛，不需要借用第三方软件的高权限开发接口。

当前已打开实例不能直接连接：它没有 remote debugging port/pipe 参数，WebView2 进程没有 TCP Listen；
此外目标以更高完整性运行，当前 GlyphShift 进程无法读取其完整根进程信息。保持这个安全默认，不扫描、
注入或提升权限绕过。

后续诊断确认这是两个独立边界：

- **Web Session 的直接原因是没有 endpoint。** WebView2 调试参数必须在创建 WebView 前配置；production
  构建不仅没有配置参数，还通过 build tag 移除了调试选项。单纯把 GlyphShift 提权不会产生 endpoint。
- **UAC 是 Native/UIA 的第二个阻断。** Yotta production manifest 固定 `requireAdministrator`，标准
  GlyphShift/Worker 是中完整性；低完整性主体对高完整性进程的读取/写入受 Windows MIC 约束，因此
  根进程信息、UIA 与注入必须失败关闭。

## 已有测试 seam

- Windows 非 production 构建可通过环境配置独立 WebView profile 和 remote debugging port。
- DevTools 快捷键只存在于非 production 构建，production 明确移除。
- Storage Root 可通过环境变量覆盖，且单实例身份按 Storage Root 派生；因此可以启动一个不接管当前实例、
  不读取正式数据的隔离测试实例。
- 当前调试 seam 使用本地端口，只能用于一次性本地验收；此前无凭据第二客户端合同已证明它不能成为
  GlyphShift 的生产 transport。

## 诊断证据

- Yotta 自带的 manifest 合同证明 production 为 `requireAdministrator`、development 为 `asInvoker`；
  两项专项测试通过。
- Windows 非 production WebView 配置合同证明只有合法的绝对 profile 和显式端口会进入 Wails options；
  两项专项测试通过。
- GlyphShift 本地诊断以同一套 Yotta 源码构建无 manifest 开发 Host，使用隔离 Storage Root/Profile
  隐藏启动；两轮均得到 6/6：构建、两类隔离、启动前参数、CDP endpoint 与 page target 全部通过。
- 授权 production 实例运行时曾确认一个 WebView2 子进程，但无调试端口/pipe 参数、无进程级 TCP
  Listen，根进程路径与命令行对中完整性诊断不可读。目标随后退出，因此未请求重启或提权复测。

## 推荐验证方式

在用户明确允许启动第二个隔离实例后：

1. 调试构建产物、Storage Root、WebView profile、协议输出和截图全部放在 `local-test/`。
2. 动态选择本地端口，仅在测试进程生命周期内使用，不写入仓库、日志或持久配置。
3. 先只读确认 WebView target、普通 DOM 范围和权限；排除 CodeMirror、用户输入、日志正文及工作流内容。
4. 复用已验证的 DOM ownership 状态机完成两代字典、应用重渲染与条件恢复。
5. 关闭测试实例后确认 endpoint 和进程全部退出，不影响用户当前 Yotta 实例。

## 生产方向

真实验收通过后，也不把调试端口包装成正式 Adapter。更合适的产品 seam 是 Yotta 宿主主动暴露一个
版本化、最小权限的 Translation Host 接口：只允许固定脚本观察/更新普通界面文本，传递有界
`source + translation + generation` 数据，并由宿主拥有窗口、WebView、导航和停用恢复生命周期。
该接口属于后续跨仓库设计，不在本次诊断中实现。

## Stop conditions

- 隔离 Storage Root 或 WebView profile 未生效，第二实例接管了当前实例。
- 需要读取正式设置、工作流、日志、用户输入或其他业务数据才能验证界面文本。
- 需要关闭、重启或提升当前用户实例，或需要把调试端口保留为生产能力。

命中任一条件即停止，不扩大权限。

## 一手资料

- [WebView2 browser flags](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags)
- [Windows Mandatory Integrity Control](https://learn.microsoft.com/en-us/windows/win32/secauthz/mandatory-integrity-control)
