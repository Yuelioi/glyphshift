# Help 任务入口与全局键盘语义

Status: Complete

## Goal

把 Help 从实现目录改成能直接解决常见问题的任务入口，并补齐桌面应用的主内容跳转、工具栏名称、
heading 层级和浮层焦点恢复，使本轮 UI/UX 优化达到最终验收状态。

## Delivery

- [x] Help 首先提供“软件未启动”“权限不匹配”“没有捕获到文字”的恢复步骤与直接页面入口。
- [x] Adapter 目录从七列横向表格收敛为名称说明、适用技术、能力三块；版本、配置与官方文档渐进展开。
- [x] 应用提供首个 Tab 可发现的“跳到主要内容”入口和 `Alt+M` 加速键，目标主区域可被程序化聚焦。
- [x] 共享页头、详情头、列表工具栏和批量工具栏具有可区分的可访问名称。
- [x] Workflow 详情及相关复合表面的可见 heading 保持 `h1 → h2 → h3`，无跳级。
- [x] 共享菜单使用 Esc 关闭后把焦点交还触发器，并由 Playwright 固定为回归合同。

## Acceptance

- Help 在 1440×900 与 960×640、深色与浅色下先显示恢复任务，再显示 Adapter 目录，页面无横向溢出。
- 中英文 Help 都能从恢复项进入 Software、Settings 与 Probe；Adapter 技术文档能力与既有 URL 合同不变。
- Adapter 普通行不持续展示版本与配置，展开详情后可读并可打开具名官方文档；内部 ID、技术目标和原始 URL 不显示。
- 首次 Tab 聚焦可见的主内容跳转；Enter 与 `Alt+M` 都把焦点移到 `<main>`，随后 Tab 进入当前页面动作。
- 代表性页面和 Workflow 四个详情分区的可见 heading 不跳级，列表与页头工具栏能按名称定位。
- 受影响 Playwright、production build、Impeccable 最终检查及本地代表性视觉/键盘证据通过。

## Current

Help 已先呈现三个恢复任务，再解释工作模型与 Adapter；目录改用可扫描的连续列表，低频技术信息按单项
渐进展开。应用首个 Tab 与 `Alt+M` 均可聚焦主内容，共享页头和列表工具栏已有具名语义，相关复合页面
heading 连续，菜单 Esc 焦点恢复由自动化测试固定。

## Verification

- Help、Management、Settings、Workflow 与 Visual 受影响 Playwright 共 48 项通过。
- Desktop production build 通过，仅保留既有的 Vite 大 chunk 提示。
- 本地代表性 Playwright 视觉/键盘检查 1 项通过，覆盖 1440/960、深色/浅色、Adapter 展开与跳转链接；
  960 宽 Adapter 列表无横向溢出，heading 轮廓为连续的 `h1 → h2 → h3`。
- Impeccable 最终 detector 为 0 findings；`git diff --check` 通过。

## Next

None.
