# 独立工具页统一布局

Status: Finished

## Goal

让 Settings 与 Help 共享同一套独立工具页骨架：全宽详情标题带、980px 最大内容列、统一首屏间距与
滚动行为；保留两页各自的设置操作与帮助阅读结构。

## Current

Settings 与 Help 已共同接入 `UtilityPageShell`。两页现在共享全宽 64px 标题带、980px 最大内容列、
20px 首屏间距、对称滚动条预留和唯一页面滚动区；Settings 保留配置分组，Help 保留连续阅读结构和
Adapter 渐进展开。

## Next

None.

## Progress

- 用户明确要求 Help 参考 Settings，并为两个独立页面统一最大宽度。
- 保留 Help 的恢复任务、产品模型说明、Adapter 渐进展开和既有文档 URL 合同。
- 组件合同固定 Settings 与 Help 必须复用共享工具页壳层，几何回归固定标题与内容列共轴和 20px 节奏。
- 受影响 Playwright 39 项通过；production build 通过，仅保留既有的大 chunk 与插件耗时提示。
- 1520/960、深色/浅色和中英文代表性视觉检查通过，Help Adapter 在紧凑内容列中提前折为两列。
- Impeccable layout detector 为 0 findings；`git diff --check` 与本机信息扫描通过。

## References

- [稳定上下文](context.md)
- [设置页构图工作](../settings-page-composition/index.md)
- [设计系统](../../../DESIGN.md)
