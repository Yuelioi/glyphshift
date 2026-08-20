# 帮助页关于信息

Status: Finished

## Goal

为帮助页增加独立“关于”Tab，清楚展示 Glyphshift 当前版本、项目 GitHub 仓库和作者哔哩哔哩主页，
并通过受控的系统浏览器入口打开外部链接。

## Current

帮助页现在包含独立 About Tab，显示 package 驱动的完整应用版本、项目 GitHub 仓库与作者哔哩哔哩
主页。两条资源使用连续列表和命名按钮，通过系统默认浏览器打开；Tauri URL 白名单只新增 GitHub 与
Bilibili 所需域名。上一项页头与满宽内容轴修改已完整保留。

## Next

None

## Progress

- 已确定 About 使用连续资源行而非大卡片，显示真实 URL，并从 package 元数据读取完整版本。
- 已同步中文、English、PRODUCT 与 DESIGN；两个外部链接具有目标名称可访问标签和统一失败提示。
- 生产前端构建通过；结构与帮助交互 Playwright 19/19、About 视觉用例 2/2 通过；960×640 与
  1440×900 均无横向溢出。
- Impeccable 最终布局扫描为 0；未运行归档 UIA 测试。
- 已通过 `scripts/review-app.ps1` 重建并启动同步审阅版；9 个 Runtime Adapter 通过生产加载器校验。

## References

- [稳定上下文](context.md)
- [执行计划](plan.md)
- [GUI 约定](../../knowledge/gui/nuxt-ui-tauri-vite-setup.md)
