# Web Desktop 首个真实目标验收

Status: Deferred — host-assisted integration only

## Goal

在用户明确授权后，只使用自有目标的调试宿主接口和一次性隔离数据空间，验证 GlyphShift 的 Web
Desktop Session 是否能在真实工具界面完成普通 DOM 原文观察、两代字典热更新、应用重渲染重新决策与
条件恢复。

## Candidate

首选自有 Wails + WebView2 工具 Yotta。当前用户实例保持不动；验收使用独立 Storage Root、独立 WebView
profile 和非 production 调试构建启动第二实例。本地调试端口只作为一次性诊断 transport，不进入产品。
Obsidian 降为外部软件备选，draw.io Desktop 继续等待私有 transport 或独立 profile seam。

## Delivery

- [ ] 用户明确授权启动第二个隔离 Yotta 实例；当前实例不关闭、不重启、不连接。
- [x] 调试构建、Storage Root、WebView profile 和原始证据全部放入本地测试目录；隐藏 Host 两轮
  CDP endpoint/page target 控制合同通过。
- [ ] 先只读确认版本、WebView target、普通 DOM 范围和 renderer 生命周期，不读取工作流、日志或设置数据。
- [ ] 捕获至少 30 条导航、设置、表单或弹窗中的去重普通界面原文；CodeMirror、输入和业务正文不计。
- [ ] 完成首代替换、第二代热更新、应用重渲染重新决策和只恢复仍 owned 节点的闭环。
- [ ] 验证窗口关闭、renderer 重建和调试接口关闭均明确断线且不会误报 Healthy。
- [ ] 形成目标版本、宿主入口、观察/应用范围、失败语义和证据边界，不外推所有 WebView2 软件。

## Stop conditions

- 隔离 Storage Root/profile 未生效，或第二实例接管当前实例。
- 任何观察会触及正式设置、工作流、日志、用户输入、storage、cookie 或网络数据。
- 需要关闭/重启当前实例、修改正式安装文件、注入私有符号或提升权限。
- 应用重渲染与恢复合同无法闭合。

命中任一条件立即 No-Go 并返回 Roadmap，不用更高权限补洞。

## Current

连接原因已查清：production 目标无 endpoint，UAC 另行阻止 Native/UIA；提升 GlyphShift 不能修复 Web
Session。隔离开发 Host 已证明 WebView2 transport 可用，但继续完成自有目标 DOM 闭环只会证明宿主
协作能力，不能证明 GlyphShift 可以连接绝大多数未主动开放接口的第三方软件。本 Slice 因此暂停，不再
作为通用实时翻译 Adapter 的当前门槛；未来只在明确的官方扩展、受控启动或宿主 SDK 场景中恢复。

## References

- [首个真实目标候选](../references/web-desktop-first-target-candidates.md)
- [Yotta WebView 宿主接入评估](../references/yotta-webview-host-seam-assessment.md)
- [Web 桌面软件授权会话评审](../references/web-desktop-authorized-session-review.md)
- [Web Desktop Session 继承管道](web-desktop-session-pipe-diagnostic.md)
