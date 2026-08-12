# 默认批大小与探针翻译筛选

Status: Complete

> 批次默认值部分后来由[全局 AI 批次设置](global-ai-batch-settings.md)取代；Probe 翻译状态筛选仍为当前实现。

## Deliverable

新建 AI Profile 默认每次最多发送 100 条候选；Probe 详情联合表提供“全部 / 未翻译 / 已翻译”筛选，
筛选在后端完整联合数据集上执行后再分页，并可与搜索和 Adapter 筛选组合、按 Probe 恢复。

## Steps

1. [x] 用 Core 查询合同和 Playwright 建立默认 100 与未翻译筛选红灯。
2. [x] 在 Core `ProbeQuery`、Desktop IPC 与 Vue 查询状态中贯通翻译状态筛选。
3. [x] 覆盖分页总数、搜索/Adapter 组合、空态、视图恢复和中英文文案。
4. [x] 运行受影响 Rust、生产构建与 Probe / AI Playwright，并重启本地真机实例。

## Verification

- 新建 Profile 的 Rust 与 GUI 默认值都是 100，已保存 Profile 的显式配置不被静默改写。
- “未翻译”按译文是否为空判断，不受技术状态名称影响；筛选后 `total` 与分页来自完整数据集。
- 真实数据、截图与运行日志继续只写入 `local-test/`。

## Result

- Rust 与 Vue 新建 Profile 默认值均为 100；显式保存的其他批大小继续保持原值。
- `ProbeTranslationFilter` 支持 `all / untranslated / translated`，在完整联合集合上与搜索、Adapter
  条件共同过滤后再计算总数与分页；页面以请求序号丢弃筛选切换期间迟到的旧分页响应。
- Probe 工具栏复用管理表筛选控件，并按 Probe 保存筛选状态；无匹配空态同时提示调整搜索或筛选。
- Capture 18 项、AI 15 项、桌面壳 70 项、AI / Probe Playwright 23 项、生产构建、格式和架构检查通过。
- 最新本地实例已用真实 Desktop IPC 验证默认 Profile 为 100，未翻译查询结果均为空译文。
