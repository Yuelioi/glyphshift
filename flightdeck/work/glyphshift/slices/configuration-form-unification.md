# 轻量配置表面统一

Status: Complete

## Goal

让 Settings 外观、Workflow 基础配置与 Software 编辑使用同一种配置表单空间语法，同时保留三页
真实的导航、控件宽度和保存语义差异。

## Current

- 新增共享 `ManagementFormSection` 与 `ManagementFormRow`，三页不再各自维护表单网格常量。
- 配置面板以 980px 最大宽度在各自可用工作区水平居中；完整边界、分组标题、说明、184px 标签轨、
  32px 列间距和字段分隔线保持一致。
- Settings 枚举选择器使用 compact 宽度并贴齐右侧；Workflow 与 Software 文本字段使用 fill 宽度。
- 三页的详情主体直接使用应用主背景画布，不再用满高外层 Card 包裹配置面板；Workflow 只为真实
  分区导航保留侧栏与分隔线。
- Settings 复用全宽详情标题带，并通过可选返回动作支持无父级列表的工具型子页。
- 字段行以容器宽度为依据，在不足 620px 时折为上下布局；不再依赖三个不一致的 viewport 断点。
- Software 补齐“软件资料”分组上下文；Settings 与 Workflow 保留各自已有的真实说明。

## Decisions

- 共享的是主体配置表面，不把三页强制成同一种页面导航。
- Workflow 保留四分区左栏；Software 继续使用无伪侧栏的独立详情页；Settings 继续即时生效且没有
  保存或未保存状态。
- 一级管理页与二级详情页使用不同承载语法：前者需要连续的数据表面，后者把空白留给主画布，只
  为实际内容建立面板。
- 图标和字段说明是可选内容，控件宽度由 compact/fill 语义表达，不由页面局部数字表达。
- Operate 密度保持 11–12px 正文与薄分隔线；配置面板只使用边界和表面层级，不增加阴影、动画或
  额外设置。

## Steps

- [x] 并排审计三页宽屏、紧凑、明暗主题及源码布局。
- [x] 建立共享配置分组和字段行组件。
- [x] 迁移 Settings、Workflow 基础配置与 Software 编辑并补齐双语文案。
- [x] 增加共享组件结构合同并完成完整回归。
- [x] 更新设计合同与 Work 稳定 UI 决定。

## Verification

- `npm run build`：通过，908 modules transformed。
- `npm test -- --config playwright.config.ts`：49/49 通过。
- 1440×900 / 960×640 的 Settings、Workflow 基础配置、Software 编辑，dark / light 视觉复核通过。
- Impeccable UI detector：0 项。

## Next

返回工作流 Runtime 错误反馈，修复 `AlreadyActive` 误报与多进程目标选择。
