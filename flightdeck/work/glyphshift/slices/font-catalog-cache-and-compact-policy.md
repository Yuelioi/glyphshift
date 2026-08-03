# 字体目录缓存与紧凑策略控件

## Goal

避免 Glyphshift 每次启动都同步重扫系统字体，并把 Workflow Font Policy 的覆盖范围改成紧凑、
易读且不抢占横向空间的控件。

## Current

已完成。桌面端已建立持久字体目录缓存：缓存有效时普通启动直接读取，首次缺失时扫描并写入；
显式刷新命令才重新扫描、替换缓存和当前 Desktop Snapshot。Font Policy 已把两个满宽按钮收成
“应用范围”选择器，并在字体目录标题显示缓存数量与刷新按钮。

## Decisions

- 字体目录是可再生的机器缓存，不属于 Workflow、Dictionary 或 AppSettings。
- 缓存缺失、损坏或 schema 不匹配时重新扫描；启动期写缓存失败不阻断应用。
- 用户主动刷新失败时返回稳定错误并保留当前目录，不把旧缓存先清空。
- Coverage 仍使用既有 `dictionary_matches` / `all_observations` 领域值，只改变交互控件。
- 刷新后复用完整 Desktop Snapshot seam，使 Backend 编译环境和 Vue 目录保持一致。

## Verification

- Rust 红回归覆盖首次扫描、跨实例读缓存、显式刷新和刷新后重开；实现后已通过。
- 聚焦 Playwright 4 项通过，覆盖紧凑选择器、缓存数量、刷新入口、coverage 保存和多 Target 隔离。
- Desktop Shell/Backend 38 项 Rust 测试、Clippy、fmt、生产构建和完整 Playwright 40 项通过。
- 1180×760 与 1440×900 视觉验收通过；字体目录、刷新动作和 coverage 控件保持清楚且不产生
  新滚动或弹窗高度变化。

## Next

- 等待用户验收与下一次提交授权；后续实现回到工作流 Runtime 错误反馈的幂等接管。
