# Adapter 技术文档链接

Status: Complete

## Goal

让用户能从桌面端直接了解每种 Adapter 所依赖的文字技术：正式清单提供可选的
`documentationUrl` 展示元数据，帮助页提供明确的“查看文档”入口，并在系统默认浏览器中安全打开
官方 HTTPS 文档。

## Delivery

- [x] 在 Native 与 Isolated Worker 清单、Runtime Adapter 选项和桌面快照中贯通可选
  `documentationUrl`；旧 Bundle 缺少字段时继续兼容，显式字段只接受无凭据、带主机名的绝对 HTTPS
  URL。
- [x] 为正式中文 Adapter 目录中的 10 个技术条目补齐官方文档链接，并由 Bundle 构建脚本验证、写入
  两类制品清单；正式 Debug Bundle 复核为 10/10 条目有文档且全部使用 HTTPS。
- [x] 帮助页增加独立“文档”列、中英文文案、可访问名称、加载和错误状态；不展示裸 URL，也不暴露
  内部制品位置。
- [x] 桌面端通过 Tauri Opener 使用系统默认浏览器，权限只允许当前官方文档域名；新文档域名必须先
  显式加入 Capability。
- [x] 完成桌面生产构建、完整 Rust workspace、全 workspace Clippy、架构检查、关键 Playwright 行为
  回归和两个窗口尺寸的视觉回归。

## Current

`documentationUrl` 已成为 Adapter 的可选展示元数据。当前正式目录 10 个条目均链接到 Microsoft、Qt、
GTK 或 raylib 官方资料；帮助页按 Adapter 行提供“查看文档”，桌面环境交给系统默认浏览器打开。旧
Bundle 可继续加载；缺少文档时界面显示“暂无文档”。

## Next

- 新增生产 Adapter 时同时填写官方 `documentationUrl`；若使用新的文档域名，同步扩展桌面 Opener
  Capability 和权限合同测试。
- 文档链接只解释技术入口，不作为实时写回覆盖或真实目标验收的替代证据。

## Boundaries

- 正式清单只接受 HTTPS 官方资料；不允许凭据、相对路径、自定义协议或任意外部域名。
- 前台不显示裸 URL、DLL 路径、内部 Adapter ID 或本机证据位置。
- 本机截图、日志与构建物仍只保存在忽略的 `target/local-test/` 下。

## References

- [产品契约](../../../../PRODUCT.md)
- [桌面设计契约](../../../../DESIGN.md)
- [raylib `DrawTextEx` 生产 Adapter](raylib-draw-text-adapter.md)
