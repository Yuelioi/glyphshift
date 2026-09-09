# 发布根文档对齐

Status: Open

## Goal

让根目录 Markdown 与当前产品、领域语言和桌面设计系统保持一致：`CONTEXT.md` 只保留统一语言，
`PRODUCT.md` 聚焦用户、任务与产品承诺，`DESIGN.md` 从现有 token 和组件提取规范；中英文 README
保持发布事实一致，并在用户提供最终截图后完成界面预览编排。

## Current

当前推进用户文档与 Release 文档包。docs 已有入口、快速开始、工作流、字典、AI 翻译和常见问题共 6 页，以及单语言 docs.json；采用日常说法，不包含机器截图。两份 README 已加入口。发布脚本从内置 yueli-docs-publish 技能复制入 scripts，CI 不依赖本机技能路径。

现有 tag Release job 在同一 checkout 中打包 docs.zip，随安装包和候选清单一起发布，SHA256SUMS 增加文档包校验值，资产数量改为 4。维护与月离文档绑定步骤见 [文档发布说明](../../../scripts/docs-publishing.md)。尚未上传、绑定文档站、提交或推送。

本机打包 7 个文件成功，重复打包字节一致；UTF-8、相对链接、ZIP 完整性与隐私扫描通过。Release YAML 解析与定向发布合同通过。GitHub runner 上的实际发布尚待下一次授权发版验证。


应用内帮助已同步当前功能，中英文包含字典 JSON/CSV 与三种导入模式、批量导出及分页、排除字典、自动补全间隔与停止规则、AI 服务预设与提示词、刷新限制、实验适配器以及 GPL-3.0-only。保留五个页签，步骤、日常维护、配置说明及用量按主题分卡片；说明卡片宽屏双列、窄窗单列。3 项帮助 Playwright 测试、前端类型检查和构建通过，宽窄窗口截图已核对。桌面 App 尚未重建为本轮修改。

AGENTS 已增加经用户确认且通过隐私检查的发布截图窄例外。CONTEXT、PRODUCT 与 DESIGN 已收敛；
中英文 README 已完成用户导向内容并明确支持任意源语言与目标语言，当前仍暂不引用截图。仓库与桌面
package 原声明 MIT；按用户最新要求，现已统一为 `GPL-3.0-only`，根 `LICENSE` 使用 GNU 官方全文，第三方许可证保留。`v*` tag push 会校验版本、构建 Windows NSIS，并用
当前仓库 `GITHUB_TOKEN` 创建 GitHub Release。

本机原始截图与证据已在知识提炼后清理，未加入 README，也未进入跟踪文档。原 README 引用的两个
预览文件当前不存在，因此引用已移除，避免发布后出现破图。公开个人仓库已创建，`main` 与 annotated
tag `v0.2.0` 已推送；GitHub Actions 完成版本校验、Windows NSIS 构建和 Release 发布。公开 Release
包含安装包、候选清单与 SHA-256 文件，下载后交叉校验一致，并已成为 latest release。

发布前 Playwright 合同、全仓 Rust 测试、Unity 静态合同、正式 NSIS Release 构建和双轴审查均通过。
`nanoid` 锁定到修复 GHSA-2v37-7h3g-55p8 的 `3.3.18` 后，`npm audit` 已无 high/critical；仍有一个仅
影响本地开发服务器的 low 级 `esbuild` 告警。安装包当前未签名，Actions 另有 `actions/cache@v4`
旧 Node 运行时被 runner 兼容执行的非阻塞注解。

`v0.2.1` 可靠性更新的 GitHub Release 工作流已成功，安装包、候选清单与 SHA-256 三个资产均已发布。
`v0.2.2` 已完成一级页面满宽页头、About Tab、版本同步和正式 NSIS 候选构建；annotated tag 指向发布
提交并已与 `main` 原子推送，GitHub Release 工作流已成功。

`v0.2.3` 已同步版本，包含持续捕获的周期性落盘修复，以及审阅启动脚本显式使用原用户数据的
选项。实机效果参数捕获和中文回写、针对性 Rust 回归、合成性能对比与发布内容测试通过，
准备提交并推送 annotated tag，由现有工作流构建和校验 NSIS 资产。

本轮同步中英文 README：去除临时资产概念，统一字典术语，补充导入导出、排除字典、刷新限制及 VGUI / CatSystem2 实验边界。历史发布记录保留，不代表当前兼容性已经全部验收。中英文 README 内容合同通过（1/1），许可证元数据、第三方锁文件条目及 README 链接检查通过。

## Next

继续维护 [用户文档](../../../docs/index.md)。用户下一次授权发版时，核对 docs.zip 与安装包来自同一标签并完整上传；发布后按 [文档发布说明](../../../scripts/docs-publishing.md) 绑定月离文档目标文档集。不要自动发布或创建标签，已发布标签不移动。截图仍待用户提供并通过隐私检查。

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
- 用户最终选择 MIT；已补齐标准许可证文件及中英文 README、npm package 元数据。
- 发布前门禁已通过：Playwright 发布合同 4/4、AI 配置定向回归 3/3、全仓 Rust 测试退出码 0、Unity
  静态合同通过；正式构建的 9 个 Runtime Adapter 通过生产加载器校验并生成唯一 NSIS 安装包。
- 发布前依赖审计将传递依赖 `nanoid` 从 `3.3.16` 锁定到 `3.3.18`，消除一个 high 级告警；剩余一个
  low 级 `esbuild` 本地开发服务器告警，不影响打包后的桌面应用。
- 双轴审查最终为 Standards 0 hard / 0 judgement、Spec 0 finding；隐私问题与 Flightdeck 自洽问题均已
  在 tag 前修正。
- 已创建公开个人仓库并推送 `main` 与 annotated tag `v0.2.0`；GitHub Actions Release job 全部通过，
  三个发布资产齐全，安装包 SHA-256 与校验文件及候选清单一致。
- 已将桌面版本同步到 `0.2.1`，正式候选的 9 个 Runtime Adapter 通过生产加载器校验；发布提交、
  `main` 与 annotated tag `v0.2.1` 已推送，GitHub Release 工作流成功且三个发布资产齐全。
- 已将桌面版本同步到 `0.2.2`，统一一级页面满宽页头并新增 About 外部资源；正式 NSIS 候选、相关
  Playwright 与生产加载器校验通过，发布提交、`main` 与 annotated tag 已推送，Release 工作流执行中。

## References

- [稳定上下文](context.md)
- [执行计划](plan.md)
- [GUI 约定](../../knowledge/gui/nuxt-ui-tauri-vite-setup.md)
