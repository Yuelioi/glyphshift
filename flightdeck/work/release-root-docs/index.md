# 发布根文档对齐

Status: Open

## Goal

让根目录 Markdown 与当前产品、领域语言和桌面设计系统保持一致：`CONTEXT.md` 只保留统一语言，
`PRODUCT.md` 聚焦用户、任务与产品承诺，`DESIGN.md` 从现有 token 和组件提取规范；中英文 README
保持发布事实一致，并在用户提供最终截图后完成界面预览编排。

## Current

AGENTS 已增加经用户确认且通过隐私检查的发布截图窄例外。CONTEXT、PRODUCT 与 DESIGN 已收敛；
中英文 README 已完成用户导向内容并明确支持任意源语言与目标语言，当前仍暂不引用截图。`v*` tag
push 会校验版本、构建 Windows NSIS，并用当前仓库 `GITHUB_TOKEN` 创建 GitHub Release。

当前仅有的 AE/探针截图包含真实安装路径，已移入 `local-test/evidence/`，未加入 README，也未进入
跟踪文档。原 README 引用的两个预览文件当前不存在，因此引用已移除，避免发布后出现破图。

## Next

等待用户提供最终匿名截图；收到后先做隐私检查，再同步加入中英文 README。

## Progress

- 已完成根目录 Markdown 清点；AGENTS 的通用隐私规则仍保留，并增加 README 发布截图窄例外。
- 用户提供发布截图后，已为 README 专用的脱敏图片补充 AGENTS 窄例外。
- 已重写 CONTEXT、PRODUCT 与 DESIGN，并同步 `.impeccable/design.json`。
- README 当前不引用截图；一张含真实路径的截图已移入本机证据区，等待最终匿名截图后再编排。
- 根文档内容合同 2/2 通过，6 份根 Markdown 隐私扫描为 0，JSON 与差异检查通过。
- 用户确认产品不只面向中文；README 与 PRODUCT 已改为任意源语言到任意目标语言。
- 已新增当前仓库 tag push 自动构建、校验并发布 Windows NSIS 的 GitHub Actions workflow。
- workflow YAML、tag 与三处版本合同、发布内容测试和安全扫描通过；本机完整 Release 已生成唯一 NSIS，
  9 个 Runtime Adapter 通过生产加载器校验，安装包哈希与候选清单一致。

## References

- [稳定上下文](context.md)
- [执行计划](plan.md)
- [GUI 约定](../../knowledge/gui/nuxt-ui-tauri-vite-setup.md)
