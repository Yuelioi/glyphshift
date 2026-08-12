# Qt 静态文字实时预览刷新

Status: Finished

## Goal

让已连接的动态 Qt Widgets 目标在探针译文变更后立即显示新一代译文；不要求目标重启，不把框架或
软件特例泄漏到 Core、Desktop 或 GUI，并继续满足停用恢复与失败开放语义。

## Current

用户截图暴露的产品链路缺口已修复：临时快速探测此前固定关闭实时预览，即使 Runtime 报告可直接替换也
不会发布 Translation Snapshot。快速探测现在会在实际 Runtime 能力确认为 `DirectReplace` 后自动开启
预览；采集型 Runtime 仍保持关闭。Qt 刷新 seam 继续负责让已发布的新代次立即重绘，并在停用后恢复。

## Next

None

## Progress

- 已用隔离真实目标连续三次复现“内部发布成功、静态文字像素零变化”。
- 已用同目标的交互式 placeholder 建立配对绿灯，排除 Dictionary、代次发布与全部 Qt 替换失效。
- 已验证主题刷新等额外 Windows 消息会引发 Qt 绘制活动，但仍不会让静态 `Name` 进入现有四个
  `drawText` hook；单纯加强通用 `RedrawWindow` 不是充分修复。
- 合成 Native Adapter 合同精确验证激活、发布与停用各触发一次刷新；真实 Qt Widgets 合同不再手工触发
  `QLabel::repaint()`，只依靠 Adapter 请求验证同一静态 `QLabel` 连续两代重新进入绘制链。
- 受影响 crate 的 Clippy、定向格式检查、空白检查和公共 ABI 全仓门禁通过。
- 隔离真实目标中首代与第二代的目标区域变化均超过 26%，停用后与原文基线零像素差。
- 用户真实桌面截图推翻了“任务完成”结论：词典输入已保存，但 `Assets` 与 placeholder 仍为原文；该场景
  是当前权威红灯，先前静态 `Name` 像素合同仅保留为底层局部证据。
- 桌面探针数据确认失败任务始终为预览关闭、generation 0，根因是快速探测请求没有发布任何 Snapshot，
  而不是 Qt 采集或词典保存失败。
- 新增两条桌面 Shell 合同：可写回的自动快速探测在启动时发布 G1、编辑后发布 G2；只有采集能力时保持
  预览关闭且不发布。
- 仓库 Playwright 通过真实桌面命令与探针编辑界面驱动隔离目标：两个原始控件在 G3 可见替换，静态标题
  在 G4 再次变化，清理后两个区域均与原文基线零像素差。
- 桌面 Shell 67 个测试与探针页面 16 个 Playwright 测试通过；定向格式检查通过。严格 Clippy 仅被当前
  仓库另一处快捷键回调的既有 `collapsible_if` 警告阻断，允许该单项既有 lint 后其余检查通过。
- 已生成不依赖开发服务器的最新本地桌面构建并确认窗口正常渲染；隔离测试目标已关闭，用户原始目标未改动。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
- [通用实时翻译覆盖历史](../real-time-translation-coverage/index.md)
