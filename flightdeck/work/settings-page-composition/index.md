# 设置页构图与页头对齐

Status: Finished

## Goal

让 Settings 在宽窗口中形成一条明确的内容轴，标题、说明、分区定位与设置面板不再左右错位；同时保持
960px 紧凑窗口的表单可读性、现有立即生效行为和 Glyphshift 的高密度 Windows 工具气质。

## Current

Settings 的详情标题内层、错误提示与 980px 设置面板现已使用同一内容轴；纵向滚动区在左右对称预留
滚动条空间，因此宽屏和紧凑窗口都不会产生半个滚动条宽度的视觉偏移。标题带到首个分组只保留一次
20px 间距，既有设置行为、表单密度和其他详情页布局保持不变。

## Next

None.

## Progress

- 已确认不把设置表单直接拉满窗口，也不增加营销式大卡片或无意义状态摘要。
- 已确认保留共享详情页头语义，但允许 Settings 使用自己的受限宽度内层构图。
- 新增宽屏/紧凑窗口几何回归，固定标题、设置布局和首个分组的左右边界与 20px 纵向节奏。
- 共享详情头受影响的 67 项 Playwright 通过；production build 通过，仅保留既有的大 chunk 提示。
- 1520/960、深色/浅色和中英文代表性视觉检查通过；Impeccable layout detector 为 0 findings。
- `git diff --check`、新增文本隐私扫描和 `local-test` 零跟踪检查通过。

## References

- [稳定上下文](context.md)
- [上一轮 UI/UX 工作](../desktop-ui-ux-consistency/index.md)
- [设计系统](../../../DESIGN.md)
