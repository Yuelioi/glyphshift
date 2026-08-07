# 默认 OCR 与悬浮取词呈现

Status: Complete

## Outcome

默认开发与 Release Runtime Bundle 都包含已通过门禁的精简 Tesseract 支持集。一次性 UIA/OCR 取词
成功后，Glyphshift 在光标附近显示独立、无装饰、置顶的有界翻译气泡；主窗口继续保留完整结果和
显式 OCR 重试流程。

## Product boundary

- OCR 仍只能在同一软件与词典的 UIA `no_text` 后由用户显式选择，不自动截图。
- 气泡只持有当前内存结果，不写截图、原文、坐标、进程身份或历史记录。
- 默认聚焦离光标最近的结果块；用户可前后浏览、复制原文/译文，或用 Esc/关闭按钮收起。
- 气泡定位选择光标所在或最近的显示器，优先右下、越界时翻转，并在可用边界内保留安全边距。
- 气泡展示不修改目标界面、不冒充 `TextReplace`；若窗口呈现失败，主窗口仍获得焦点与完整结果。

## Delivery

- 默认 Bundle 构建要求本地生成的 13 项 OCR 支持文件全部存在，随后固定打包 UIA 与 OCR 两个
  Acquisition Worker；候选开关和旧的“标准 Bundle 不含 OCR”分支已删除。
- Tauri 注册独立 `interactive-translation-bubble` 窗口、读取/关闭命令和内存状态；最近块选择与
  负坐标跨屏边界使用确定性 Rust 测试固定。
- Vue 气泡以 `surface=translation-bubble` 独立启动，沿用现有主题、语言和设计 token；浏览器测试
  通过合成事件验证 OCR/UIA 来源、最近块、缺失译文、导航与 Esc 关闭。

## Verification

- `cargo test -p glyphshift-desktop-shell interactive_translation`：13/13 通过。
- `cargo clippy -p glyphshift-desktop-shell --all-targets -- -D warnings`：通过。
- `npm run build`：通过。
- 定向 Playwright：取词主面板与悬浮气泡 6/6 通过；截图只保存在本地 evidence。
- Impeccable 机械检查发现译文区侧边强调线后已移除该套路；最终截图人工复核通过。
- 默认 OCR Runtime Bundle：28 个文件、2 个 Acquisition Worker、13 个 OCR support files；Loader
  合同 1/1 通过。

## Residual risk

当前机器没有物理多显示器/混合缩放环境。负坐标和多屏选择已由确定性几何测试覆盖，真实 WGC 单屏
负坐标、受保护窗口和退出清理已有证据；物理跨屏与混合 DPI 仍应在发布前补一轮本地 smoke，但不再
阻止默认 Bundle 集成。
