# AE 2025 文字获取兼容性

Status: Open

## Goal

在现有 DirectWrite Adapter 内恢复 AE 2025 的文字采集和统一排版的中文显示，保留不支持的局部排版，并验证实际画面、更新与停用恢复。

## Current

采集和统一 Typography 的替换已修复。此前提前检查替换资格导致面板零观测；随后用户试用确认有原文和译文但画面仍是英文。当前在实际绘制时复制目标的统一排版与最新尺寸，按译文长度重建字符范围，不改变目标原对象。没有新增 Adapter 或 Adobe 私有分支。

真实 AE 原生合成设置的 `3D Renderer` 和自带脚本面板的 `Points Follow Nulls` 均已通过生产 Runtime 的中文显示、下一代更新和停用精确像素恢复。x64/x86 合同覆盖每架构 60 个格式组合，以及同一布局的译文更新、移除和停用。

完整工作流还暴露空布局地址复用：空创建保留旧原文关联，导致面板标题残影。已补上每次成功创建时覆盖或移除关联，并用真实 DirectWrite 复用空布局的像素红例复现、修复；两个架构均转绿。

最后通过 `scripts/review-app.ps1 -UseUserData` 同源重建桌面与 Runtime，正常重启空白 AE 目标并恢复完整用户工作流。生产 loader 验证 13 个 Adapter；最终 Playwright 确认原字典按钮译文可见、空白区域与修复前无翻译基准像素精确一致。AE 实际加载的文件与审阅包和通过合同的产物一致。桌面与 AE 均保持打开供试用。

## Next

发布前验证通过。提交本次通用修复并推送 v0.4.1 标签，等待 Windows Release 流水线完成。

## Progress

- 完成 Adobe 官方改版研究和两版本真实入口验证；主面板从 GDI+ 转为 DirectWrite/Direct2D。
- 采集由菜单 54 条恢复为 92 条（含 38 条 DirectWrite 面板原文），观察模式和观察加替换模式均通过。
- 中文像素红例转绿，复制当前布局格式；局部格式、inline object 和 drawing effect 继续原样透传。
- 原生合成设置与脚本面板两项 Playwright 实机可见性用例通过；两代命中非零，停用后文本区域像素精确恢复。
- 完整用户工作流确认中文可见，并定位标题残影的空布局地址复用；新增保护和两个架构的回归。
- 最终完整用户工作流可见性与无残影像素验收通过，格式化及 diff 检查通过，应用保持打开。
- 用户试用通过并授权发布 0.4.1，包含提交、推送和 Release。
- 部分窄控件出现中文逐字换行或裁切；截图符合布局宽度限制，但未测量确认根因。用户接受保留此限制并继续发布。
- 没有构建或执行归档 UIA/OCR。

- 0.4.1 发布前活动包回归：589 项通过、0 项失败、26 项忽略；前端生产构建通过。版本与锁文件统一，暂存范围及隐私检查通过。

## References

- [稳定上下文](context.md)
- [中文显示与残影修复](references/typography-translation.md)
- [采集阶段诊断](references/capture-diagnosis.md)
- [Adobe 官方来源研究](references/adobe-ui-research.md)
- [复杂格式原生合同](../../../crates/adapters/platform/native-host/tests/native_directwrite_capture_formatting_contract.rs)
- [运行时文字 Adapter 验证标准](../../knowledge/rendering/runtime-text-adapter-validation.md)
