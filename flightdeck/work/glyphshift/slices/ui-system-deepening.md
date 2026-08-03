# 桌面 UI 视觉系统深化

Status: Complete

## Goal

统一 Software、Settings、Workflow 编辑器及明暗主题的页面骨架与表面层级，使高密度桌面界面在
1440×900 和 960×640 下保持优雅、清晰和稳定。

## Current

- Software、Settings 与 Workflow 已复用同一工作表面；四类详情页共享全宽详情头。
- Software 和 Workflow 基础配置使用可响应的标签/控件横向表单行，960×640 保持足够输入宽度。
- Workflow 左侧分区当前项改为低强度 selection，右区保持单一内容表面。
- 浅色主题以冷灰框架包围纯白 Table 内容面；暗色使用石墨层级与低亮度蓝色主按钮。
- Fluent 2、Windows App Settings、Spectrum、Carbon、VS Code 与 Nuxt UI 调研已沉淀为实现规则。

## Decisions

- 保留现有高密度 Windows 管理器风格，以钴蓝替换 emerald 作为统一强调色。
- 列表页继续使用页头和满宽数据面板；详情页使用全宽详情头和单一工作表面。
- Workflow 只有一个左侧单层分区栏；Software 不创建伪导航，采用左右对齐表单行。
- Settings 占满页面剩余空间，内部保持单列、可读宽度和立即生效行为。
- 浅色 Table 内容面使用纯白，外层框架、表头、分页和边界保留冷灰层级，避免整页糊成一块。
- 继续复用 Nuxt UI；当前后端分页足以控制大集合渲染，不增加虚拟列表或新的 UI 库。

## Steps

- [x] 完成现状截图与源码审计。
- [x] 完成官方设计系统与成熟桌面工作台调研。
- [x] 建立共享表面 token 和全宽详情头。
- [x] 统一 Software 详情和 Settings 工作表面。
- [x] 收紧 Workflow 侧栏、活动项和基础表单的层级。
- [x] 更新设计合同与 Playwright 视觉覆盖。
- [x] 完成双主题、双尺寸视觉复核和全量验证。

## Verification

- `npm.cmd run build`：通过，903 modules transformed。
- `npm.cmd test`：49/49 通过。
- 1440×900 / 960×640 的 Software、Settings、Workflow，dark / light 视觉复核通过。
- 对比度：dark placeholder 4.60:1、light placeholder 4.79:1、light muted text 5.55:1。
- 用户复核后的主题微调：暗色主按钮使用深蓝底白字；浅色 Table 内容区使用纯白。
- Impeccable detector：0 项。

## Next

返回工作流 Runtime 错误反馈，修复 `AlreadyActive` 误报与多进程目标选择。

## References

- [UI 深化调研](../references/ui-system-deepening-research.md)
- [Glyphshift 设计系统](../../../../DESIGN.md)
